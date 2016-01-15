use std::fs::{self, File};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::default::Default;
use std::io::Write;
use error::Result;
use chrono::{DateTime, Local};

use liquid::{Renderable, LiquidOptions, Context, Value};

use markdown;
use liquid;

#[derive(Debug)]
pub struct Document {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub content: String,
    pub is_post: bool,
    pub date: Option<DateTime<Local>>,
    markdown: bool,
}

impl Document {
    pub fn new(name: String,
               attributes: HashMap<String, String>,
               content: String,
               is_post: bool,
               date: Option<DateTime<Local>>,
               markdown: bool)
               -> Document {
        Document {
            name: name,
            attributes: attributes,
            content: content,
            is_post: is_post,
            date: date,
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

    pub fn as_html(&self, post_data: &Vec<Value>) -> Result<String> {
        let mut options: LiquidOptions = Default::default();
        let template = try!(liquid::parse(&self.content, &mut options));

        // TODO: pass in documents as template data if as_html is called on Index
        // Document..
        let mut data = Context::with_values(self.get_attributes());
        data.set_val("posts", Value::Array(post_data.clone()));

        Ok(try!(template.render(&mut data)).unwrap_or(String::new()))
    }

    pub fn create_file(&self,
                       dest: &Path,
                       layouts: &HashMap<String, String>,
                       post_data: &Vec<Value>)
                       -> Result<()> {
        // construct target path
        let mut file_path_buf = PathBuf::new();
        file_path_buf.push(dest);
        file_path_buf.push(&self.name);
        file_path_buf.set_extension("html");

        let file_path = file_path_buf.as_path();

        let layout_path = try!(self.attributes
                                   .get(&"extends".to_owned())
                                   .ok_or(format!("No extends property creating {}", self.name)));

        let layout = try!(layouts.get(layout_path)
                                 .ok_or(format!("No layout path {} creating {}",
                                                layout_path,
                                                self.name)));

        // create target directories if any exist
        file_path.parent().map(|p| fs::create_dir_all(p));

        let mut file = try!(File::create(&file_path));

        let mut data = Context::new();

        // compile with liquid
        let mut html = try!(self.as_html(post_data));

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

        let template = try!(liquid::parse(&layout, &mut options));

        let res = try!(template.render(&mut data)).unwrap_or(String::new());

        try!(file.write_all(&res.into_bytes()));
        println!("Created {}", file_path.display());
        Ok(())
    }
}
