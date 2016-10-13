use crossbeam;

use std::fs::{self, File};
use std::io::{self, Write, ErrorKind};
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use liquid::Value;
use walkdir::{WalkDir, DirEntry, WalkDirIterator};
use document::Document;
use error::{Error, Result};
use config::Config;
use chrono::{UTC, FixedOffset};
use chrono::offset::TimeZone;
use rss::{Channel, Rss};
use std::sync::Arc;
use glob::Pattern;

fn ignore_filter(entry: &DirEntry, source: &Path, ignore: &[Pattern]) -> bool {
    if compare_paths(entry.path(), source) {
        return true;
    }
    let path = entry.path().strip_prefix(&source).unwrap_or(entry.path());
    let file_name = entry.file_name().to_str().unwrap_or("");
    if file_name.starts_with('_') || file_name.starts_with('.') {
        return false;
    }
    !ignore.iter().any(|p| p.matches_path(path))
}

fn compare_paths(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(p), Ok(p2)) => p == p2,
        _ => false,
    }
}

fn contain_path(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(p1), Ok(p2)) => p1.starts_with(p2),
        _ => false,
    }
}

/// The primary build function that transforms a directory into a site
pub fn build(config: &Config) -> Result<()> {
    trace!("Build configuration: {:?}", config);

    // join("") makes sure path has a trailing slash
    let source = PathBuf::from(&config.source).join("");
    let source = source.as_path();
    let dest = PathBuf::from(&config.dest).join("");
    let dest = dest.as_path();

    let template_extensions: Vec<&OsStr> = config.template_extensions
        .iter()
        .map(OsStr::new)
        .collect();

    let layouts = source.join(&config.layouts);
    let layouts = layouts.as_path();
    let posts = source.join(&config.posts);
    let posts = posts.as_path();

    debug!("Layouts directory: {:?}", layouts);
    debug!("Posts directory: {:?}", posts);
    debug!("Draft mode enabled: {}", config.include_drafts);
    if config.include_drafts {
        debug!("Draft directory: {:?}", config.drafts);
    }

    let mut documents = vec![];

    let walker = WalkDir::new(&source)
        .into_iter()
        .filter_entry(|e| {
            (ignore_filter(e, source, &config.ignore) || compare_paths(e.path(), posts)) &&
            !compare_paths(e.path(), dest)
        })
        .filter_map(|e| e.ok());

    for entry in walker {
        let entry_path = entry.path();
        let extension = &entry_path.extension().unwrap_or(OsStr::new(""));
        if template_extensions.contains(extension) {
            // if the document is in the posts folder it's considered a post
            let is_post = entry_path.parent().map(|p| contain_path(p, posts)).unwrap_or(false);
            let new_path = entry_path.strip_prefix(source).expect("Entry not in source folder");

            let doc = try!(Document::parse(&entry_path, new_path, is_post, &config.post_path));
            if !doc.is_draft || config.include_drafts {
                documents.push(doc);
            }
        }
    }

    if config.include_drafts {
        let drafts = source.join(&config.drafts);
        let drafts = drafts.as_path();

        let walker = WalkDir::new(drafts)
            .into_iter()
            .filter_entry(|e| {
                (ignore_filter(e, source, &config.ignore) || compare_paths(e.path(), drafts)) &&
                !compare_paths(e.path(), dest)
            })
            .filter_map(|e| e.ok());

        for entry in walker {
            let entry_path = entry.path();
            let extension = &entry_path.extension().unwrap_or(OsStr::new(""));
            let new_path =
                posts.join(entry_path.strip_prefix(drafts).expect("Draft not in draft folder!"));
            let new_path = new_path.strip_prefix(source).expect("Entry not in source folder");
            if template_extensions.contains(extension) {
                let doc = try!(Document::parse(&entry_path, new_path, true, &config.post_path));
                documents.push(doc);
            }
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

    // copy all remaining files in the source to the destination
    if !compare_paths(source, dest) {
        info!("Copying remaining assets");
        let source_str = try!(source.to_str()
            .ok_or(format!("Cannot convert pathname {:?} to UTF-8", source)));

        let walker = WalkDir::new(&source)
            .into_iter()
            .filter_entry(|e| {
                ignore_filter(e, source, &config.ignore) &&
                !template_extensions.contains(&e.path()
                    .extension()
                    .unwrap_or(OsStr::new(""))) && !compare_paths(e.path(), dest)
            })
            .filter_map(|e| e.ok());

        for entry in walker {
            let entry_path = try!(entry.path()
                .to_str()
                .ok_or(format!("Cannot convert pathname {:?} to UTF-8", entry.path())));

            let relative = try!(entry_path.split(source_str)
                .last()
                .ok_or(format!("Empty path")));

            if try!(entry.metadata()).is_dir() {
                try!(fs::create_dir_all(&dest.join(relative)));
                debug!("Created new directory {:?}", dest.join(relative));
            } else {
                if let Some(parent) = Path::new(relative).parent() {
                    try!(fs::create_dir_all(&dest.join(parent)));
                }

                try!(fs::copy(entry.path(), &dest.join(relative))
                    .map_err(|e| format!("Could not copy {:?}: {}", entry.path(), e)));
                debug!("Copied {:?} to {:?}", entry.path(), dest.join(relative));
            }
        }
    }

    Ok(())
}

// creates a new RSS file with the contents of the site blog
fn create_rss(path: &str, dest: &Path, config: &Config, documents: &[Document]) -> Result<()> {
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
