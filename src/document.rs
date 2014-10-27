use std::fmt;
use std::io;
use std::collections::HashMap;
use std::io::File;

// reimport mustache from root module where its imported via extern crate mustache;
use mustache;

pub struct Document {
    pub attributes: HashMap<String, String>,
    pub content: String,
    pub filename: String,
}

impl Document {
    pub fn new(attributes: HashMap<String, String>, content: String, filename: String) -> Document {
        Document {
            attributes: attributes,
            content: content,
            filename: filename,
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
        //
        // TODO: pass in documents as template data if as_html is called on Index Document..
        template.render(&mut w, &self.attributes);

        w.unwrap().into_ascii().into_string()
        /* and end it here, I don't know whats going on here... WAT */
    }

    pub fn create_file(&self, layout: &str, path: &str) {
        let file_path = (path.to_string() + self.filename);

        let mut file = File::create(&Path::new(file_path.as_slice()));
        let mut data = HashMap::new();

        data.insert("content", self.as_html());

        let template = mustache::compile_str(layout);

        template.render(&mut file, &data);

        println!("Created {}{}", path, self.filename);
    }
}

impl fmt::Show for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Attributes: {}\nContent: {}\n\nFilename: {}", self.attributes, self.content, self.filename)
    }
}
