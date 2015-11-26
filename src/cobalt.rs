use crossbeam;

use std::sync::Arc;
use std::fs::{self, File};
use std::io::{Read};
use std::path::Path;
use std::collections::HashMap;
use std::ffi::OsStr;
use liquid::Value;
use walkdir::WalkDir;
use document::Document;
use error::Result;

/// The primary build function that tranforms a directory into a site
pub fn build(source: &Path, dest: &Path, layout_str: &str, posts_str: &str) -> Result<()> {
    // TODO make configurable
    let template_extensions = [OsStr::new("tpl"), OsStr::new("md")];

    let layouts_path = source.join(layout_str);
    let posts_path = source.join(posts_str);

    let layouts = try!(get_layouts(&layouts_path));

    let mut documents = vec![];
    let mut post_data = vec![];

    let walker = WalkDir::new(&source).into_iter();

    for entry in walker.filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
        if template_extensions.contains(&entry.path()
                                              .extension()
                                              .unwrap_or(OsStr::new(""))) &&
           entry.path().parent() != Some(layouts_path.as_path()) {
            let doc = try!(parse_document(&entry.path(), source));
            if entry.path().parent() == Some(posts_path.as_path()) {
                post_data.push(Value::Object(doc.get_attributes()));
            }
            documents.push(doc);
        }
    }

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
        let walker = WalkDir::new(&source)
                         .into_iter()
                         .filter_map(|e| e.ok())
                         .filter(|f| {
                             let p = f.path();
                             // don't copy hidden files
                             !p.file_name()
                               .expect(&format!("No file name for {:?}", p))
                               .to_str()
                               .unwrap_or("")
                               .starts_with(".") &&
                             !template_extensions.contains(&p.extension()
                                                             .unwrap_or(OsStr::new(""))) &&
                             p != dest && p != layouts_path.as_path()
                         });

        for entry in walker {
            let relative = entry.path()
                                .to_str()
                                .expect(&format!("Invalid UTF-8 in {:?}", entry))
                                .split(source.to_str()
                                             .expect(&format!("Invalid UTF-8 in {:?}", source)))
                                .last()
                                .expect(&format!("Empty path"));

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

    let walker = WalkDir::new(layouts_path).into_iter();

    // go through the layout directory and add
    // filename -> text content to the layout map
    for entry in walker.filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
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
    let attributes = try!(extract_attributes(path));
    let content = try!(extract_content(path));

    let new_path = path.to_str()
                       .expect(&format!("Invalid UTF-8 in {:?}", path))
                       .split(source.to_str()
                                    .expect(&format!("Invalid UTF-8 in {:?}", source)))
                       .last()
                       .expect(&format!("Empty path"));
    let markdown = path.extension().unwrap_or(OsStr::new("")) == OsStr::new("md");

    Ok(Document::new(new_path.to_owned(), attributes, content, markdown))
}

fn parse_file(path: &Path) -> Result<String> {
    let mut file = try!(File::open(path));
    let mut text = String::new();
    try!(file.read_to_string(&mut text));
    Ok(text)
}

fn extract_attributes(path: &Path) -> Result<HashMap<String, String>> {
    let mut attributes = HashMap::new();
    attributes.insert("name".to_owned(),
                      path.file_stem()
                          .expect(&format!("No file stem for {:?}", path))
                          .to_str()
                          .expect(&format!("Invalid UTF-8 in file stem for {:?}", path))
                          .to_owned());

    let content = parse_file(path).expect(&format!("Failed to parse {:?}", path));

    if content.contains("---") {
        let mut content_splits = content.split("---");

        let attribute_string = try!(content_splits.nth(0).ok_or("Empty content"));

        for attribute_line in attribute_string.split("\n") {
            if !attribute_line.contains(':') {
                continue;
            }

            let attribute_split: Vec<&str> = attribute_line.split(':').collect();

            let key = attribute_split[0].trim_matches(' ').to_owned();
            let value = attribute_split[1].trim_matches(' ').to_owned();

            attributes.insert(key, value);
        }
    }

    Ok(attributes)
}

fn extract_content(path: &Path) -> Result<String> {
    let content = try!(parse_file(path));

    if content.contains("---") {
        let mut content_splits = content.split("---");

        return Ok(try!(content_splits.nth(1).ok_or("No content after header")).to_owned());
    }

    return Ok(content);
}
