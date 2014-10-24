use std::os;
use std::io::fs;
use std::io::File;
use std::path::Path;

use document::Document;

pub struct Runner;

impl Runner {
    pub fn run(path_string: &str) {
        let base_path = os::make_absolute(&Path::new(path_string));
        let document_path = os::make_absolute(&Path::new((path_string.to_string() + "/_posts").as_slice()));

        println!("Generating site in {}", base_path.as_str().unwrap());

        let documents = Runner::parse_documents(document_path);
    }

    fn parse_documents(document_path: Path) {
        let paths = fs::readdir(&document_path);
        let mut document_vec = vec!();

        if paths.is_ok() {
            for path in paths.unwrap().iter() {
                document_vec.push(Document::new(path));
            }
        } else {
            println!("Path {} doesn't exist", document_path.as_str().unwrap());
        }
    }
}
