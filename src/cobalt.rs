use std::io;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::collections::HashMap;
use std::ffi::OsStr;
use liquid::Value;

use walkdir::WalkDir;

use document::Document;
use util;

pub fn build(source: &Path, dest: &Path, layout_str: &str, posts_str: &str) -> io::Result<()> {
    // TODO make configurable
    let template_extensions = [OsStr::new("tpl"), OsStr::new("md")];

    let layouts_path = source.join(layout_str);
    let posts_path = source.join(posts_str);

    let mut layouts: HashMap<String, String> = HashMap::new();

    let walker = WalkDir::new(&layouts_path).into_iter();

    // go through the layout directory and add
    // filename -> text content to the layout map
    for entry in walker.filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
        let mut text = String::new();
        File::open(entry.path()).unwrap().read_to_string(&mut text);
        layouts.insert(entry.path()
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string(),
                       text);
    }

    let mut documents = vec![];
    let mut post_data = vec![];

    let walker = WalkDir::new(&source).into_iter();

    for entry in walker.filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
        if template_extensions.contains(&entry.path()
                                              .extension()
                                              .unwrap_or(OsStr::new(""))) &&
           entry.path().parent() != Some(layouts_path.as_path()) {
            let doc = parse_document(&entry.path(), source);
            if entry.path().parent() == Some(posts_path.as_path()) {
                post_data.push(Value::Object(doc.get_attributes()));
            }
            documents.push(doc);
        }
    }

    for doc in documents.iter() {
        try!(doc.create_file(dest, &layouts, &post_data));
    }

    // copy everything
    if source != dest {
        try!(util::copy_recursive_filter(source, dest, &|p| -> bool {
            !p.file_name().unwrap().to_str().unwrap_or("").starts_with(".")
                && !template_extensions.contains(&p.extension().unwrap_or(OsStr::new("")))
                && p != dest
                && p != layouts_path.as_path()
        }));
    }

    Ok(())
}

fn parse_document(path: &Path, source: &Path) -> Document {
    let attributes = extract_attributes(path);
    let content = extract_content(path).unwrap();
    let new_path = path.to_str().unwrap().split(source.to_str().unwrap()).last().unwrap();
    let markdown = path.extension().unwrap_or(OsStr::new("")) == OsStr::new("md");

    Document::new(new_path.to_string(), attributes, content, markdown)
}

fn parse_file(path: &Path) -> io::Result<String> {
    let mut file = try!(File::open(path));
    let mut text = String::new();
    try!(file.read_to_string(&mut text));
    Ok(text)
}

fn extract_attributes(path: &Path) -> HashMap<String, String> {
    let mut attributes = HashMap::new();
    attributes.insert("name".to_string(),
                      path.file_stem().unwrap().to_str().unwrap().to_string());

    let content = parse_file(path).unwrap();

    if content.contains("---") {
        let mut content_splits = content.split("---");

        let attribute_string = content_splits.nth(0).unwrap();

        for attribute_line in attribute_string.split("\n") {
            if !attribute_line.contains(':') {
                continue;
            }

            let attribute_split: Vec<&str> = attribute_line.split(':').collect();

            let key = attribute_split[0].trim_matches(' ').to_string();
            let value = attribute_split[1].trim_matches(' ').to_string();

            attributes.insert(key, value);
        }
    }

    return attributes;
}

fn extract_content(path: &Path) -> io::Result<String> {
    let content = try!(parse_file(path));

    if content.contains("---") {
        let mut content_splits = content.split("---");

        return Ok(content_splits.nth(1).unwrap().to_string());
    }

    return Ok(content);
}
