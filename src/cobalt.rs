use std::fs::{self, File};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::ffi::OsStr;
use liquid::Value;
use walkdir::{WalkDir, DirEntry, WalkDirIterator};
use document::Document;
use error::{ErrorKind, Result};
use config::Config;
use chrono::{UTC, FixedOffset};
use chrono::offset::TimeZone;
use rss::{Channel, Rss};
use glob::Pattern;

fn ignore_filter(entry: &DirEntry, source: &Path, ignore: &[Pattern]) -> bool {
    if compare_paths(entry.path(), source) {
        return true;
    }
    let path = entry.path();
    let path = path.strip_prefix(&source).unwrap_or(path);
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

/// Checks if one path is the starting point of another path.
fn starts_with_path(this: &Path, starts_with: &Path) -> bool {
    match (fs::canonicalize(this), fs::canonicalize(starts_with)) {
        (Ok(p), Ok(p2)) => p.starts_with(p2),
        _ => false,
    }
}

/// The primary build function that transforms a directory into a site
pub fn build(config: &Config) -> Result<()> {
    trace!("Build configuration: {:?}", config);

    let source = Path::new(&config.source);
    let dest = Path::new(&config.dest);

    let template_extensions: Vec<&OsStr> = config.template_extensions
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

    let walker = WalkDir::new(&source)
        .into_iter()
        .filter_entry(|e| {
            (ignore_filter(e, source, &config.ignore) || compare_paths(e.path(), &posts_path)) &&
            !compare_paths(e.path(), dest)
        })
        .filter_map(|e| e.ok());

    for entry in walker {
        let entry_path = entry.path();
        let extension = &entry_path.extension().unwrap_or_else(|| OsStr::new(""));
        if template_extensions.contains(extension) {
            // if the document is in the posts folder it's considered a post
            let is_post =
                entry_path.parent().map(|p| starts_with_path(p, &posts_path)).unwrap_or(false);

            let new_path = entry_path.strip_prefix(source).expect("Entry not in source folder");

            let doc = try!(Document::parse(&entry_path, new_path, is_post, &config.post_path));

            // Check if categories are in document
            if is_post == true {
                if let Some(categories) = doc.attributes.get("categories") {
                    if let &Value::Array(ref categories_array) = categories {
                        for category in categories_array {
                            if let &Value::Str(ref category_string) = category {
                                if !&config.categories_flat.contains(category_string) {
                                    info!("file {:?} category \"{}\" not found",
                                          entry_path,
                                          category_string);
                                }
                            }
                        }
                    }
                }
            }

            if !doc.is_draft || config.include_drafts {
                documents.push(doc);
            }
        }
    }

    if config.include_drafts {
        let drafts = source.join(&config.drafts);

        let walker = WalkDir::new(&drafts)
            .into_iter()
            .filter_entry(|e| {
                (ignore_filter(e, source, &config.ignore) || compare_paths(e.path(), &drafts)) &&
                !compare_paths(e.path(), dest)
            })
            .filter_map(|e| e.ok());

        for entry in walker {
            let entry_path = entry.path();
            let extension = &entry_path.extension().unwrap_or_else(|| OsStr::new(""));
            let new_path = posts_path
                .join(entry_path.strip_prefix(&drafts).expect("Draft not in draft folder!"));
            let new_path = new_path.strip_prefix(source).expect("Entry not in source folder");
            if template_extensions.contains(extension) {
                let doc = try!(Document::parse(&entry_path, new_path, true, &config.post_path));
                documents.push(doc);
            }
        }
    }

    // January 1, 1970 0:00:00 UTC, the beginning of time
    let default_date = UTC.timestamp(0, 0).with_timezone(&FixedOffset::east(0));

    let (mut posts, documents): (Vec<Document>, Vec<Document>) = documents.into_iter()
        .partition(|x| x.is_post);

    // sort documents by date, if there's no date (none was provided or it couldn't be read) then
    // fall back to the default date
    posts.sort_by(|a, b| b.date.unwrap_or(default_date).cmp(&a.date.unwrap_or(default_date)));

    // collect all posts attributes to pass them to other posts for rendering
    let simple_posts_data: Vec<Value> = posts.iter()
        .map(|x| Value::Object(x.attributes.clone()))
        .collect();

    // ------------------------------------------------------------------
    // name, description, link, excerpt_separator, categories
    let mut config_data = Vec::<Value>::new();
    let mut val1 = HashMap::<String, Value>::new();
    val1.insert("name".to_owned(),
                Value::Str(config.name.to_owned().unwrap_or("".to_string())));
    config_data.push(Value::Object(val1));

    let mut val2 = HashMap::<String, Value>::new();
    val2.insert("description".to_owned(),
                Value::Str(config.description.to_owned().unwrap_or("".to_string())));
    config_data.push(Value::Object(val2));

    let mut val3 = HashMap::<String, Value>::new();
    val3.insert("link".to_owned(),
                Value::Str(config.link.to_owned().unwrap_or("".to_string())));
    config_data.push(Value::Object(val3));

    let mut val4 = HashMap::<String, Value>::new();
    val4.insert("excerpt_separator".to_owned(),
                Value::Str(config.excerpt_separator.to_owned()));
    config_data.push(Value::Object(val4));

    // ------------------------------------------------------------------
    // categories
    let mut cat_vec: Vec<Value> = Vec::new();
    for (s, v) in config.categories.clone() {
        let mut subcat: Vec<Value> = Vec::new();
        for sc in v {
            let mut val8 = HashMap::<String, Value>::new();
            let posts_for_cat = get_posts_for_category(&sc, &simple_posts_data);
            val8.insert("name".to_owned(), Value::Str(sc));
            val8.insert("posts".to_owned(), Value::Array(posts_for_cat));
            subcat.push(Value::Object(val8));
        }
        let mut val6 = HashMap::<String, Value>::new();
        let posts_for_cat = get_posts_for_category(&s, &simple_posts_data);
        val6.insert("name".to_owned(), Value::Str(s));
        val6.insert("posts".to_owned(), Value::Array(posts_for_cat));
        cat_vec.push(Value::Object(val6));

        let mut val7 = HashMap::<String, Value>::new();
        val7.insert("subcategories".to_owned(), Value::Array(subcat));
        cat_vec.push(Value::Object(val7));
    }
    // push into config
    let mut val5 = HashMap::<String, Value>::new();
    val5.insert("categories".to_owned(), Value::Array(cat_vec));
    config_data.push(Value::Object(val5));

    /*println!("---- simple_posts_data --------------------------------------------");
    println!("{:?}", simple_posts_data);
    println!("---- config_data --------------------------------------------");
    println!("{:?}", config_data);
    println!("------------------------------------------------");*/

    trace!("Generating posts");
    for mut post in &mut posts {
        trace!("Generating {}", post.path);

        let mut context = post.get_render_context(&simple_posts_data, &config_data);

        try!(post.render_excerpt(&mut context, &source, &config.excerpt_separator));
        let post_html = try!(post.render(&mut context, &source, &layouts, &mut layouts_cache));
        try!(create_document_file(&post_html, &post.path, dest));
    }

    // check if we should create an RSS file and create it!
    if let Some(ref path) = config.rss {
        try!(create_rss(path, dest, &config, &posts));
    }

    // during post rendering additional attributes such as content were
    // added to posts. collect them so that non-post documents can access them
    let posts_data: Vec<Value> = posts.into_iter()
        .map(|x| Value::Object(x.attributes))
        .collect();

    trace!("Generating other documents");
    for mut doc in documents {
        trace!("Generating {}", doc.path);

        let mut context = doc.get_render_context(&posts_data, &config_data);
        let doc_html = try!(doc.render(&mut context, &source, &layouts, &mut layouts_cache));
        try!(create_document_file(&doc_html, &doc.path, dest));
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
                    .unwrap_or_else(|| OsStr::new(""))) &&
                !compare_paths(e.path(), dest)
            })
            .filter_map(|e| e.ok());

        for entry in walker {
            let entry_path = try!(entry.path()
                .to_str()
                .ok_or(format!("Cannot convert pathname {:?} to UTF-8", entry.path())));

            let relative = try!(entry_path.split(source_str)
                .last()
                .map(|s| s.trim_left_matches("/"))
                .ok_or("Empty path"));

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
fn create_rss(path: &str, dest: &Path, config: &Config, posts: &[Document]) -> Result<()> {
    match (&config.name, &config.description, &config.link) {
        // these three fields are mandatory in the RSS standard
        (&Some(ref name), &Some(ref description), &Some(ref link)) => {
            trace!("Generating RSS data");

            let items = posts.iter()
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
        try!(fs::create_dir_all(parent)
            .map_err(|e| format!("Could not create {:?}: {}", parent, e)));
    }

    let mut file = try!(File::create(&file_path)
        .map_err(|e| format!("Could not create {:?}: {}", file_path, e)));

    try!(file.write_all(&content.as_bytes()));
    info!("Created {}", file_path.display());
    Ok(())
}


fn get_posts_for_category(search_category: &str, posts: &Vec<Value>) -> Vec<Value> {
    //println!("Searching for {}", search_category);
    let mut catposts = Vec::<Value>::new();
    for post in posts {
        if let &Value::Object(ref obj) = post {
            if let Some(categories) = obj.get("categories") {
                if let &Value::Array(ref categories_array) = categories {
                    for category in categories_array {
                        if let &Value::Str(ref category_string) = category {
                            if category_string == search_category {
                                //println!("found {}", category_string);
                                catposts.push(post.clone());
                            }
                        }
                    }
                }

            }
        }
    }
    return catposts;
}

// The tests are taken from tests/fixtures/`posts_in_subfolder`/
#[test]
fn test_starts_with_path() {
    let posts_folder = Path::new("tests/fixtures/posts_in_subfolder/posts");

    assert!(!starts_with_path(Path::new("tests/fixtures/posts_in_subfolder"), posts_folder));
    assert!(starts_with_path(Path::new("tests/fixtures/posts_in_subfolder/posts"),
                             posts_folder));
    assert!(starts_with_path(Path::new("tests/fixtures/posts_in_subfolder/posts/20170103"),
                             posts_folder));
    assert!(starts_with_path(Path::new("tests/fixtures/posts_in_subfolder/posts/2017/01/08"),
                             posts_folder));
}
