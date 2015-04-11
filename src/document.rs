use std::io;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::default::Default;
use liquid::Renderable;
use liquid::LiquidOptions;
use liquid::Context;
use liquid::value::Value;
use std::io::Write;

use liquid;

pub struct Document {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub content: String,
}

impl Document {
    pub fn new(name: String, attributes: HashMap<String, String>, content: String) -> Document {
        Document {
            name: name,
            attributes: attributes,
            content: content,
        }
    }

    pub fn as_html(&self) -> String {
        let mut options : LiquidOptions = Default::default();
        let template = liquid::parse(&self.content, &mut options).unwrap();

        // TODO: pass in documents as template data if as_html is called on Index Document..
        let mut data = Context{
            values: HashMap::new(),
            filters: Default::default()
        };
        let w = template.render(&mut data);

        w.unwrap()
    }

    pub fn create_file(&self, dest: &Path, layouts: &HashMap<String, String>) -> io::Result<()>{
        // construct target path
        let mut file_path_buf = PathBuf::new();
        file_path_buf.push(dest);
        file_path_buf.push(&self.name);
        file_path_buf.set_extension("html");

        let file_path = file_path_buf.as_path();

        let layout_path = self.attributes.get(&"@extends".to_string()).unwrap();
        let layout = layouts.get(layout_path).unwrap();

        // create target directories if any exist
        match file_path.parent() {
            Some(ref parents) => try!(fs::create_dir_all(parents)),
            None => ()
        };
        

        let mut file = try!(File::create(&file_path));

        let mut data = Context {
            values: HashMap::new(),
            filters: Default::default()
        };

        data.values.insert("content".to_string(), Value::Str(self.as_html()));

        // Insert the attributes into the layout template
        for key in self.attributes.keys() {
            data.values.insert(key.clone(), Value::Str(self.attributes.get(key).unwrap().clone()));
        }

        let mut options : LiquidOptions = Default::default();
        let template = liquid::parse(&layout, &mut options).unwrap();

        let res = template.render(&mut data).unwrap();

        println!("Created {}", file_path.display());
        file.write_all(&res.into_bytes())
    }
}

//impl fmt::Show for Document {
    //fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //write!(f, "Attributes: {}\nContent: {}\n\nFilename: {}", self.attributes, self.content, self.path.display())
    //}
//}
