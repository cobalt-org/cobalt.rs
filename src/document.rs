use std::fmt;
use std::io;
use std::collections::HashMap;
use std::io::fs;
use std::io::File;
use std::io::IoResult;

use mustache;

pub struct Document {
    pub attributes: HashMap<String, String>,
    pub content: String,
    pub path: Path,
}

impl Document {
    pub fn new(attributes: HashMap<String, String>, content: String, path: Path) -> Document {
        Document {
            attributes: attributes,
            content: content,
            path: path,
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

    pub fn create_file(&self, dest: &Path, layout_root: &Path) -> IoResult<()>{
        let layout_path = layout_root.join(self.attributes.get_copy(&"@extends".to_string()));
        // TODO this is super inefficient
        let layout      = File::open(&layout_path).read_to_string().unwrap();
        let file_path   = dest.join(&self.path);

        // create target directories
        try!(fs::mkdir_recursive(&file_path.dir_path(), io::USER_RWX));

        let mut file = File::create(&file_path);
        let mut data = HashMap::new();

        data.insert("content", self.as_html());

        // Insert the attributes into the layout template
        for key in self.attributes.keys() {
            data.insert(key.as_slice(), self.attributes.get_copy(key));
        }

        let template = mustache::compile_str(layout.as_slice());

        // TODO: wrap with try!, mustache.rs uses its own error format :/
        template.render(&mut file, &data);

        println!("Created {}", file_path.display());
        Ok(())
    }
}

impl fmt::Show for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Attributes: {}\nContent: {}\n\nFilename: {}", self.attributes, self.content, self.path.display())
    }
}
