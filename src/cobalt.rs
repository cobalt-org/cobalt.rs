use crossbeam;

use std::sync::Arc;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::collections::HashMap;
use std::ffi::OsStr;
use liquid::Value;
use walkdir::WalkDir;
use document::Document;
use error::Result;
use yaml_rust::YamlLoader;
use chrono::{DateTime, UTC, FixedOffset};
use chrono::offset::TimeZone;

macro_rules! walker {
    ($dir:expr) => {
        WalkDir::new($dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|f| {
                // skip directories
                f.file_type().is_file()
                &&
                // don't copy hidden files
                !f.path()
                  .file_name()
                  .and_then(|name| name.to_str())
                  .unwrap_or(".")
                  .starts_with(".")
            })
    }
}

/// The primary build function that tranforms a directory into a site
pub fn build(source: &Path, dest: &Path, layout_str: &str, posts_str: &str) -> Result<()> {
    // TODO make configurable
    let template_extensions = [OsStr::new("tpl"), OsStr::new("md")];

    let layouts_path = source.join(layout_str);
    let posts_path = source.join(posts_str);

    let layouts = try!(get_layouts(&layouts_path));

    let mut documents = vec![];

    for entry in walker!(&source) {
        if template_extensions.contains(&entry.path()
                                              .extension()
                                              .unwrap_or(OsStr::new(""))) &&
           entry.path().parent() != Some(layouts_path.as_path()) {
            let mut doc = try!(parse_document(&entry.path(), source));

            // if the document is in the posts folder it's considered a post
            if entry.path().parent() == Some(posts_path.as_path()) {
                doc.is_post = true;
            }

            documents.push(doc);
        }
    }

    // January 1, 1970 0:00:00 UTC, the beginning of time
    let default_date = UTC.timestamp(0, 0).with_timezone(&FixedOffset::east(0));

    // sort documents by date, if there's no date (none was provided or it couldn't be read) then
    // fall back to the default date
    documents.sort_by(|a, b| {
        b.date.unwrap_or(default_date.clone()).cmp(&a.date.unwrap_or(default_date.clone()))
    });

    // these are the attributes of all documents that are posts, so that they can be
    // passed to the renderer
    let post_data: Vec<Value> = documents.iter()
                                         .filter(|x| x.is_post)
                                         .map(|x| Value::Object(x.get_attributes()))
                                         .collect();

    // thread handles to join later
    let mut handles = vec![];

    // generate documents (in parallel)
    // TODO I'm probably underutilizing crossbeam
    crossbeam::scope(|scope| {
        let post_data = Arc::new(post_data);
        let layouts = Arc::new(layouts);
        for doc in &documents {
            let post_data = post_data.clone();
            let layouts = layouts.clone();
            let handle = scope.spawn(move || doc.create_file(dest, &layouts, &post_data));
            handles.push(handle);
        }
    });

    for handle in handles {
        try!(handle.join());
    }

    // copy all remaining files in the source to the destination
    if source != dest {
        let source_str = try!(source.to_str()
                                    .ok_or(format!("Cannot convert pathname {:?} to UTF-8",
                                                   source)));

        for entry in walker!(&source).filter(|f| {
            !template_extensions.contains(&f.path()
                                            .extension()
                                            .unwrap_or(OsStr::new(""))) &&
            f.path() != dest && f.path() != layouts_path.as_path()
        }) {
            let entry_path = try!(entry.path()
                                       .to_str()
                                       .ok_or(format!("Cannot convert pathname {:?} to UTF-8",
                                                      entry.path())));

            let relative = try!(entry_path.split(source_str)
                                          .last()
                                          .ok_or(format!("Empty path")));

            if try!(entry.metadata()).is_dir() {
                try!(fs::create_dir_all(&dest.join(relative)));
            } else {
                try!(fs::copy(entry.path(), &dest.join(relative)));
            }
        }
    }

    Ok(())
}

/// Gets all layout files from the specified path (usually _layouts/)
/// This walks the specified directory recursively
///
/// Returns a map filename -> content
fn get_layouts(layouts_path: &Path) -> Result<HashMap<String, String>> {
    let mut layouts = HashMap::new();

    // go through the layout directory and add
    // filename -> text content to the layout map
    for entry in walker!(layouts_path) {
        let mut text = String::new();
        let mut file = try!(File::open(entry.path()));
        try!(file.read_to_string(&mut text));

        let path = try!(entry.path()
                             .file_name()
                             .and_then(|name| name.to_str())
                             .ok_or(format!("Cannot convert pathname {:?} to UTF-8",
                                            entry.path().file_name())));

        layouts.insert(path.to_owned(), text);
    }

    Ok(layouts)
}

fn parse_document(path: &Path, source: &Path) -> Result<Document> {
    let mut attributes = HashMap::new();
    attributes.insert("name".to_owned(),
                      try!(path.file_stem()
                               .and_then(|stem| stem.to_str())
                               .ok_or(format!("Invalid UTF-8 in file stem for {:?}", path)))
                          .to_owned());

    let mut content = try!(parse_file(path));

    // if there is front matter, split the file and parse it
    if content.contains("---") {
        let content2 = content.clone();
        let mut content_splits = content2.split("---");

        // above the split are the attributes
        let attribute_string = content_splits.next().unwrap_or("");

        // everything below the split becomes the new content
        content = content_splits.next().unwrap_or("").to_owned();

        let yaml_result = try!(YamlLoader::load_from_str(attribute_string));

        let yaml_attributes = try!(yaml_result[0]
                                       .as_hash()
                                       .ok_or(format!("Incorrect front matter format in {:?}",
                                                      path)));

        for (key, value) in yaml_attributes {
            // TODO is unwrap_or the best way to handle this?
            attributes.insert(key.as_str().unwrap_or("").to_owned(),
                              value.as_str().unwrap_or("").to_owned());
        }
    }

    let date = attributes.get("date")
                         .and_then(|d| DateTime::parse_from_str(d, "%d %B %Y %H:%M:%S %z").ok());

    let path_str = try!(path.to_str()
                            .ok_or(format!("Cannot convert pathname {:?} to UTF-8", path)));

    let source_str = try!(source.to_str()
                                .ok_or(format!("Cannot convert pathname {:?} to UTF-8", source)));

    let new_path = try!(path_str.split(source_str)
                                .last()
                                .ok_or(format!("Empty path")));

    let markdown = path.extension().unwrap_or(OsStr::new("")) == OsStr::new("md");

    Ok(Document::new(new_path.to_owned(),
                     attributes,
                     content,
                     false,
                     date,
                     markdown))
}

fn parse_file(path: &Path) -> Result<String> {
    let mut file = try!(File::open(path));
    let mut text = String::new();
    try!(file.read_to_string(&mut text));
    Ok(text)
}
