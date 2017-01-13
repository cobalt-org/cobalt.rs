use std::fs::File;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::default::Default;
use error::Result;
use chrono::{DateTime, FixedOffset, Datelike, Timelike};
use yaml_rust::{Yaml, YamlLoader};
use std::io::Read;
use regex::Regex;
use rss;

#[cfg(all(feature="syntax-highlight", not(windows)))]
use syntax_highlight::{initialize_codeblock, decorate_markdown};

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
    pub is_draft: bool,
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
               is_draft: bool,
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
            is_draft: is_draft,
            date: date,
            file_path: file_path,
            markdown: markdown,
        }
    }

    pub fn parse(file_path: &Path,
                 new_path: &Path,
                 mut is_post: bool,
                 post_path: &Option<String>)
                 -> Result<Document> {
        let mut attributes = HashMap::new();
        let content = try!(read_file(file_path));

        // if there is front matter, split the file and parse it
        let splitter = Regex::new(r"---\s*\r?\n").unwrap();
        let content = if splitter.is_match(&content) {
            let mut splits = splitter.splitn(&content, 2);

            // above the split are the attributes
            let attribute_split = splits.next().unwrap_or("");

            // everything below the split becomes the new content
            let content_split = splits.next().unwrap_or("").to_owned();

            let yaml_result = try!(YamlLoader::load_from_str(attribute_split));

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

            content_split
        } else {
            content
        };

        if let Value::Bool(val) = *attributes.entry("is_post".to_owned())
            .or_insert_with(|| Value::Bool(is_post)) {
            is_post = val;
        }

        let is_draft = if let Some(&Value::Bool(true)) = attributes.get("draft") {
            true
        } else {
            false
        };

        let date = attributes.get("date")
            .and_then(|d| d.as_str())
            .and_then(|d| DateTime::parse_from_str(d, "%d %B %Y %H:%M:%S %z").ok());

        // if the file has a .md extension we assume it's markdown
        // TODO add a "markdown" flag to yaml front matter
        let markdown = file_path.extension().unwrap_or_else(|| OsStr::new("")) == OsStr::new("md");

        let layout = attributes.get("extends").and_then(|l| l.as_str()).map(|x| x.to_owned());

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
            if let Some(ref path) = *post_path {
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
                         is_draft,
                         date,
                         file_path.to_string_lossy().into_owned(),
                         markdown))
    }


    /// Metadata for generating RSS feeds
    pub fn to_rss(&self, root_url: &str) -> rss::Item {
        let description = self.attributes
            .get("description")
            .or(self.attributes.get("excerpt"))
            .or(self.attributes.get("content"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_owned());

        // Swap back slashes to forward slashes to ensure the URL's are valid on Windows
        let link = root_url.to_owned() + &self.path.replace("\\", "/");
        let guid = rss::Guid {
            value: link.clone(),
            is_perma_link: true,
        };

        rss::Item {
            title: self.attributes.get("title").and_then(|s| s.as_str()).map(|s| s.to_owned()),
            link: Some(link),
            guid: Some(guid),
            pub_date: self.date.map(|date| date.to_rfc2822()),
            description: description,
            ..Default::default()
        }
    }

    /// Prepares liquid context for further rendering.
    pub fn get_render_context(&self, posts: &[Value]) -> Context {
        let mut context = Context::with_values(self.attributes.clone());
        context.set_val("posts", Value::Array(posts.to_vec()));
        context
    }

    /// Renders liquid templates into HTML in the context of current document.
    ///
    /// Takes `content` string and returns rendered HTML. This function doesn't
    /// take `"extends"` attribute into account. This function can be used for
    /// rendering content or excerpt.
    #[cfg(all(feature="syntax-highlight", not(windows)))]
    fn render_html(&self, content: &str, context: &mut Context, source: &Path) -> Result<String> {
        let mut options =
            LiquidOptions { file_system: Some(source.to_owned()), ..Default::default() };
        options.blocks.insert("highlight".to_string(), Box::new(initialize_codeblock));
        let template = try!(liquid::parse(content, options));
        let mut html = try!(template.render(context)).unwrap_or(String::new());

        if self.markdown {
            html = {
                let mut buf = String::new();
                let parser = cmark::Parser::new(&html);
                #[cfg(feature="syntax-highlight")]
                cmark::html::push_html(&mut buf, decorate_markdown(parser));
                #[cfg(not(feature="syntax-highlight"))]
                cmark::html::push_html(&mut buf, parser);
                buf
            };
        }
        Ok(html.to_owned())
    }
    #[cfg(any(not(feature="syntax-highlight"), windows))]
    fn render_html(&self, content: &str, context: &mut Context, source: &Path) -> Result<String> {
        let options = LiquidOptions { file_system: Some(source.to_owned()), ..Default::default() };
        let template = try!(liquid::parse(content, options));
        let mut html = try!(template.render(context)).unwrap_or_default();

        if self.markdown {
            html = {
                let mut buf = String::new();
                let parser = cmark::Parser::new(&html);
                cmark::html::push_html(&mut buf, parser);
                buf
            };
        }
        Ok(html.to_owned())
    }

    /// Renders excerpt and adds it to attributes of the document.
    pub fn render_excerpt(&mut self,
                          context: &mut Context,
                          source: &Path,
                          default_excerpt_separator: &str)
                          -> Result<()> {
        let excerpt_html = {
            let excerpt_attr = self.attributes
                .get("excerpt")
                .and_then(|attr| attr.as_str());

            let excerpt_separator: &str = self.attributes
                .get("excerpt_separator")
                .and_then(|attr| attr.as_str())
                .unwrap_or(default_excerpt_separator);

            let excerpt = if let Some(excerpt_str) = excerpt_attr {
                excerpt_str
            } else if excerpt_separator.is_empty() {
                ""
            } else {
                self.content.split(excerpt_separator).next().unwrap_or(&self.content)
            };

            try!(self.render_html(excerpt, context, source))
        };

        self.attributes.insert("excerpt".to_owned(), Value::Str(excerpt_html));
        Ok(())
    }

    /// Renders the document to an HTML string.
    ///
    /// Side effects:
    ///
    /// * content is inserted to the attributes of the document
    /// * content is inserted to context
    /// * layout may be inserted to layouts cache
    ///
    /// When we say "content" we mean only this document without extended layout.
    pub fn render(&mut self,
                  context: &mut Context,
                  source: &Path,
                  layouts_dir: &Path,
                  layouts_cache: &mut HashMap<String, String>)
                  -> Result<String> {
        let content_html = try!(self.render_html(&self.content, context, source));
        self.attributes.insert("content".to_owned(), Value::Str(content_html.clone()));
        context.set_val("content", Value::Str(content_html.clone()));

        if let Some(ref layout) = self.layout {
            let layout_data_ref = match layouts_cache.entry(layout.to_owned()) {
                Entry::Vacant(vacant) => {
                    let layout_data = try!(read_file(layouts_dir.join(layout)).map_err(|e| {
                        format!("Layout {} can not be read (defined in {}): {}",
                                layout,
                                self.file_path,
                                e)
                    }));
                    vacant.insert(layout_data)
                }
                Entry::Occupied(occupied) => occupied.into_mut(),
            };

            let options =
                LiquidOptions { file_system: Some(source.to_owned()), ..Default::default() };
            let template = try!(liquid::parse(layout_data_ref, options));
            Ok(try!(template.render(context)).unwrap_or_default())
        } else {
            Ok(content_html)
        }
    }
}
