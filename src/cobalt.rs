use std::fs::{self, File};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::ffi::OsStr;
use liquid::{Value, Object};
use chrono::{UTC, FixedOffset};
use chrono::offset::TimeZone;
use rss::{Channel, Rss};
use serde_yaml;

use document::Document;
use error::{ErrorKind, Result};
use config::{Config, Dump};
use files::FilesBuilder;

/// The primary build function that transforms a directory into a site
pub fn build(config: &Config) -> Result<()> {
    trace!("Build configuration: {:?}", config);

    let source = Path::new(&config.source);
    let dest = Path::new(&config.dest);

    let template_extensions: Vec<&OsStr> = config
        .template_extensions
        .iter()
        .map(OsStr::new)
        .collect();

    let layouts = source.join(&config.layouts);
    let mut layouts_cache = HashMap::new();
    let posts_path = source.join(&config.posts);

    debug!("Layouts directory: {:?}", layouts);
    debug!("Posts directory: {:?}", posts_path);
    debug!("Draft mode enabled: {}", config.include_drafts);
    if config.include_drafts {
        debug!("Draft directory: {:?}", config.drafts);
    }

    let mut documents = vec![];

    let ignore_dest = {
        let rel_dest = dest.strip_prefix(source);
        if let Ok(rel_dest) = rel_dest {
            let ignore_dest = rel_dest.join("**/*");
            let ignore_dest = ignore_dest
                .to_str()
                .ok_or_else(|| format!("Cannot convert pathname {:?} to UTF-8", rel_dest))?
                .to_owned();
            Some(ignore_dest)
        } else {
            None
        }
    };

    let mut page_files = FilesBuilder::new(source)?;
    page_files
        .add_ignore(&format!("!{}", config.posts))?
        .add_ignore(&format!("!{}/**", config.posts))?
        .add_ignore(&format!("{}/**/_*", config.posts))?
        .add_ignore(&format!("{}/**/_*/**", config.posts))?;
    for line in &config.ignore {
        page_files.add_ignore(line.as_str())?;
    }
    if let Some(ref ignore_dest) = ignore_dest {
        page_files.add_ignore(ignore_dest)?;
    }
    let page_files = page_files.build()?;
    for file_path in page_files
            .files()
            .filter(|p| {
                        template_extensions
                            .contains(&p.extension().unwrap_or_else(|| OsStr::new("")))
                    }) {
        let src_path = source.join(file_path.as_path());

        let new_path = source.join(file_path);
        let new_path = new_path
            .strip_prefix(source)
            .expect("Entry not in source folder");

        // if the document is in the posts folder it's considered a post
        let is_post = src_path.starts_with(posts_path.as_path());

        let doc = Document::parse(src_path.as_path(), new_path, is_post, &config.post_path)?;
        if !doc.is_draft || config.include_drafts {
            documents.push(doc);
        }
    }

    if config.include_drafts {
        let drafts_root = source.join(&config.drafts);
        let mut draft_files = FilesBuilder::new(drafts_root.as_path())?;
        for line in &config.ignore {
            draft_files.add_ignore(line.as_str())?;
        }
        let draft_files = draft_files.build()?;
        for file_path in draft_files
                .files()
                .filter(|p| {
                            template_extensions
                                .contains(&p.extension().unwrap_or_else(|| OsStr::new("")))
                        }) {
            let src_path = drafts_root.join(file_path.as_path());

            let new_path = posts_path.join(file_path);
            let new_path = new_path
                .strip_prefix(source)
                .expect("Entry not in source folder");
            let doc = try!(Document::parse(src_path.as_path(), new_path, true, &config.post_path));
            documents.push(doc);
        }
    }

    // January 1, 1970 0:00:00 UTC, the beginning of time
    let default_date = UTC.timestamp(0, 0).with_timezone(&FixedOffset::east(0));

    let (mut posts, documents): (Vec<Document>, Vec<Document>) =
        documents.into_iter().partition(|x| x.is_post);

    // sort documents by date, if there's no date (none was provided or it couldn't be read) then
    // fall back to the default date
    posts.sort_by(|a, b| {
                      b.date
                          .unwrap_or(default_date)
                          .cmp(&a.date.unwrap_or(default_date))
                  });

    if &config.post_order == "asc" {
        posts.reverse();
    }

    // collect all posts attributes to pass them to other posts for rendering
    let simple_posts_data: Vec<Value> = posts
        .iter()
        .map(|x| Value::Object(x.attributes.clone()))
        .collect();

    trace!("Generating posts");
    for (i, mut post) in &mut posts.iter_mut().enumerate() {
        trace!("Generating {}", post.path);

        // posts are in reverse date order, so previous post is the next in the list (+1)
        if let Some(previous) = simple_posts_data.get(i + 1) {
            post.attributes
                .insert("previous".to_owned(), previous.clone());
        }
        if i >= 1 {
            if let Some(next) = simple_posts_data.get(i - 1) {
                post.attributes.insert("next".to_owned(), next.clone());
            }
        }

        if config.dump.contains(&Dump::Liquid) {
            create_liquid_dump(dest, &post.path, &post.content, &post.attributes)?;
        }

        let mut context = post.get_render_context(&simple_posts_data);

        try!(post.render_excerpt(&mut context, source, &config.excerpt_separator));
        let post_html = try!(post.render(&mut context, source, &layouts, &mut layouts_cache));
        try!(create_document_file(&post_html, &post.path, dest));
    }

    // check if we should create an RSS file and create it!
    if let Some(ref path) = config.rss {
        try!(create_rss(path, dest, config, &posts));
    }

    // during post rendering additional attributes such as content were
    // added to posts. collect them so that non-post documents can access them
    let posts_data: Vec<Value> = posts
        .into_iter()
        .map(|x| Value::Object(x.attributes))
        .collect();

    trace!("Generating other documents");
    let timestamp = Value::Str(UTC::now().timestamp().to_string());
    for mut doc in documents {
        trace!("Generating {}", doc.path);

        if config.dump.contains(&Dump::Liquid) {
            create_liquid_dump(dest, &doc.path, &doc.content, &doc.attributes)?;
        }
        doc.attributes
            .insert("timestamp".to_owned(), timestamp.clone());

        let mut context = doc.get_render_context(&posts_data);
        let doc_html = try!(doc.render(&mut context, source, &layouts, &mut layouts_cache));
        try!(create_document_file(&doc_html, &doc.path, dest));
    }

    // copy all remaining files in the source to the destination
    {
        info!("Copying remaining assets");
        let mut asset_files = FilesBuilder::new(source)?;
        for line in &config.ignore {
            asset_files.add_ignore(line.as_str())?;
        }
        if let Some(ref ignore_dest) = ignore_dest {
            asset_files.add_ignore(ignore_dest)?;
        }
        let asset_files = asset_files.build()?;
        for file_path in asset_files
                .files()
                .filter(|p| {
                            !template_extensions
                                 .contains(&p.extension().unwrap_or_else(|| OsStr::new("")))
                        }) {
            {
                let parent_dir = file_path.parent();
                if let Some(parent_dir) = parent_dir {
                    let parent_dir = dest.join(parent_dir);
                    fs::create_dir_all(parent_dir.as_path())?;
                    debug!("Created new directory {:?}", parent_dir);
                }
            }
            let src_file = source.join(file_path.as_path());
            let dest_file = dest.join(file_path);

            fs::copy(src_file.as_path(), dest_file.as_path())
                .map_err(|e| format!("Could not copy {:?} into {:?}: {}", src_file, dest_file, e))?;
            debug!("Copied {:?} to {:?}", src_file, dest_file);
        }
    }

    Ok(())
}

