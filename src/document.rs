use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::{Path, PathBuf};
use std::default::Default;
use error::Result;
use chrono::{Datelike, Timelike};
use regex::Regex;
use rss;
use jsonfeed;
use serde_yaml;
use itertools;

use syntax_highlight::decorate_markdown;

use liquid::{Renderable, Context, Value};

use config;
use files;
use frontmatter;
use pulldown_cmark as cmark;
use liquid;
use legacy::wildwest;
use template;

lazy_static!{
    static ref FRONT_MATTER_DIVIDE: Regex = Regex::new(r"---\s*\r?\n").unwrap();
    static ref MARKDOWN_REF: Regex = Regex::new(r"(?m:^ {0,3}\[[^\]]+\]:.+$)").unwrap();
}

pub fn split_document(content: &str) -> Result<(Option<&str>, &str)> {
    if FRONT_MATTER_DIVIDE.is_match(content) {
        let mut splits = FRONT_MATTER_DIVIDE.splitn(content, 2);

        // above the split are the attributes
        let front_split = splits.next().unwrap_or("");

        // everything below the split becomes the new content
        let content_split = splits.next().unwrap_or("");

        if front_split.is_empty() {
            Ok((None, content_split))
        } else {
            Ok((Some(front_split), content_split))
        }
    } else {
        Ok((None, content))
    }
}

/// Convert the source file's relative path into a format useful for generating permalinks that
/// mirror the source directory hierarchy.
fn format_path_variable(source_file: &Path) -> String {
    let parent = source_file
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_owned();
    let mut path = parent.replace("\\", "/");
    if path.starts_with("./") {
        path.remove(0);
    }
    if path.starts_with('/') {
        path.remove(0);
    }
    path
}

fn permalink_attributes(front: &frontmatter::Frontmatter,
                        dest_file: &Path)
                        -> HashMap<String, String> {
    let mut attributes = HashMap::new();

    attributes.insert(":path".to_owned(), format_path_variable(dest_file));

    let filename = dest_file.file_stem().and_then(|s| s.to_str());
    if let Some(filename) = filename {
        attributes.insert(":filename".to_owned(), filename.to_owned());
    }

    // TODO(epage): Add `collection` (the collection's slug), see #257
    // or `parent.slug`, see #323

    attributes.insert(":slug".to_owned(), front.slug.clone());

    // TODO(epage): slugify categories?  See #257
    attributes.insert(":categories".to_owned(),
                      itertools::join(front.categories.iter(), "/"));

    attributes.insert(":output_ext".to_owned(), ".html".to_owned());

    if let Some(ref date) = front.published_date {
        attributes.insert(":year".to_owned(), date.year().to_string());
        attributes.insert(":month".to_owned(), format!("{:02}", &date.month()));
        attributes.insert(":i_month".to_owned(), date.month().to_string());
        attributes.insert(":day".to_owned(), format!("{:02}", &date.day()));
        attributes.insert(":i_day".to_owned(), date.day().to_string());
        attributes.insert(":hour".to_owned(), format!("{:02}", &date.hour()));
        attributes.insert(":minute".to_owned(), format!("{:02}", &date.minute()));
        attributes.insert(":second".to_owned(), format!("{:02}", &date.second()));
    }

    // Allow customizing any of the above with custom frontmatter attributes
    // TODO(epage): Place in a `custom` variable.  See #257
    for (key, val) in &front.custom {
        let key = format!(":{}", key);
        // HACK: We really should support nested types
        let val = val.to_string();
        attributes.insert(key, val);
    }

    attributes
}

fn explode_permalink<S: Into<String>>(permalink: S, attributes: HashMap<String, String>) -> String {
    explode_permalink_string(permalink.into(), attributes)
}

fn explode_permalink_string(permalink: String, attributes: HashMap<String, String>) -> String {
    // TODO(epage): Switch to liquid templating
    let mut p = permalink;

    for (key, val) in attributes {
        p = p.replace(&key, &val);
    }

    // Handle cases where substutions were blank
    p = p.replace("//", "/");

    if p.starts_with('/') {
        p.remove(0);
    }

    p
}

fn format_url_as_file<S: AsRef<str>>(permalink: S) -> PathBuf {
    format_url_as_file_str(permalink.as_ref())
}

