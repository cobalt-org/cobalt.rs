use std::{io, fs};
use std::fs::File;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::default::Default;
use std::io::Write;

use liquid::{Renderable, LiquidOptions, Context, Value};

use markdown;
use liquid;

#[derive(Debug)]
pub struct Document {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub content: String,
    markdown: bool,
}

impl Document {
    pub fn new(name: String,
               attributes: HashMap<String, String>,
               content: String,
               markdown: bool)
               -> Document {
        Document {
            name: name,
            attributes: attributes,
            content: content,
            markdown: markdown,
        }
    }

    pub fn get_attributes(&self) -> HashMap<String, Value> {
        let mut data = HashMap::new();
        for key in self.attributes.keys() {
            if let Some(val) = self.attributes.get(key) {
                data.insert(key.to_owned(), Value::Str(val.clone()));
            }
        }
        data
    }

    pub fn as_html(&self, post_data: &Vec<Value>) -> Result<String, String> {
        let mut options: LiquidOptions = Default::default();
        let template = try!(liquid::parse(&self.content, &mut options));

        // TODO: pass in documents as template data if as_html is called on Index Document..
        let mut data = Context::with_values(self.get_attributes());
        data.set_val("posts", Value::Array(post_data.clone()));

        Ok(template.render(&mut data).unwrap_or(String::new()))
    }

    pub fn create_file(&self,
                       dest: &Path,
                       layouts: &HashMap<String, String>,
                       post_data: &Vec<Value>)
                       -> io::Result<()> {
        // construct target path
        let mut file_path_buf = PathBuf::new();
        file_path_buf.push(dest);
        file_path_buf.push(&self.name);
        file_path_buf.set_extension("html");

        let file_path = file_path_buf.as_path();

        let layout_path = self.attributes.get(&"@extends".to_owned()).expect(&format!("No @extends line creating {:?}", self.name));
        let layout = layouts.get(layout_path).expect(&format!("No layout path {:?} creating {:?}", layout_path, self.name));

        // create target directories if any exist
        match file_path.parent() {
            Some(ref parents) => try!(fs::create_dir_all(parents)),
            None => (),
        };

        let mut file = try!(File::create(&file_path));

        let mut data = Context::new();

        // TODO: improve error handling for liquid errors
        let mut html = match self.as_html(post_data) {
            Ok(x) => x,
            Err(e) => {
                println!("Warning, liquid failed: {}", e);
                String::new()
            }
        };
        if self.markdown {
            html = markdown::to_html(&html);
        }
        data.set_val("content", Value::Str(html));

        // Insert the attributes into the layout template
        for key in self.attributes.keys() {
            if let Some(val) = self.attributes.get(key) {
                data.set_val(key, Value::Str(val.clone()));
            }
        }

        let mut options: LiquidOptions = Default::default();
        // TODO: improve error handling for liquid errors
        let template = match liquid::parse(&layout, &mut options) {
            Ok(x) => x,
            Err(e) => {
                panic!("Warning, liquid failed: {}", e);
            }
        };

        let res = template.render(&mut data).unwrap_or(String::new());

        println!("Created {}", file_path.display());
        file.write_all(&res.into_bytes())
    }
}
