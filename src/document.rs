use std::fs::File;
use std::collections::{HashMap, BTreeMap};
use std::collections::hash_map::Entry;
use std::path::{Path, PathBuf};
use std::default::Default;
use error::Result;
use chrono::{DateTime, FixedOffset, Datelike, Timelike};
use yaml_rust::{Yaml, YamlLoader};
use std::io::Read;
use regex::Regex;
use rss;
use itertools::Itertools;

#[cfg(all(feature="syntax-highlight", not(windows)))]
use syntax_highlight::{initialize_codeblock, decorate_markdown};

use liquid::{Renderable, LiquidOptions, Context, Value};

use pulldown_cmark as cmark;
use liquid;

lazy_static!{
    static ref DATE_VARIABLES: Regex =
        Regex::new(":(year|month|i_month|day|i_day|short_year|hour|minute|second)").unwrap();
    static ref SLUG_INVALID_CHARS: Regex = Regex::new(r"([^a-zA-Z0-9]+)").unwrap();
    static ref FRONT_MATTER_DIVIDE: Regex = Regex::new(r"---\s*\r?\n").unwrap();
    static ref MARKDOWN_REF: Regex = Regex::new(r"(?m:^ {0,3}\[[^\]]+\]:.+$)").unwrap();
}

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

fn yaml_btreemap_to_hashmap(input: &BTreeMap<Yaml, Yaml>) -> HashMap<String, Value> {
    let mut result = HashMap::new();
    for (yaml_key, yaml_value) in input {
        if let Yaml::String(ref s) = *yaml_key {
            let liquid_value = yaml_to_liquid(yaml_value).expect("Value must be liquid data");
            result.insert(s.to_owned(), liquid_value);
        } else {
            panic!("Key in yaml dictionary must be string");
        }
    }
    result
}

fn yaml_to_liquid(yaml: &Yaml) -> Option<Value> {
    match *yaml {
        Yaml::Real(ref s) |
        Yaml::String(ref s) => Some(Value::Str(s.to_owned())),
        Yaml::Integer(i) => Some(Value::Num(i as f32)),
        Yaml::Boolean(b) => Some(Value::Bool(b)),
        Yaml::Array(ref a) => Some(Value::Array(a.iter().filter_map(yaml_to_liquid).collect())),
        Yaml::Hash(ref dict) => Some(Value::Object(yaml_btreemap_to_hashmap(dict))),
        Yaml::BadValue | Yaml::Null => None,
        _ => panic!("Not implemented yet"),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, BTreeMap};
    use yaml_rust as yaml;
    use liquid;

    #[test]
    fn test_yaml_to_liquid() {
        let key_str = "key".to_owned();
        let input = {
            let mut temp = BTreeMap::new();
            temp.insert(yaml::Yaml::String(key_str.clone()), yaml::Yaml::Integer(42));
            yaml::Yaml::Hash(temp)
        };
        let expected = {
            let mut temp = HashMap::new();
            temp.insert(key_str.clone(), liquid::Value::Num(42.0));
            Some(liquid::Value::Object(temp))
        };
        assert_eq!(super::yaml_to_liquid(&input), expected);
    }

    #[test]
    fn test_dictionary_conversion() {
        let key_str = "key".to_owned();
        let input = {
            let mut temp = BTreeMap::new();
            temp.insert(yaml::Yaml::String(key_str.clone()), yaml::Yaml::Integer(42));
            temp
        };
        let expected = {
            let mut temp = HashMap::new();
            temp.insert(key_str.clone(), liquid::Value::Num(42.0));
            temp
        };
        assert_eq!(super::yaml_btreemap_to_hashmap(&input), expected);
    }
}