fn format_url_as_file_str(permalink: &str) -> PathBuf {
    let mut path = Path::new(&permalink);

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

    path_buf
}

fn document_attributes(front: &frontmatter::Frontmatter,
                       source_file: &str,
                       url_path: &str)
                       -> liquid::Object {
    let mut attributes = liquid::Object::new();

    attributes.insert("path".to_owned(), liquid::Value::str(url_path));
    // TODO(epage): Remove?  See #257
    attributes.insert("source".to_owned(), liquid::Value::str(source_file));
    attributes.insert("title".to_owned(), liquid::Value::str(&front.title));
    if let Some(ref description) = front.description {
        attributes.insert("description".to_owned(), liquid::Value::str(description));
    }
    attributes.insert("categories".to_owned(),
                      liquid::Value::Array(front
                                               .categories
                                               .iter()
                                               .map(|c| liquid::Value::str(c))
                                               .collect()));
    if let Some(ref published_date) = front.published_date {
        // TODO(epage): Rename to published_date. See #257
        attributes.insert("date".to_owned(),
                          liquid::Value::Str(published_date.format()));
    }
    // TODO(epage): Rename to `is_draft`. See #257
    attributes.insert("draft".to_owned(), liquid::Value::Bool(front.is_draft));
    // TODO(epage): Remove? See #257
    attributes.insert("is_post".to_owned(), liquid::Value::Bool(front.is_post));

    // TODO(epage): Place in a `custom` variable.  See #257
    for (key, val) in &front.custom {
        attributes.insert(key.clone(), val.clone());
    }

    attributes
}

#[derive(Debug)]
pub struct Document {
    pub url_path: String,
    pub file_path: PathBuf,
    pub content: String,
    pub attributes: liquid::Object,
    pub front: frontmatter::Frontmatter,
}

impl Document {
    pub fn new(url_path: String,
               file_path: PathBuf,
               content: String,
               attributes: liquid::Object,
               front: frontmatter::Frontmatter)
               -> Document {
        Document {
            url_path: url_path,
            file_path: file_path,
            content: content,
            attributes: attributes,
            front: front,
        }
    }

    pub fn parse(src_path: &Path,
                 rel_path: &Path,
                 default_front: frontmatter::FrontmatterBuilder)
                 -> Result<Document> {
        trace!("Parsing {:?}", rel_path);
        let content = files::read_file(src_path)?;
        let (front, content) = split_document(&content)?;
        let legacy_front: wildwest::FrontmatterBuilder =
            front
                .map(|s| serde_yaml::from_str(s))
                .map_or(Ok(None), |r| r.map(Some))?
                .unwrap_or_else(wildwest::FrontmatterBuilder::new);

        let front: frontmatter::FrontmatterBuilder = legacy_front.into();
        let front = front.merge_path(rel_path).merge(default_front);

        let front = front.build()?;

        let perma_attributes = permalink_attributes(&front, rel_path);
        let (file_path, url_path) = {
            let permalink = front.permalink.as_ref();
            let url_path = explode_permalink(permalink, perma_attributes);
            let file_path = format_url_as_file(&url_path);
            (file_path, url_path)
        };

        let doc_attributes =
            document_attributes(&front, rel_path.to_str().unwrap_or(""), url_path.as_ref());

        Ok(Document::new(url_path,
                         file_path,
                         content.to_string(),
                         doc_attributes,
                         front))
    }


    /// Metadata for generating RSS feeds
    pub fn to_rss(&self, root_url: &str) -> Result<rss::Item> {
        let link = format!("{}/{}", root_url, &self.url_path);
        let guid = rss::GuidBuilder::default()
            .value(link.clone())
            .permalink(true)
            .build()?;

        let item = rss::ItemBuilder::default()
            .title(Some(self.front.title.clone()))
            .link(Some(link))
            .guid(Some(guid))
            .pub_date(self.front.published_date.map(|date| date.to_rfc2822()))
            .description(self.description_to_str())
            .build()?;
        Ok(item)
    }

