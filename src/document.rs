use std::fmt;
use std::io;
use std::collections::HashMap;
use std::io::File;

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

        // a Writer which impl Writer is needed here (could be file, could be socket or any other writer)
        // StringWriter doesn't exist yet, therefore I have to use a MemWriter
        let mut w = io::MemWriter::new();

        // why do I have to say &mut here
        // mutable reference?!?!
        //
        // TODO: pass in documents as template data if as_html is called on Index Document..
        template.render(&mut w, &self.attributes);

        w.unwrap().into_ascii().into_string()
    }

    pub fn create_file(&self, path: &Path, layout_root: &Path) {
        let layout_path = layout_root.join(self.attributes.get_copy(&"@extends".to_string()));
        let layout      = File::open(&layout_path).read_to_string().unwrap();
        let file_path   = path.join(self.filename.as_slice());

        let mut file = File::create(&file_path);
        let mut data = HashMap::new();

        data.insert("content", self.as_html());

        // Insert the attributes into the layout template
        for key in self.attributes.keys() {
            data.insert(key.as_slice(), self.attributes.get_copy(key));
        }

        let template = mustache::compile_str(layout.as_slice());

        template.render(&mut file, &data);

        println!("Created {}{}", path.display(), self.filename);
    }
}

impl fmt::Show for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Attributes: {}\nContent: {}\n\nFilename: {}", self.attributes, self.content, self.filename)
    }
}
