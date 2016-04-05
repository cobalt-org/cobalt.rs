use std::fs::{self, File};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::default::Default;
use std::io::Write;
use error::Result;
use chrono::{DateTime, FixedOffset};
use rss;

use liquid::{Renderable, LiquidOptions, Context, Value};

use pulldown_cmark as cmark;
use liquid;

#[derive(Debug)]
pub struct Document {
    pub name: String,
    pub path: String,
    pub attributes: HashMap<String, String>,
    pub content: String,
    pub is_post: bool,
    pub date: Option<DateTime<FixedOffset>>,
    markdown: bool,
}

impl Document {
    pub fn new(name: String,
               path: String,
               attributes: HashMap<String, String>,
               content: String,
               is_post: bool,
               date: Option<DateTime<FixedOffset>>,
               markdown: bool)
               -> Document {
        Document {
            name: name,
            path: path,
            attributes: attributes,
            content: content,
            is_post: is_post,
            date: date,
            markdown: markdown,
        }
    }

    /// Metadata for generating RSS feeds
    pub fn to_rss(&self, root_url: &str) -> rss::Item {
        rss::Item {
            title: self.attributes.get("title").map(|s| s.to_owned()),
            link: Some(root_url.to_owned() + &self.path),
            pub_date: self.date.map(|date| date.to_rfc2822()),
            description: self.attributes.get("description").map(|s| s.to_owned()),
            ..Default::default()
        }
    }

    /// Attributes that are injected into the template when rendering
    pub fn get_attributes(&self) -> HashMap<String, Value> {
        let mut data = HashMap::new();
        data.insert("name".to_owned(), Value::Str(self.name.clone()));
        data.insert("path".to_owned(), Value::Str(self.path.clone()));
        for key in self.attributes.keys() {
            if let Some(val) = self.attributes.get(key) {
                data.insert(key.to_owned(), Value::Str(val.clone()));
            }
        }
        data
    }

    pub fn as_html(&self, post_data: &Vec<Value>) -> Result<String> {
        let options: LiquidOptions = Default::default();
        let template = try!(liquid::parse(&self.content, options));

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
        file_path_buf.push(&self.path);
        file_path_buf.set_extension("html");

        let file_path = file_path_buf.as_path();

        let layout_path = try!(self.attributes
                                   .get(&"extends".to_owned())
                                   .ok_or(format!("No extends property in {}", self.name)));

        let layout = try!(layouts.get(layout_path)
                                 .ok_or(format!("Layout {} can not be found (defined in {})",
                                                layout_path,
                                                self.name)));

        // create target directories if any exist
        file_path.parent().map(|p| fs::create_dir_all(p));

        let mut file = try!(File::create(&file_path));

        // Insert the attributes into the layout template
        // TODO we're currently calling get_attributes twice on each document render, can we get it
        // to a single call?
        let mut data = Context::with_values(self.get_attributes());

        // compile with liquid
        let mut html = try!(self.as_html(post_data));

        if self.markdown {
            html = {
                let mut buf = String::new();
                let parser = cmark::Parser::new(&html);
                cmark::html::push_html(&mut buf, parser);
                buf
            };
        }

        data.set_val("content", Value::Str(html));
        if self.is_post {
            data.set_val("is_post", Value::Bool(true));
        }

        let options: LiquidOptions = Default::default();

        let template = try!(liquid::parse(&layout, options));

        let res = try!(template.render(&mut data)).unwrap_or(String::new());

        try!(file.write_all(&res.into_bytes()));
        info!("Created {}", file_path.display());
        Ok(())
    }
}