fn create_liquid_dump(dest: &Path, path: &str, content: &str, attributes: &Object) -> Result<()> {
    let mut liquid_file_path = dest.join(path);
    let mut liquid_file_name = OsStr::new("_").to_os_string();
    {
        let original_file_name = liquid_file_path.file_name().ok_or("File name missing")?;
        liquid_file_name.push(original_file_name);
        liquid_file_name.push(".liquid");
    }
    liquid_file_path.set_file_name(liquid_file_name);

    let mut dump_file_path = liquid_file_path.clone();
    dump_file_path.set_extension(".yml");

    info!("Dumping content at {}", liquid_file_path.display());
    let mut liquid_out = fs::File::create(liquid_file_path)?;
    liquid_out.write_all(content.as_bytes())?;

    info!("Dumping attributes at {}", dump_file_path.display());
    let mut dump_out = fs::File::create(dump_file_path)?;
    let values = serde_yaml::to_string(attributes)?;
    dump_out.write_all(values.as_bytes())?;

    Ok(())
}

// creates a new RSS file with the contents of the site blog
fn create_rss(path: &str, dest: &Path, config: &Config, posts: &[Document]) -> Result<()> {
    match (&config.name, &config.description, &config.link) {
        // these three fields are mandatory in the RSS standard
        (&Some(ref name), &Some(ref description), &Some(ref link)) => {
            trace!("Generating RSS data");

            let items = posts.iter().map(|doc| doc.to_rss(link)).collect();

            let channel = Channel {
                title: name.to_owned(),
                link: link.to_owned(),
                description: description.to_owned(),
                items: items,
                ..Default::default()
            };

            let rss_string = Rss(channel).to_string();
            trace!("RSS data: {}", rss_string);

            let rss_path = dest.join(path);

            let mut rss_file = try!(File::create(&rss_path));
            try!(rss_file.write_all(&rss_string.into_bytes()));

            info!("Created RSS file at {}", rss_path.display());
            Ok(())
        }
        _ => Err(ErrorKind::ConfigFileMissingFields.into()),
    }
}

fn create_document_file<T: AsRef<Path>, R: AsRef<Path>>(content: &str,
                                                        path: T,
                                                        dest: R)
                                                        -> Result<()> {
    // construct target path
    let file_path = dest.as_ref().join(path);

    // create target directories if any exist
    if let Some(parent) = file_path.parent() {
        try!(fs::create_dir_all(parent).map_err(|e| {
                                                    format!("Could not create {:?}: {}", parent, e)
                                                }));
    }

    let mut file =
        try!(File::create(&file_path).map_err(|e| {
                                                  format!("Could not create {:?}: {}", file_path, e)
                                              }));

    try!(file.write_all(content.as_bytes()));
    info!("Created {}", file_path.display());
    Ok(())
}
