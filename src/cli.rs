use std::os;
use std::io::fs;
use std::io::File;
use std::path::Path;

use document::Document;

pub struct Runner;

impl Runner {
    pub fn run(path_string: &str) {
        let base_path      = os::make_absolute(&Path::new(path_string));
        let documents_path = os::make_absolute(&Path::new((path_string.to_string() + "/_posts").as_slice()));

        println!("Generating site in {}", base_path.as_str().unwrap());

        let documents = Runner::parse_documents(documents_path);
    }

    fn parse_documents(documents_path: Path) -> Vec<Document> {
        let paths = fs::readdir(&documents_path);
        let mut documents = vec!();

        if paths.is_ok() {
            for path in paths.unwrap().iter() {
                if path.extension_str().unwrap() != "tpl" {
                    continue;
                }

                let attributes = Runner::extract_attributes(path);
                let content    = Runner::extract_content(path);

                documents.push(Document::new(attributes, content));
            }
        } else {
            println!("Path {} doesn't exist", documents_path.as_str().unwrap());
        }

        /*for document in documents.iter() {
            println!("{}", document.as_html());
        }*/

        return documents;
    }

    fn extract_attributes(document_path: &Path) -> Vec<(String, String)> {
        let mut attributes = vec!();
        attributes.push(("name".to_string(), document_path.filestem_str().unwrap().to_string()));

        let content = File::open(document_path).read_to_string().unwrap();

        let mut content_splits = content.as_slice().split_str("---");

        let attribute_string = content_splits.nth(0u).unwrap();

        for attribute_line in attribute_string.split_str("\n") {
            if !attribute_line.contains_char(':') {
                continue;
            }

            let mut attribute_split = attribute_line.split(':');

            let key   = attribute_split.nth(0u).unwrap().trim_chars(' ').to_string();
            let value = attribute_split.nth(0u).unwrap().trim_chars(' ').to_string();

            attributes.push((key, value));
        }

        return attributes;
    }

    fn extract_content(document_path: &Path) -> String {
        let content = File::open(document_path).read_to_string().unwrap();

        let mut content_splits = content.as_slice().split_str("---");

        content_splits.nth(1u).unwrap().to_string()
    }
}
