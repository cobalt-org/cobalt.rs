use std::fs::{File};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::default::Default;
use error::Result;
use chrono::{DateTime, FixedOffset};
use yaml_rust::YamlLoader;
use std::io::Read;
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

fn read_file(path: &Path) -> Result<String> {
    let mut file = try!(File::open(path));
    let mut text = String::new();
    try!(file.read_to_string(&mut text));
    Ok(text)
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

    pub fn parse(path: &Path, source: &Path) -> Result<Document> {
        let mut attributes = HashMap::new();
        let mut content = try!(read_file(path));

        // if there is front matter, split the file and parse it
        // TODO: make this a regex to support lines of any length
        if content.contains("---") {
            let content2 = content.clone();
            let mut content_splits = content2.split("---");

            // above the split are the attributes
            let attribute_string = content_splits.next().unwrap_or("");

            // everything below the split becomes the new content
            content = content_splits.next().unwrap_or("").to_owned();

            let yaml_result = try!(YamlLoader::load_from_str(attribute_string));

            let yaml_attributes = try!(yaml_result[0]
                                           .as_hash()
                                           .ok_or(format!("Incorrect front matter format in \
                                                           {:?}",
                                                          path)));

            for (key, value) in yaml_attributes {
                // TODO is unwrap_or the best way to handle this?
                attributes.insert(key.as_str().unwrap_or("").to_owned(),
                                  value.as_str().unwrap_or("").to_owned());
            }
        }

        let date = attributes.get("date")
                             .and_then(|d| {
                                 DateTime::parse_from_str(d, "%d %B %Y %H:%M:%S %z").ok()
                             });

        let path_str = try!(path.to_str()
                                .ok_or(format!("Cannot convert pathname {:?} to UTF-8", path)));

        let source_str = try!(source.to_str()
                                    .ok_or(format!("Cannot convert pathname {:?} to UTF-8",
                                                   source)));

        let new_path = try!(path_str.split(source_str)
                                    .last()
                                    .ok_or(format!("Empty path")));

        // construct path
        let mut path_buf = PathBuf::from(new_path);
        path_buf.set_extension("html");

        let path_str = try!(path_buf.to_str()
                                    .ok_or(format!("Cannot convert pathname {:?} to UTF-8",
                                                   path_str)));

        let markdown = path.extension().unwrap_or(OsStr::new("")) == OsStr::new("md");

        let name = try!(path.file_stem()
                            .and_then(|stem| stem.to_str())
                            .ok_or(format!("Invalid UTF-8 in file stem for {:?}", path)));

        Ok(Document::new(name.to_owned(),
                         path_str.to_owned(),
                         attributes,
                         content,
                         false,
                         date,
                         markdown))
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

        for (key, val) in &self.attributes {
            data.insert(key.to_owned(), Value::Str(val.clone()));
        }

        // We replace to swap back slashes to forward slashes to ensure the URL's are valid
        data.insert("path".to_owned(), Value::Str(self.path.replace("\\", "/")));

        data.insert("is_post".to_owned(), Value::Bool(self.is_post));

        data
    }

    pub fn as_html(&self,
                   source: &Path,
                   post_data: &Vec<Value>,
                   layouts: &HashMap<String, String>)
                   -> Result<String> {
        let options = LiquidOptions { file_system: Some(source.to_owned()), ..Default::default() };
        let template = try!(liquid::parse(&self.content, options));

        let layout = if let Some(ref layout) = self.layout {
            Some(try!(layouts.get(layout)
                           .ok_or(format!("Layout {} can not be found (defined in {})",
                                          layout,
                                          self.file_path))))
        } else {
            None
        };

        let mut data = Context::with_values(self.get_attributes());
        data.set_val("posts", Value::Array(post_data.clone()));

        let mut html = try!(template.render(&mut data)).unwrap_or(String::new());

        if self.markdown {
            html = {
                let mut buf = String::new();
                let parser = cmark::Parser::new(&html);
                cmark::html::push_html(&mut buf, parser);
                buf
            };
        }

        let options = LiquidOptions { file_system: Some(source.to_owned()), ..Default::default() };

        let template = if let Some(layout) = layout {
            data.set_val("content", Value::Str(html));

            try!(liquid::parse(&layout, options))
        } else {
            try!(liquid::parse(&html, options))
        };

        Ok(try!(template.render(&mut data)).unwrap_or(String::new()))
    }
}