    /// Metadata for generating JSON feeds
    pub fn to_jsonfeed(&self, root_url: &str) -> jsonfeed::Item {
        let link = format!("{}/{}", root_url, &self.url_path);

        jsonfeed::Item {
            id: link.clone(),
            url: Some(link),
            title: Some(self.front.title.clone()),
            content: jsonfeed::Content::Html(self.description_to_str()
                                                 .unwrap_or_else(|| "".into())),
            date_published: self.front.published_date.map(|date| date.to_rfc2822()),
            // TODO completely implement categories, see Issue 131
            tags: Some(self.front.categories.clone()),
            ..Default::default()
        }
    }

    fn description_to_str(&self) -> Option<String> {
        self.front
            .description
            .as_ref()
            .map(|s| s.as_str())
            .or_else(|| self.attributes.get("excerpt").and_then(|s| s.as_str()))
            .or_else(|| self.attributes.get("content").and_then(|s| s.as_str()))
            .map(|s| s.to_owned())
    }


    /// Prepares liquid context for further rendering.
    pub fn get_render_context(&self) -> Context {
        Context::with_values(self.attributes.clone())
    }

    /// Renders liquid templates into HTML in the context of current document.
    ///
    /// Takes `content` string and returns rendered HTML. This function doesn't
    /// take `"extends"` attribute into account. This function can be used for
    /// rendering content or excerpt.
    fn render_html(&self,
                   content: &str,
                   context: &mut Context,
                   parser: &template::LiquidParser,
                   syntax_theme: &str)
                   -> Result<String> {
        let template = parser.parse(content)?;
        let html = template.render(context)?.unwrap_or_default();

        let html = match self.front.format {
            frontmatter::SourceFormat::Raw => html,
            frontmatter::SourceFormat::Markdown => {
                let mut buf = String::new();
                let options = cmark::OPTION_ENABLE_FOOTNOTES | cmark::OPTION_ENABLE_TABLES;
                let parser = cmark::Parser::new_ext(&html, options);
                cmark::html::push_html(&mut buf, decorate_markdown(parser, syntax_theme));
                buf
            }
        };
        Ok(html.to_owned())
    }

    /// Renders excerpt and adds it to attributes of the document.
    pub fn render_excerpt(&mut self,
                          context: &mut Context,
                          parser: &template::LiquidParser,
                          syntax_theme: &str)
                          -> Result<()> {
        let excerpt_html = {
            let excerpt_attr = self.attributes
                .get("excerpt")
                .and_then(|attr| attr.as_str());

            let excerpt_separator = &self.front.excerpt_separator;

            if let Some(excerpt_str) = excerpt_attr {
                self.render_html(excerpt_str, context, parser, syntax_theme)?
            } else if excerpt_separator.is_empty() {
                self.render_html("", context, parser, syntax_theme)?
            } else {
                let excerpt = extract_excerpt(&self.content, self.front.format, excerpt_separator);
                self.render_html(&excerpt, context, parser, syntax_theme)?
            }
        };

        self.attributes
            .insert("excerpt".to_owned(), Value::Str(excerpt_html));
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
                  parser: &template::LiquidParser,
                  layouts_dir: &Path,
                  layouts_cache: &mut HashMap<String, String>,
                  syntax_theme: &str)
                  -> Result<String> {
        let content_html = self.render_html(&self.content, context, parser, syntax_theme)?;
        self.attributes
            .insert("content".to_owned(), Value::Str(content_html.clone()));
        context.set_val("content", Value::Str(content_html.clone()));

        if let Some(ref layout) = self.front.layout {
            let layout_data_ref = match layouts_cache.entry(layout.to_owned()) {
                Entry::Vacant(vacant) => {
                    let layout_data = files::read_file(layouts_dir.join(layout))
                        .map_err(|e| {
                                     format!("Layout {} can not be read (defined in {:?}): {}",
                                             layout,
                                             self.file_path,
                                             e)
                                 })?;
                    vacant.insert(layout_data)
                }
                Entry::Occupied(occupied) => occupied.into_mut(),
            };

            let template = parser.parse(layout_data_ref)?;
            Ok(template.render(context)?.unwrap_or_default())
        } else {
            Ok(content_html)
        }
    }

