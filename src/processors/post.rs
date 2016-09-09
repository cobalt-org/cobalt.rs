use std::path::{PathBuf, Path};
use util::*;
use walkdir::{WalkDir, DirEntry};
use document::Document;
use chrono::{UTC, FixedOffset};
use chrono::offset::TimeZone;
use liquid::Value;
use config::Config;
use error::{Result, Error};
use std::fs::{self, File};
use std::io::{self, Write, ErrorKind};
use std::sync::Arc;
use processor::Processor;
use crossbeam;

pub struct Post;

impl Processor for Post {
    fn match_dir(&mut self, entry: &DirEntry, config: &Config) -> bool {
        // join("") makes sure path has a trailing slash
        let source = PathBuf::from(&config.source).join("");
        let source = source.as_path();

        let posts = source.join(&config.posts);
        let posts = posts.as_path();

        compare_paths(entry.path(), posts)
    }

    fn process(&mut self, dir: DirEntry, config: &Config) -> Result<()> {
        let source = PathBuf::from(&config.source).join("");
        let source = source.as_path();
        let dest = PathBuf::from(&config.dest).join("");
        let dest = dest.as_path();

        let layouts = source.join(&config.layouts);
        let layouts = layouts.as_path();

        let mut documents = vec![];

        for entry in WalkDir::new(&dir.path())
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok()) {
            let entry_path = &entry.path();
            let new_path = entry_path.strip_prefix(source).expect("Entry not in source folder");
            let doc = try!(Document::parse(entry_path, new_path, true, &config.post_path));

            if !doc.is_draft || config.include_drafts {
                documents.push(doc);
            }
        }

            // January 1, 1970 0:00:00 UTC, the beginning of time
        let default_date = UTC.timestamp(0, 0).with_timezone(&FixedOffset::east(0));

        // sort documents by date, if there's no date (none was provided or it couldn't be read) then
        // fall back to the default date
        documents.sort_by(|a, b| b.date.unwrap_or(default_date).cmp(&a.date.unwrap_or(default_date)));

        // check if we should create an RSS file and create it!
        if let &Some(ref path) = &config.rss {
            try!(create_rss(path, dest, &config, &documents));
        }

        // these are the attributes of all documents that are posts, so that they can be
        // passed to the renderer
        // TODO: do we have to clone these?
        let post_data: Vec<Value> = documents.iter()
            .filter(|x| x.is_post)
            .map(|x| Value::Object(x.attributes.clone()))
            .collect();

        // thread handles to join later
        let mut handles = vec![];

        // generate documents (in parallel)
        crossbeam::scope(|scope| {
            let post_data = Arc::new(post_data);

            for doc in &documents {
                trace!("Generating {}", doc.path);
                let post_data = post_data.clone();

                let handle = scope.spawn(move || {
                    let content = try!(doc.as_html(&source, &post_data, &layouts));
                    create_document_file(content, &doc.path, dest)
                });
                handles.push(handle);
            }
        });

        for handle in handles {
            try!(handle.join());
        }

        Ok(())
    }
}


use rss::{Channel, Rss};

// creates a new RSS file with the contents of the site blog
pub fn create_rss(path: &str, dest: &Path, config: &Config, documents: &[Document]) -> Result<()> {
    match (&config.name, &config.description, &config.link) {
        // these three fields are mandatory in the RSS standard
        (&Some(ref name), &Some(ref description), &Some(ref link)) => {
            trace!("Generating RSS data");

            let items = documents.iter()
                .filter(|x| x.is_post)
                .map(|doc| doc.to_rss(link))
                .collect();

            let channel = Channel {
                title: name.to_owned(),
                link: link.to_owned(),
                description: description.to_owned(),
                items: items,
                ..Default::default()
            };

            let rss = Rss(channel);
            let rss_string = rss.to_string();
            trace!("RSS data: {}", rss_string);

            let rss_path = dest.join(path);

            let mut rss_file = try!(File::create(&rss_path));
            try!(rss_file.write_all(&rss_string.into_bytes()));

            info!("Created RSS file at {}", rss_path.display());
            Ok(())
        }
        _ => {
            Err(Error::from("name, description and link need to be defined in the config file to \
                             generate RSS"))
        }
    }
}



fn create_document_file<T: AsRef<Path>>(content: String, path: T, dest: &Path) -> Result<()> {
    // construct target path
    let file_path_buf = dest.join(path);
    let file_path = file_path_buf.as_path();

    // create target directories if any exist
    if let Some(parent) = file_path.parent() {
        try!(create_dir_all(parent).map_err(|e| format!("Could not create {:?}: {}", parent, e)));
    }

    let mut file = try!(File::create(&file_path)
                        .map_err(|e| format!("Could not create {:?}: {}", file_path, e)));

    try!(file.write_all(&content.into_bytes()));
    info!("Created {}", file_path.display());
    Ok(())
}

/// A slightly less efficient implementation of fs::create_dir_all
/// that eliminates the race condition problems of the original
fn create_dir_all(path: &Path) -> io::Result<()> {
    let mut new_path = PathBuf::new();
    for component in path {
        new_path.push(component);
        match fs::create_dir(&new_path) {
            Ok(_) => {}
            Err(ref e) if e.kind() == ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
