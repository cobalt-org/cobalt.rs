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
        let base_path      = &Path::new(path_string);
        let documents_path = base_path.as_str().unwrap().to_string() + "/_posts";
        let layout_path    = base_path.as_str().unwrap().to_string() + "/_layouts/default.tpl";
        let index_path     = base_path.as_str().unwrap().to_string() + "/index.tpl";
        let build_path     = base_path.as_str().unwrap().to_string() + "/build/";

        println!("Generating site in {}\n", path_string);

        let posts     = Runner::parse_documents(documents_path.as_slice());
        let layout    = Runner::parse_file(layout_path.as_slice());
        let index     = Runner::parse_index(index_path.as_slice());
        let post_path = Runner::create_dirs(build_path.as_slice());

        Runner::create_files(build_path.as_slice(), post_path.as_slice(), index, posts, layout.as_slice());
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

                documents.push(
                    Document::new(
                        attributes,
                        content,
                        path.filestem_str().unwrap().to_string() + ".html",
                    )
                );
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

    fn parse_index(index_path: &str) -> Document {
        let mut index_attr = HashMap::new();
        let path = Path::new(index_path);

        index_attr.insert("name".to_string(), "index".to_string());

        Document::new(
            index_attr,
            Runner::parse_file(index_path),
            path.filestem_str().unwrap().to_string() + ".html",
        )
    }

    fn create_dirs(build_path: &str) -> String {
        let postpath = (build_path.to_string() + "posts/");

        fs::mkdir(&Path::new(build_path), io::USER_RWX);
        fs::mkdir(&Path::new(postpath.as_slice()), io::USER_RWX);

        // TODO: copy non cobalt relevant folders into /build folder (assets, stylesheets, etc...)

        println!("Directory {} created\n", build_path);

        return postpath;
    }

    fn create_files(index_path: &str, document_path: &str, index: Document, documents: Vec<Document>, layout: &str) {
        // TODO: use different layout than for posts..
        index.create_file(layout, index_path);

        for document in documents.iter() {
            document.create_file(layout, document_path);
        }
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