/// Formats a user specified custom path, adding custom parameters
/// and "exploding" the URL.
fn format_path(p: &str,
               attributes: &HashMap<String, Value>,
               date: &Option<DateTime<FixedOffset>>)
               -> Result<String> {
    let mut p = p.to_owned();

    if DATE_VARIABLES.is_match(&p) {
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

/// The base-name without an extension.  Correlates to Jekyll's :name path tag
fn file_stem(p: &Path) -> String {
    p.file_stem().map(|os| os.to_string_lossy().into_owned()).unwrap_or_else(|| "".to_owned())
}

/// Create a slug for a given file.  Correlates to Jekyll's :slug path tag
fn slugify(name: &str) -> String {
    let slug = SLUG_INVALID_CHARS.replace_all(name, "-");
    slug.trim_matches('-').to_lowercase()
}

/// Title-case a single word
fn title_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c.flat_map(|t| t.to_lowercase())).collect(),
    }
}

/// Format a user-visible title out of a slug.  Correlates to Jekyll's "title" attribute
fn titleize_slug(slug: &str) -> String {
    slug.split('-').map(title_case).join(" ")
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
        let mut attributes: HashMap<String, Value> = HashMap::new();
        let content = try!(read_file(file_path));

        // if there is front matter, split the file and parse it
        let content = if FRONT_MATTER_DIVIDE.is_match(&content) {
            let mut splits = FRONT_MATTER_DIVIDE.splitn(&content, 2);

            // above the split are the attributes
            let attribute_split = splits.next().unwrap_or("");

            // everything below the split becomes the new content
            let content_split = splits.next().unwrap_or("").to_owned();

            let yaml_result = try!(YamlLoader::load_from_str(attribute_split));

            let yaml_attributes = try!(yaml_result[0]
                .as_hash()
                .ok_or_else(|| format!("Incorrect front matter format in {:?}", file_path)));

            for (key, value) in yaml_attributes {
                if let Some(v) = yaml_to_liquid(value) {
                    let key = key.as_str()
                        .ok_or_else(|| format!("Invalid key {:?}", key))?
                        .to_owned();
                    attributes.insert(key, v);
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

        let file_stem = file_stem(new_path);
        let slug = slugify(&file_stem);
        attributes.entry("title".to_owned())
            .or_insert_with(|| Value::Str(titleize_slug(slug.as_str())));
        attributes.entry("slug".to_owned())
            .or_insert_with(|| Value::Str(slug));

        let mut markdown = false;
        if let Value::Str(ref ext) =
            *attributes.entry("ext".to_owned())
                .or_insert_with(|| {
                    Value::Str(new_path.extension()
                        .and_then(|os| os.to_str())
                        .unwrap_or("")
                        .to_owned())
                }) {
            markdown = ext == "md";
        }

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
            .ok_or_else(|| format!("Cannot convert pathname {:?} to UTF-8", path_buf)));

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
            .or_else(|| self.attributes.get("excerpt"))
            .or_else(|| self.attributes.get("content"))
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

    /// Extracts references iff markdown content.
    pub fn extract_markdown_references(&self, excerpt_separator: &str) -> String {
        let mut trail = String::new();

        if self.markdown && MARKDOWN_REF.is_match(&self.content) {
            for mat in MARKDOWN_REF.find_iter(&self.content) {
                trail.push_str(mat.as_str());
                trail.push('\n');
            }
        }
        trail +
        self.content
            .split(excerpt_separator)
            .next()
            .unwrap_or(&self.content)
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

            if let Some(excerpt_str) = excerpt_attr {
                try!(self.render_html(excerpt_str, context, source))
            } else if excerpt_separator.is_empty() {
                try!(self.render_html("", context, source))
            } else {
                try!(self.render_html(&self.extract_markdown_references(excerpt_separator),
                                      context,
                                      source))
            }
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

#[test]
fn test_file_stem() {
    let input = PathBuf::from("/embedded/path/___filE-worlD-__09___.md");
    let actual = file_stem(input.as_path());
    assert_eq!(actual, "___filE-worlD-__09___");
}

#[test]
fn test_slugify() {
    let actual = slugify("___filE-worlD-__09___");
    assert_eq!(actual, "file-world-09");
}

#[test]
fn test_titleize_slug() {
    let actual = titleize_slug("tItLeIzE-sLuG");
    assert_eq!(actual, "Titleize Slug");
}
