extern crate core;

use std::io;
use std::fs;
use std::fs::PathExt;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::collections::HashMap;
use self::core::str::StrExt;
use std::ffi::OsStr;
use liquid::Value;

use document::Document;
use util;

pub fn build(source: &Path, dest: &Path, layout_str: &str, posts_str: &str) -> io::Result<()>{
    // TODO make configurable
    let template_extensions = [OsStr::new("tpl"), OsStr::new("md")];

    let layouts_path = source.join(layout_str);
    let posts_path = source.join(posts_str);

    let mut layouts : HashMap<String, String> = HashMap::new();

    // go through the layout directory and add
    // filename -> text content to the layout map
    match fs::walk_dir(&layouts_path) {
        Ok(files) => for layout in files {
            let layout = try!(layout).path();
            if layout.is_file() {
                let mut text = String::new();
                try!(try!(File::open(&layout)).read_to_string(&mut text));
                layouts.insert(layout.as_path().file_name().unwrap().to_str().unwrap().to_string(), text);
            }
        },
        Err(_) => println!("Warning: No layout path found ({})\n", source.display())
    };

    let mut documents = vec![];
    let mut post_data = vec![];

    // walk source directory and find files that are written in
    // a template file extension
    for p in try!(fs::walk_dir(source)) {
        let p = p.unwrap().path();
        let path = p.as_path();
        // check for file extensions
        if template_extensions.contains(&path.extension().unwrap_or(OsStr::new("")))
        // check that file is not in the layouts folder
        && path.parent() != Some(layouts_path.as_path()) {
            let doc = parse_document(&path, source);
            if path.parent() == Some(posts_path.as_path()){
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
    let content    = extract_content(path).unwrap();
    let new_path   = path.relative_from(source).unwrap();
    // let markdown   = path.extension().unwrap_or(OsStr::new("")) == OsStr::new("md");

    Document::new(
        new_path.to_str().unwrap().to_string(),
        attributes,
        content,
        // markdown
    )
}

fn parse_file(path: &Path) -> io::Result<String> {
    let mut file = try!(File::open(path));
    let mut text = String::new();
    file.read_to_string(&mut text);
    Ok(text)
}

fn extract_attributes(path: &Path) -> HashMap<String, String> {
    let mut attributes = HashMap::new();
    attributes.insert("name".to_string(), path.file_stem().unwrap().to_str().unwrap().to_string());

    let content = parse_file(path).unwrap();

    if content.contains("---") {
        let mut content_splits = content.split("---");

        let attribute_string = content_splits.nth(0).unwrap();

        for attribute_line in attribute_string.split("\n") {
            if !attribute_line.contains_char(':') {
                continue;
            }

            let mut attribute_split = attribute_line.split(':');

            // TODO: Refactor, find a better way for doing this
            // .nth() method is consuming the iterator and therefore the 0th index on the second method
            // is in real index 1
            let key   = attribute_split.nth(0).unwrap().trim_matches(' ').to_string().clone();
            let value = attribute_split.nth(0).unwrap().trim_matches(' ').to_string().clone();

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
