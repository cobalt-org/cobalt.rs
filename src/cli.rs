use std::os;
use std::io::fs;
use std::io::File;
use std::path::Path;

use document::Document;

pub struct Runner;

impl Runner {
    pub fn run(path_string: &str) {
        let base_path = os::make_absolute(&Path::new(path_string));
        let documents_path = os::make_absolute(&Path::new((path_string.to_string() + "/_posts").as_slice()));

        println!("Generating site in {}", base_path.as_str().unwrap());

        let documents = Runner::parse_documents(documents_path);
    }

    fn parse_documents(documents_path: Path) -> Vec<Document> {
        let paths = fs::readdir(&documents_path);
        let mut documents = vec!();

        if paths.is_ok() {
            for path in paths.unwrap().iter() {
                let attributes   = Runner::extract_attributes(path);
                let content      = Runner::extract_content(path);

                for attribute in attributes.iter() {
                    println!("{}", attribute);
                }

                documents.push(Document::new(attributes, content));
            }
        } else {
            println!("Path {} doesn't exist", documents_path.as_str().unwrap());
        }

        return documents;
    }

    fn extract_attributes(document_path: &Path) -> Vec<(String, String)> {
        let content = File::open(document_path).read_to_string().unwrap();

        // TODO: Regex matching for retrieving the attributes

        vec![
            ("Test Key".to_string(), "Test Value".to_string()),
            ("Test Key2".to_string(), "Test Value2".to_string())
        ]
    }

    fn extract_content(document_path: &Path) -> String {
        let content = File::open(document_path).read_to_string().unwrap();

        // TODO: Regex matching for retrieving the content

        return "Test".to_string();
    }
}
