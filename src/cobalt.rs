extern crate libc;

use std::io;
use std::io::fs;
use std::io::File;
use std::io::IoResult;
use std::path::Path;
use std::collections::HashMap;

use document::Document;
use util;

pub fn build(source: &Path, dest: &Path) -> IoResult<()>{
    let posts_source      = source.join("_posts");
    let posts_destination = dest.join("posts");

    let layouts = source.join("_layouts");

    // create target directories
    try!(fs::mkdir_recursive(&posts_destination, io::USER_RWX));

    // create index
    let index_source = source.join("index.tpl");
    let index = parse_document(&index_source);
    try!(index.create_file(dest, &layouts));

    // create posts
    let posts = parse_documents(&posts_source);
    for post in posts.iter() {
        try!(post.create_file(&posts_destination, &layouts));
    }

    // copy everything
    if source != dest {
        try!(util::copy_recursive_filter(source, dest, |p| -> bool {
            !p.filename_str().unwrap().starts_with(".")
            && p != dest
            && p != &index_source
            && p != &posts_source
            && p != &layouts
        }));
    }

    Ok(())
}

fn parse_documents(path: &Path) -> Vec<Document> {
    match fs::readdir(path) {
        Ok(paths) => paths.iter().filter_map( |path|
                if path.extension_str().unwrap() == "tpl" {
                    Some(parse_document(path))
                }else{
                    None
                }
            ).collect(),
        // TODO panic!
        Err(_) => panic!("Path {} doesn't exist\n", path.display())
    }
}

fn parse_document(path: &Path) -> Document {
    let attributes = extract_attributes(path);
    let content    = extract_content(path);

    Document::new(
        attributes,
        content,
        path.filestem_str().unwrap().to_string() + ".html",
    )
}

fn parse_file(path: &Path) -> String {
    match File::open(path) {
        // TODO handle IOResult
        Ok(mut x) => x.read_to_string().unwrap(),
        // TODO panic!
        Err(_) => panic!("File {} doesn't exist\n", path.display())
    }
}

fn extract_attributes(path: &Path) -> HashMap<String, String> {
    let mut attributes = HashMap::new();
    attributes.insert("name".to_string(), path.filestem_str().unwrap().to_string());

    let content = parse_file(path);

    if content.as_slice().contains("---") {
        let mut content_splits = content.as_slice().split_str("---");

        let attribute_string = content_splits.nth(0u).unwrap();

        for attribute_line in attribute_string.split_str("\n") {
            if !attribute_line.contains_char(':') {
                continue;
            }

            let mut attribute_split = attribute_line.split(':');

            // TODO: Refactor, find a better way for doing this
            // .nth() method is consuming the iterator and therefore the 0th index on the second method
            // is in real index 1
            let key   = attribute_split.nth(0u).unwrap().trim_chars(' ').to_string().clone();
            let value = attribute_split.nth(0u).unwrap().trim_chars(' ').to_string().clone();

            attributes.insert(key, value);
        }
    }

    return attributes;
}

fn extract_content(path: &Path) -> String {
    let content = parse_file(path);

    if content.as_slice().contains("---") {
        let mut content_splits = content.as_slice().split_str("---");

        return content_splits.nth(1u).unwrap().to_string();
    }

    return content;
}
