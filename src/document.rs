use std::fs::File;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::default::Default;
use error::Result;
use chrono::{DateTime, FixedOffset, Datelike, Timelike};
use yaml_rust::{Yaml, YamlLoader};
use std::io::Read;
use regex::Regex;
use rss;

use liquid::{Renderable, LiquidOptions, Context, Value};

use pulldown_cmark as cmark;
use liquid;

#[derive(Debug)]
pub struct Document {
    pub path: String,
    pub attributes: HashMap<String, Value>,
    pub content: String,
    pub layout: Option<String>,
    pub is_post: bool,
    pub date: Option<DateTime<FixedOffset>>,
    file_path: String,
    markdown: bool,
}

fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = try!(File::open(path));
    let mut text = String::new();
    try!(file.read_to_string(&mut text));
    Ok(text)
}

fn yaml_to_liquid(yaml: &Yaml) -> Option<Value> {
    match *yaml {
        Yaml::Real(ref s) |
        Yaml::String(ref s) => Some(Value::Str(s.to_owned())),
        Yaml::Integer(i) => Some(Value::Num(i as f32)),
        Yaml::Boolean(b) => Some(Value::Bool(b)),
        Yaml::Array(ref a) => Some(Value::Array(a.iter().filter_map(yaml_to_liquid).collect())),
        Yaml::BadValue | Yaml::Null => None,
        _ => panic!("Not implemented yet"),
    }
}

/// Formats a user specified custom path, adding custom parameters
/// and "exploding" the URL.
fn format_path(p: &str,
               attributes: &HashMap<String, Value>,
               date: &Option<DateTime<FixedOffset>>)
               -> Result<String> {
    let mut p = p.to_owned();

    let time_vars = Regex::new(":(year|month|i_month|day|i_day|short_year|hour|minute|second)")
        .unwrap();
    if time_vars.is_match(&p) {
        let date =
            try!(date.ok_or(format!("Can not format file path without a valid date ({:?})", p)));

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
        p = match *val {
            Value::Str(ref v) => p.replace(&(String::from(":") + key), v),
            Value::Num(ref v) => p.replace(&(String::from(":") + key), &v.to_string()),
            _ => p,
        }
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
               attributes: HashMap<String, Value>,
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

    pub fn parse(file_path: &Path,
                 source: &Path,
                 mut is_post: bool,
                 post_path: &Option<String>)
                 -> Result<Document> {
        let mut attributes = HashMap::new();
        let mut content = try!(read_file(file_path));

        // if there is front matter, split the file and parse it
        // TODO: make this a regex to support lines of any length
        if content.contains("---") {
            let content2 = content.clone();
            let mut content_splits = content2.splitn(2, "---");

            // above the split are the attributes
            let attribute_string = content_splits.next().unwrap_or("");

            // everything below the split becomes the new content
            content = content_splits.next().unwrap_or("").to_owned();

            let yaml_result = try!(YamlLoader::load_from_str(attribute_string));

            let yaml_attributes = try!(yaml_result[0]
                .as_hash()
                .ok_or(format!("Incorrect front matter format in {:?}", file_path)));

            for (key, value) in yaml_attributes {
                if let Some(v) = yaml_to_liquid(value) {
                    attributes.insert(try!(key.as_str().ok_or(format!("Invalid key {:?}", key)))
                                          .to_owned(),
                                      v);
                }
            }
        }

        if let &mut Value::Bool(val) = attributes.entry("is_post".to_owned())
            .or_insert(Value::Bool(is_post)) {
            is_post = val;
        }

        let date = attributes.get("date")
            .and_then(|d| d.as_str())
            .and_then(|d| DateTime::parse_from_str(d, "%d %B %Y %H:%M:%S %z").ok());

        // if the file has a .md extension we assume it's markdown
        // TODO add a "markdown" flag to yaml front matter
        let markdown = file_path.extension().unwrap_or(OsStr::new("")) == OsStr::new("md");

        let layout = attributes.get("extends").and_then(|l| l.as_str()).map(|x| x.to_owned());

        let new_path = try!(file_path.strip_prefix(source)
            .map_err(|_| "File path not in source".to_owned()));

        let mut path_buf = PathBuf::from(new_path);
        path_buf.set_extension("html");

        // if the user specified a custom path override
        // format it and push it over the original file name
        // TODO replace "date", "pretty", "ordinal" and "none"
        // for Jekyl compatibility
        if let Some(path) = attributes.get("path").and_then(|p| p.as_str()) {
            path_buf = PathBuf::from(try!(format_path(path, &attributes, &date)));
        } else if is_post {
            // check if there is a global setting for post paths
            if let &Some(ref path) = post_path {
                path_buf = PathBuf::from(try!(format_path(path, &attributes, &date)));
            }
        };

        let path = try!(path_buf.to_str()
            .ok_or(format!("Cannot convert pathname {:?} to UTF-8", path_buf)));

        // Swap back slashes to forward slashes to ensure the URL's are valid on Windows
        attributes.insert("path".to_owned(), Value::Str(path.replace("\\", "/")));

        Ok(Document::new(path.to_owned(),
                         attributes,
                         content,
                         layout,
                         is_post,
                         date,
                         file_path.to_string_lossy().into_owned(),
                         markdown))
    }


    /// Metadata for generating RSS feeds
    pub fn to_rss(&self, root_url: &str) -> rss::Item {
        rss::Item {
            title: self.attributes.get("title").and_then(|s| s.as_str()).map(|s| s.to_owned()),
            link: Some(root_url.to_owned() + &self.path),
            pub_date: self.date.map(|date| date.to_rfc2822()),
            description: self.attributes
                .get("description")
                .and_then(|s| s.as_str())
                .map(|s| s.to_owned()),
            ..Default::default()
        }
    }

    pub fn as_html(&self,
                   source: &Path,
                   post_data: &[Value],
                   layouts_path: &Path)
                   -> Result<String> {
        let options = LiquidOptions { file_system: Some(source.to_owned()), ..Default::default() };
        let template = try!(liquid::parse(&self.content, options));

        let layout = if let Some(ref layout) = self.layout {
            Some(try!(read_file(layouts_path.join(layout)).map_err(|e| {
                format!("Layout {} can not be read (defined in {}): {}",
                        layout,
                        self.file_path,
                        e)
            })))
        } else {
            None
        };

        let mut data = Context::with_values(self.attributes.clone());
        data.set_val("posts", Value::Array(post_data.to_vec()));

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
