extern crate libc;

use std::io::fs;
use std::io::fs::PathExtensions;
use std::io::File;
use std::io::IoResult;
use std::path::Path;
use std::collections::HashMap;

use document::Document;
use util;

pub fn build(source: &Path, dest: &Path) -> IoResult<()>{
    // TODO make configurable
    let template_extensions = ["tpl", "md"];

    let layouts_path = source.join("_layouts");
    let mut layouts = HashMap::new();

    match fs::walk_dir(&layouts_path) {
        Ok(mut files) => for layout in files {
            if(layout.is_file()){
                let text = File::open(&layout).read_to_string().unwrap();
                layouts.insert(layout.filename_str().unwrap().to_string(), text);
            }
        },
        Err(_) => println!("Warning: No layout path found ({})\n", source.display())
    };

    // create posts
    let posts : Vec<Document> = match fs::walk_dir(source) {
        Ok(directories) => directories.filter_map(|p|
                if template_extensions.contains(&p.extension_str().unwrap_or(""))
                && p.dir_path() != layouts_path {
                    Some(parse_document(&p, source))
                }else{
                    None
                }
            ).collect(),
        Err(_) => panic!("Path {} doesn't exist\n", source.display())
    };

    for post in posts.iter() {
        try!(post.create_file(dest, &layouts));
    }

    // copy everything
    if source != dest {
        try!(util::copy_recursive_filter(source, dest, |p| -> bool {
            !p.filename_str().unwrap().starts_with(".")
            && !template_extensions.contains(&p.extension_str().unwrap_or(""))
            && p != dest
            && p != &layouts_path
        }));
    }

    Ok(())
}

fn parse_document(path: &Path, source: &Path) -> Document {
    let attributes   = extract_attributes(path);
    let content      = extract_content(path);
    let mut new_path = path.path_relative_from(source).unwrap();
    new_path.set_extension("html");

    Document::new(
        attributes,
        content,
        new_path
    )
}

fn parse_file(path: &Path) -> String {
    match File::open(path) {
        // TODO handle IOResult
        Ok(mut x) => x.read_to_string().unwrap(),
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
