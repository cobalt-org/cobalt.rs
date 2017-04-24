use std::fs::{self, File};
use std::collections::{HashMap, BTreeMap};
use std::io::Write;
use std::path::{self, Path};
use std::ffi::OsStr;
use liquid::Value;
use walkdir::{WalkDir, DirEntry, WalkDirIterator};
use document::Document;
use error::{ErrorKind, Result};
use config::{Config, Dump};
use chrono::{UTC, FixedOffset};
use chrono::offset::TimeZone;
use rss::{Channel, Rss};
use glob::Pattern;
use toml;
use liquid;

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

fn convert_value(liquid_value: &liquid::Value) -> Result<toml::Value> {
    match *liquid_value {
        liquid::Value::Str(ref s) => Ok(toml::Value::String(s.to_string())),
        liquid::Value::Num(n) => Ok(toml::Value::Float(n as f64)),
        liquid::Value::Bool(b) => Ok(toml::Value::Boolean(b)),
        liquid::Value::Array(ref a) => {
            let toml_array: Result<Vec<toml::Value>> = a.iter().map(convert_value).collect();
            let toml_array = toml_array?;
            Ok(toml::Value::Array(toml_array))
        }
        liquid::Value::Object(ref t) => {
            let toml_object: Result<BTreeMap<String, toml::Value>> = t.iter()
                .map(|(k, v)| convert_value(v).map(|v| (k.to_string(), v)))
                .collect();
            let toml_object = toml_object?;
            Ok(toml::Value::Table(toml_object))
        }
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

            let doc = try!(Document::parse(entry_path, new_path, is_post, &config.post_path));
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
                let doc = try!(Document::parse(entry_path, new_path, true, &config.post_path));
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

    trace!("Generating posts");
    for mut post in &mut posts {
        trace!("Generating {}", post.path);

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
    let posts_data: Vec<Value> = posts.into_iter()
        .map(|x| Value::Object(x.attributes))
        .collect();

    trace!("Generating other documents");
    for mut doc in documents {
        trace!("Generating {}", doc.path);

        if config.dump.contains(&Dump::Liquid) {
            create_liquid_dump(dest, &doc.path, &doc.content, &doc.attributes)?;
        }

        let mut context = doc.get_render_context(&posts_data);
        let doc_html = try!(doc.render(&mut context, source, &layouts, &mut layouts_cache));
        try!(create_document_file(&doc_html, &doc.path, dest));
    }

    // copy all remaining files in the source to the destination
    if !compare_paths(source, dest) {
        info!("Copying remaining assets");
        let source_str = try!(source.to_str()
            .ok_or_else(|| format!("Cannot convert pathname {:?} to UTF-8", source)));

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
                .ok_or_else(|| format!("Cannot convert pathname {:?} to UTF-8", entry.path())));

            let relative = if source_str == "." {
                entry_path
            } else {
                try!(entry_path.split(source_str)
                    .last()
                    .map(|s| s.trim_left_matches(path::MAIN_SEPARATOR))
                    .ok_or("Empty path"))
            };

            if try!(entry.metadata()).is_dir() {
                try!(fs::create_dir_all(&dest.join(relative)));
                debug!("Created new directory {:?}", dest.join(relative));
            } else {
                if let Some(parent) = Path::new(relative).parent() {
                    try!(fs::create_dir_all(&dest.join(parent)));
                }

                try!(fs::copy(entry.path(), &dest.join(relative)).map_err(|e| {
                    format!("Could not copy {:?} into {:?}: {}",
                            entry.path(),
                            dest.join(relative),
                            e)
                }));
                debug!("Copied {:?} to {:?}", entry.path(), dest.join(relative));
            }
        }
    }

    Ok(())
}

fn create_liquid_dump(dest: &Path,
                      path: &str,
                      content: &str,
                      attributes: &HashMap<String, Value>)
                      -> Result<()> {
    let mut liquid_file_path = dest.join(path);
    let mut liquid_file_name = OsStr::new(" ").to_os_string();
    {
        let original_file_name = liquid_file_path.file_name().ok_or("File name missing")?;
        liquid_file_name.push(original_file_name);
        liquid_file_name.push(".liquid");
    }
    liquid_file_path.set_file_name(liquid_file_name);

    let mut toml_file_path = liquid_file_path.clone();
    toml_file_path.set_extension(".toml");

    let mut liquid_out = fs::File::create(liquid_file_path)?;
    liquid_out.write_all(content.as_bytes())?;

    let mut toml_out = fs::File::create(toml_file_path)?;
    let values = convert_value(&liquid::Value::Object(attributes.clone()))?;
    let values = values.to_string();
    toml_out.write_all(values.as_bytes())?;

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
        try!(fs::create_dir_all(parent)
            .map_err(|e| format!("Could not create {:?}: {}", parent, e)));
    }

    let mut file = try!(File::create(&file_path)
        .map_err(|e| format!("Could not create {:?}: {}", file_path, e)));

    try!(file.write_all(content.as_bytes()));
    info!("Created {}", file_path.display());
    Ok(())
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
