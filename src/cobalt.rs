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
        let base_path      = os::make_absolute(&Path::new(path_string));
        let documents_path = os::make_absolute(&Path::new((path_string.to_string() + "/_posts").as_slice()));
        let layout_path    = os::make_absolute(&Path::new((path_string.to_string() + "/_layouts/default.tpl").as_slice()));
        let index_path     = os::make_absolute(&Path::new(path_string.to_string() + "/index.tpl"));
        let build_path     = os::make_absolute(&Path::new(path_string.to_string() + "/build"));
        let post_path      = os::make_absolute(&Path::new(path_string.to_string() + "/build/posts"));

        println!("Generating site in {}\n", base_path.as_str().unwrap());

        let mut documents = Runner::parse_documents(documents_path);
        let layout    = Runner::parse_file(layout_path);

        let mut index_attr = HashMap::new();
        index_attr.insert("name".to_string(), "index".to_string());

        let index     = Document::new(
            index_attr,
            Runner::parse_file(index_path)
        );

        documents.insert(0, index);

        Runner::create_build(build_path, post_path, documents, layout);
    }

    fn parse_documents(documents_path: Path) -> Vec<Document> {
        let paths = fs::readdir(&documents_path);
        let mut documents = vec!();

        if paths.is_ok() {
            for path in paths.unwrap().iter() {
                if path.extension_str().unwrap() != "tpl" {
                    continue;
                }

                let attributes = Runner::extract_attributes(path.clone());
                let content    = Runner::extract_content(path.clone());

                documents.push(Document::new(attributes, content));
            }
        } else {
            println!("Path {} doesn't exist\n", documents_path.as_str().unwrap());
            unsafe { libc::exit(1 as libc::c_int); }
        }

        return documents;
    }

    fn parse_file(file_path: Path) -> String {
        if File::open(&file_path).is_ok() {
            return File::open(&file_path).read_to_string().unwrap();
        } else {
            println!("File {} doesn't exist\n", file_path.as_str().unwrap());
            unsafe { libc::exit(1 as libc::c_int); }
        }
    }

    fn create_build(build_path: Path, post_path: Path, documents: Vec<Document>, layout: String) {
        fs::mkdir(&build_path, io::USER_RWX);
        fs::mkdir(&post_path, io::USER_RWX);

        for document in documents.iter() {
            document.create_file(build_path.clone(), post_path.clone(), layout.clone());
        }

        println!("Directory {} created", build_path.as_str().unwrap());
    }


    fn extract_attributes(document_path: Path) -> HashMap<String, String> {
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), document_path.filestem_str().unwrap().to_string());

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

    fn extract_content(document_path: Path) -> String {
        let content = Runner::parse_file(document_path);

        let mut content_splits = content.as_slice().split_str("---");

        content_splits.nth(1u).unwrap().to_string()
    }
}
