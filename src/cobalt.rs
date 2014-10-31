extern crate libc;

use std::io;
use std::io::fs;
use std::io::File;
use std::path::Path;
use std::collections::HashMap;

use document::Document;

pub struct Runner;

impl Runner {
    pub fn run(path_string: &str) {
        let base_path      = Path::new(path_string);
        let documents_path = base_path.join("_posts");
        let layout_path    = base_path.join("_layouts");
        let index_path     = base_path.join("index.tpl");
        let build_path     = base_path.join("build");

        println!("Generating site in {}\n", path_string);

        let index     = Runner::parse_document(&index_path);
        let posts     = Runner::parse_documents(&documents_path);
        let post_path = Runner::create_dirs(&build_path);

        Runner::create_files(&build_path, &post_path, &layout_path, index, posts);
    }

    fn parse_documents(path: &Path) -> Vec<Document> {
        match fs::readdir(path) {
            Ok(paths) => paths.iter().filter_map( |path|
                    if path.extension_str().unwrap() == "tpl" {
                        Some(Runner::parse_document(path))
                    }else{
                        None
                    }
                ).collect(),
            // TODO panic!
            Err(_) => fail!("Path {} doesn't exist\n", path.display())
        }
    }

    fn parse_document(path: &Path) -> Document {
        let attributes = Runner::extract_attributes(path);
        let content    = Runner::extract_content(path);

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
            Err(_) => fail!("File {} doesn't exist\n", path.display())
        }
    }

    fn create_dirs(build_path: &Path) -> Path {
        let postpath = build_path.join("posts");

        fs::mkdir(build_path, io::USER_RWX);
        fs::mkdir(&postpath, io::USER_RWX);

        // TODO: copy non cobalt relevant folders into /build folder (assets, stylesheets, etc...)

        println!("Directory {} created\n", build_path.display());

        return postpath;
    }

    fn create_files(index_path: &Path, document_path: &Path, layout_path: &Path, index: Document, documents: Vec<Document>) {
        index.create_file(index_path, layout_path);

        for document in documents.iter() {
            document.create_file(document_path, layout_path);
        }
    }


    fn extract_attributes(path: &Path) -> HashMap<String, String> {
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), path.filestem_str().unwrap().to_string());

        let content = Runner::parse_file(path);

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
        let content = Runner::parse_file(path);

        if content.as_slice().contains("---") {
            let mut content_splits = content.as_slice().split_str("---");

            return content_splits.nth(1u).unwrap().to_string();
        }

        return content;
    }
}
