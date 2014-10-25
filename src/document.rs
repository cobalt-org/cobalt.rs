extern crate mustache;

use std::fmt;
use std::io;
use std::str;
use std::collections::HashMap;
use std::io::File;

pub struct Document {
    attributes: HashMap<String, String>,
    content: String,
}

impl Document {
    pub fn new(attributes: HashMap<String, String>, content: String) -> Document {
        Document {
            attributes: attributes,
            content: content,
        }
    }

    pub fn as_html(&self) -> String {
        let template = mustache::compile_str(self.content.as_slice());

        /* let the brainfuck start here */
        // a Writer which impl Writer is needed here (could be file, could be socket or any other writer)
        // StringWriter doesn't exist yet, therefore I have to use a MemWriter
        // and convert the u8 vec sequence into ascii so its convertable to u8 again
        let mut w = io::MemWriter::new();

        // why do I have to say &mut here
        // mutable reference?!?!
        template.render(&mut w, &self.attributes);

        w.unwrap().into_ascii().into_string()
        /* and end it here, I don't know whats going on here... WAT */
    }

    pub fn create_file(&self, build_path: Path, document_path: Path, layout: String) {
        if self.attributes.get_copy(&"name".to_string()) == "index".to_string() {
            let mut file = File::create(&Path::new((build_path.as_str().unwrap().to_string() + "/index.html").as_slice()));

            let mut data = HashMap::new();
            data.insert("content", self.as_html());

            let template = mustache::compile_str(layout.as_slice());

            template.render(&mut file, &data);
        } else {
            let mut file = File::create(
                &Path::new(
                    (document_path.as_str().unwrap().to_string() + "/" + self.attributes.get_copy(&"name".to_string()) + ".html").as_slice()
                )
            );

            let mut data = HashMap::new();
            data.insert("content", self.as_html());

            let template = mustache::compile_str(layout.as_slice());

            template.render(&mut file, &data);
        }

        println!("Created {}", self.attributes.get_copy(&"name".to_string()));
    }
}

impl fmt::Show for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Attributes: {}\nContent: {}", self.attributes, self.content)
    }
}
