extern crate libc;

use std::os;
use std::io;
use std::io::fs;
use std::io::File;
use std::path::Path;
use std::collections::HashMap;

use document::Document;

pub struct Runner;

impl Runner {
    pub fn run(path_string: &str) {
        let documents_path = path_string.to_string() + "/_posts";
        let layout_path    = path_string.to_string() + "/_layouts/default.tpl";
        let index_path     = path_string.to_string() + "/index.tpl";
        let build_path     = path_string.to_string() + "/build";
        let post_path      = path_string.to_string() + "/build/posts";

        println!("Generating site in {}\n", path_string);

        let mut posts = Runner::parse_documents(documents_path.as_slice());
        let layout    = Runner::parse_file(layout_path.as_slice());

        let mut documents = Runner::parse_index(index_path.as_slice(), posts);
        Runner::create_build(build_path.as_slice(), post_path.as_slice(), documents, layout);
    }

    fn parse_documents(documents_path: &str) -> Vec<Document> {
        let path = &Path::new(documents_path);

        let paths = fs::readdir(path);
        let mut documents = vec!();

        if paths.is_ok() {
            for path in paths.unwrap().iter() {
                if path.extension_str().unwrap() != "tpl" {
                    continue;
                }

                let attributes = Runner::extract_attributes(path.as_str().unwrap());
                let content    = Runner::extract_content(path.as_str().unwrap());

                documents.push(Document::new(attributes, content));
            }
        } else {
            println!("Path {} doesn't exist\n", documents_path);
            unsafe { libc::exit(1 as libc::c_int); }
        }

        return documents;
    }

    fn parse_file(file_path: &str) -> String {
        let path = &Path::new(file_path);

        if File::open(path).is_ok() {
            return File::open(path).read_to_string().unwrap();
        } else {
            println!("File {} doesn't exist\n", file_path);
            unsafe { libc::exit(1 as libc::c_int); }
        }
    }

    fn parse_index(index_path: &str, posts: Vec<Document>) -> Vec<Document> {
        let mut index_attr = HashMap::new();
        index_attr.insert("name".to_string(), "index".to_string());

        let index     = Document::new(
            index_attr,
            Runner::parse_file(index_path)
        );

        let mut documents = posts;
        documents.insert(0, index);

        println!("{}", documents);

        return documents;
    }

    fn create_build(build_path: &str, post_path: &str, documents: Vec<Document>, layout: String) {
        fs::mkdir(&Path::new(build_path), io::USER_RWX);
        fs::mkdir(&Path::new(post_path), io::USER_RWX);

        for document in documents.iter() {
            document.create_file(build_path, post_path, layout.clone());
        }

        println!("Directory {} created", build_path);
    }


    fn extract_attributes(document_path: &str) -> HashMap<String, String> {
        let path = Path::new(document_path);
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), path.filestem_str().unwrap().to_string());

        let content = Runner::parse_file(document_path);

        let mut content_splits = content.as_slice().split_str("---");

        let attribute_string = content_splits.nth(0u).unwrap();

        for attribute_line in attribute_string.split_str("\n") {
            if !attribute_line.contains_char(':') {
                continue;
            }

            let mut attribute_split = attribute_line.split(':');

            let key   = attribute_split.nth(0u).unwrap().trim_chars(' ').to_string().clone();
            let value = attribute_split.nth(0u).unwrap().trim_chars(' ').to_string().clone();

            attributes.insert(key, value);
        }

        return attributes;
    }

    fn extract_content(document_path: &str) -> String {
        let content = Runner::parse_file(document_path);

        let mut content_splits = content.as_slice().split_str("---");

        content_splits.nth(1u).unwrap().to_string()
    }
}