    pub fn render_dump(&self, dump: config::Dump) -> Result<(String, String)> {
        match dump {
            config::Dump::DocObject => {
                let content = serde_yaml::to_string(&self.attributes)?;
                Ok((content, "yml".to_owned()))
            }
            config::Dump::DocTemplate => Ok((self.content.clone(), "liquid".to_owned())),
            config::Dump::DocLinkObject => {
                let perma_attributes = permalink_attributes(&self.front, Path::new("<null>"));
                let content = serde_yaml::to_string(&perma_attributes)?;
                Ok((content, "yml".to_owned()))
            }
            config::Dump::Document => {
                let frontmatter = serde_yaml::to_string(&self.front)?;
                let content = self.content.clone();
                let ext = match self.front.format {
                    frontmatter::SourceFormat::Raw => "liquid",
                    frontmatter::SourceFormat::Markdown => "md",
                }.to_owned();
                let content = itertools::join(&[frontmatter, "---".to_owned(), content], "\n");
                Ok((content, ext))
            }
        }
    }
}

fn extract_excerpt_raw(content: &str, excerpt_separator: &str) -> String {
    content
        .split(excerpt_separator)
        .next()
        .unwrap_or(&content)
        .to_owned()
}

fn extract_excerpt_markdown(content: &str, excerpt_separator: &str) -> String {
    let mut trail = String::new();

    if MARKDOWN_REF.is_match(&content) {
        for mat in MARKDOWN_REF.find_iter(&content) {
            trail.push_str(mat.as_str());
            trail.push('\n');
        }
    }
    trail + content.split(excerpt_separator).next().unwrap_or(&content)
}

fn extract_excerpt(content: &str,
                   format: frontmatter::SourceFormat,
                   excerpt_separator: &str)
                   -> String {
    match format {
        frontmatter::SourceFormat::Markdown => extract_excerpt_markdown(content, excerpt_separator),
        frontmatter::SourceFormat::Raw => extract_excerpt_raw(content, excerpt_separator),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn split_document_empty() {
        let input = "";
        let (frontmatter, content) = split_document(input).unwrap();
        assert!(frontmatter.is_none());
        assert_eq!(content, "");
    }

    #[test]
    fn split_document_no_front_matter() {
        let input = "Body";
        let (frontmatter, content) = split_document(input).unwrap();
        assert!(frontmatter.is_none());
        assert_eq!(content, "Body");
    }

    #[test]
    fn split_document_empty_front_matter() {
        let input = "---\nBody";
        let (frontmatter, content) = split_document(input).unwrap();
        assert!(frontmatter.is_none());
        assert_eq!(content, "Body");
    }

    #[test]
    fn split_document_empty_body() {
        let input = "frontmatter---\n";
        let (frontmatter, content) = split_document(input).unwrap();
        assert_eq!(frontmatter.unwrap(), "frontmatter");
        assert_eq!(content, "");
    }

    #[test]
    fn format_path_variable_file() {
        let input = Path::new("/hello/world/file.liquid");
        let actual = format_path_variable(input);
        assert_eq!(actual, "hello/world");
    }

    #[test]
    fn format_path_variable_relative() {
        let input = Path::new("hello/world/file.liquid");
        let actual = format_path_variable(input);
        assert_eq!(actual, "hello/world");

        let input = Path::new("./hello/world/file.liquid");
        let actual = format_path_variable(input);
        assert_eq!(actual, "hello/world");
    }

    #[test]
    fn explode_permalink_relative() {
        let attributes = HashMap::new();
        let actual = explode_permalink("relative/path", attributes);
        assert_eq!(actual, "relative/path");
    }

    #[test]
    fn explode_permalink_absolute() {
        let attributes = HashMap::new();
        let actual = explode_permalink("/abs/path", attributes);
        assert_eq!(actual, "abs/path");
    }

    #[test]
    fn explode_permalink_blank_substitution() {
        let attributes = HashMap::new();
        let actual = explode_permalink("//path/middle//end", attributes);
        assert_eq!(actual, "path/middle/end");
    }

    #[test]
    fn format_url_as_file_absolute() {
        let actual = format_url_as_file("/hello/world.html");
        assert_eq!(actual, Path::new("hello/world.html"));
    }

    #[test]
    fn format_url_as_file_no_explode() {
        let actual = format_url_as_file("/hello/world.custom");
        assert_eq!(actual, Path::new("hello/world.custom"));
    }

    #[test]
    fn format_url_as_file_explode() {
        let actual = format_url_as_file("/hello/world");
        assert_eq!(actual, Path::new("hello/world/index.html"));
    }
}
