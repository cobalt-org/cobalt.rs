use std::fs::File;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::default::Default;
use error::Result;
use chrono::{DateTime, FixedOffset, Datelike, Timelike};
use yaml_rust::YamlLoader;
use std::io::Read;
use regex::Regex;
use rss;

use liquid::{Renderable, LiquidOptions, Context, Value};

use pulldown_cmark as cmark;
use liquid;

#[derive(Debug)]
pub struct Document {
    pub path: String,
    // TODO store attributes as liquid values?
    pub attributes: HashMap<String, String>,
    pub content: String,
    pub layout: Option<String>,
    pub is_post: bool,
    pub date: Option<DateTime<FixedOffset>>,
    file_path: String,
    markdown: bool,
}

fn read_file(path: &Path) -> Result<String> {
    let mut file = try!(File::open(path));
    let mut text = String::new();
    try!(file.read_to_string(&mut text));
    Ok(text)
}

/// Formats a user specified custom path, adding custom parameters
/// and "exploding" the URL.
fn format_path(p: &str,
               attributes: &HashMap<String, String>,
               date: &Option<DateTime<FixedOffset>>)
               -> Result<String> {
    let mut p = p.to_owned();

    let time_vars = Regex::new(":(year|month|i_month|day|i_day|short_year|hour|minute|second)")
                        .unwrap();
    if time_vars.is_match(&p) {
        let date = try!(date.ok_or(format!("Can not format file path without a valid date \
                                            ({:?})",
                                           p)));

        p = p.replace(":year", &date.year().to_string());
        p = p.replace(":month", &format!("{:02}", &date.month()));
        p = p.replace(":i_month", &date.month().to_string());
        p = p.replace(":day", &format!("{:02}", &date.day()));
        p = p.replace(":i_day", &date.day().to_string());
        p = p.replace(":hour", &format!("{:02}", &date.hour()));
        p = p.replace(":minute", &format!("{:02}", &date.minute()));
        p = p.replace(":second", &format!("{:02}", &date.second()));
    }

    for (key, val) in attributes {
        p = p.replace(&(String::from(":") + key), val);
    }

    // TODO if title is present inject title slug

    let mut path = Path::new(&p);

    // remove the root prefix (leading slash on unix systems)
    if path.has_root() {
        let mut components = path.components();
        components.next();
        path = components.as_path();
    }

    let mut path_buf = path.to_path_buf();

    // explode the url if no extension was specified
    if path_buf.extension().is_none() {
        path_buf.push("index.html")
    }

    Ok(path_buf.to_string_lossy().into_owned())
}

impl Document {
    pub fn new(path: String,
               attributes: HashMap<String, String>,
               content: String,
               layout: Option<String>,
               is_post: bool,
               date: Option<DateTime<FixedOffset>>,
               file_path: String,
               markdown: bool)
               -> Document {
        Document {
            path: path,
            attributes: attributes,
            content: content,
            layout: layout,
            is_post: is_post,
            date: date,
            file_path: file_path,
            markdown: markdown,
        }
    }

    pub fn parse(file_path: &Path, source: &Path) -> Result<Document> {
        let mut attributes = HashMap::new();
        let mut content = try!(read_file(file_path));

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
                                                          file_path)));

            for (key, value) in yaml_attributes {
                // TODO store attributes as liquid values
                attributes.insert(key.as_str().unwrap_or("").to_owned(),
                                  value.as_str().unwrap_or("").to_owned());
            }
        }

        let date = attributes.get("date")
                             .and_then(|d| {
                                 DateTime::parse_from_str(d, "%d %B %Y %H:%M:%S %z").ok()
                             });

        // if the file has a .md extension we assume it's markdown
        // TODO add a "markdown" flag to yaml front matter
        let markdown = file_path.extension().unwrap_or(OsStr::new("")) == OsStr::new("md");

        let layout = attributes.get(&"extends".to_owned()).cloned();

        let path_str = try!(file_path.to_str()
                                     .ok_or(format!("Cannot convert pathname {:?} to UTF-8",
                                                    file_path)));

        let source_str = try!(source.to_str()
                                    .ok_or(format!("Cannot convert pathname {:?} to UTF-8",
                                                   source)));

        // construct a relative path to the source
        // TODO: use strip_prefix instead
        let new_path = try!(path_str.split(source_str)
                                    .last()
                                    .ok_or(format!("Empty path")));

        let mut path_buf = PathBuf::from(new_path);
        path_buf.set_extension("html");

        // if the user specified a custom path override
        // format it and push it over the original file name
        if let Some(path) = attributes.get("path") {

            // TODO replace "date", "pretty", "ordinal" and "none"
            // for Jekyl compatibility

            path_buf = if let Some(parent) = path_buf.parent() {
                let mut p = PathBuf::from(parent);
                p.push(try!(format_path(path, &attributes, &date)));
                p
            } else {
                PathBuf::from(try!(format_path(path, &attributes, &date)))
            }
        };

        let path = try!(path_buf.to_str()
                                .ok_or(format!("Cannot convert pathname {:?} to UTF-8", path_buf)));

        Ok(Document::new(path.to_owned(),
                         attributes,
                         content,
                         layout,
                         false,
                         date,
                         file_path.to_string_lossy().into_owned(),
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
