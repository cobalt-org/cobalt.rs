use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::default::Default;
use std::path::{Path, PathBuf};

use chrono::{Datelike, Timelike};
use itertools;
use jsonfeed;
use liquid;
use liquid::Value;
use pulldown_cmark as cmark;
use regex::Regex;
use rss;
use serde_yaml;

use error::*;
use cobalt_model::files;
use cobalt_model::slug;
use cobalt_model;
use syntax_highlight::decorate_markdown;
use template;

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

fn permalink_attributes(front: &cobalt_model::Frontmatter, dest_file: &Path) -> liquid::Object {
    let mut attributes = liquid::Object::new();

    attributes.insert("parent".to_owned(),
                      Value::Str(format_path_variable(dest_file)));

    let filename = dest_file.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    attributes.insert("name".to_owned(), Value::str(filename));

    attributes.insert("ext".to_owned(), Value::str(".html"));

    // TODO(epage): Add `collection` (the collection's slug), see #257
    // or `parent.slug`, see #323

    attributes.insert("slug".to_owned(), Value::str(&front.slug));

    attributes.insert("categories".to_owned(),
                      Value::Str(itertools::join(front.categories.iter().map(slug::slugify), "/")));

    if let Some(ref date) = front.published_date {
        attributes.insert("year".to_owned(), Value::Str(date.year().to_string()));
        attributes.insert("month".to_owned(),
                          Value::Str(format!("{:02}", &date.month())));
        attributes.insert("i_month".to_owned(), Value::Str(date.month().to_string()));
        attributes.insert("day".to_owned(), Value::Str(format!("{:02}", &date.day())));
        attributes.insert("i_day".to_owned(), Value::Str(date.day().to_string()));
        attributes.insert("hour".to_owned(),
                          Value::Str(format!("{:02}", &date.hour())));
        attributes.insert("minute".to_owned(),
                          Value::Str(format!("{:02}", &date.minute())));
        attributes.insert("second".to_owned(),
                          Value::Str(format!("{:02}", &date.second())));
    }

    attributes.insert("data".to_owned(), Value::Object(front.data.clone()));

    attributes
}

fn explode_permalink<S: AsRef<str>>(permalink: S, attributes: &liquid::Object) -> Result<String> {
    explode_permalink_string(permalink.as_ref(), attributes)
}

fn explode_permalink_string(permalink: &str, attributes: &liquid::Object) -> Result<String> {
    lazy_static!{
       static ref PERMALINK_PARSER: liquid::Parser = liquid::Parser::new();
    }
    let p = PERMALINK_PARSER.parse(permalink)?;
    let mut p = p.render(attributes)?;

    // Handle the user doing windows-style
    p = p.replace("\\", "/");

    // Handle cases where substutions were blank
    p = p.replace("//", "/");

    if p.starts_with('/') {
        p.remove(0);
    }

    Ok(p)
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

fn document_attributes(front: &cobalt_model::Frontmatter,
                       source_file: &Path,
                       url_path: &str)
                       -> liquid::Object {
    let categories = liquid::Value::Array(front
                                              .categories
                                              .iter()
                                              .map(|c| liquid::Value::str(c))
                                              .collect());
    // Reason for `file`:
    // - Allow access to assets in the original location
    // - Ease linking back to page's source
    let file: liquid::Object =
        vec![("permalink".to_owned(), liquid::Value::str(source_file.to_str().unwrap_or(""))),
             ("parent".to_owned(),
              liquid::Value::str(source_file.parent().and_then(Path::to_str).unwrap_or("")))]
            .into_iter()
            .collect();
    let attributes =
        vec![("permalink".to_owned(), liquid::Value::str(url_path)),
             ("title".to_owned(), liquid::Value::str(&front.title)),
             ("description".to_owned(),
              liquid::Value::str(front.description.as_ref().map(|s| s.as_str()).unwrap_or(""))),
             ("categories".to_owned(), categories),
             ("is_draft".to_owned(), liquid::Value::Bool(front.is_draft)),
             ("file".to_owned(), liquid::Value::Object(file)),
             ("collection".to_owned(), liquid::Value::str(&front.collection)),
             ("data".to_owned(), liquid::Value::Object(front.data.clone()))];
    let mut attributes: liquid::Object = attributes.into_iter().collect();

    if let Some(ref published_date) = front.published_date {
        attributes.insert("published_date".to_owned(),
                          liquid::Value::Str(published_date.format()));
    }

    attributes
}

#[derive(Debug)]
pub struct Document {
    pub url_path: String,
    pub file_path: PathBuf,
    pub content: String,
    pub attributes: liquid::Object,
    pub front: cobalt_model::Frontmatter,
}

impl Document {
    pub fn new(url_path: String,
               file_path: PathBuf,
               content: String,
               attributes: liquid::Object,
               front: cobalt_model::Frontmatter)
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
                 default_front: cobalt_model::FrontmatterBuilder)
                 -> Result<Document> {
        trace!("Parsing {:?}", rel_path);
        let content = files::read_file(src_path)?;
        let builder =
            cobalt_model::DocumentBuilder::<cobalt_model::FrontmatterBuilder>::parse(&content)?;
        let (front, content) = builder.parts();
        let front = front.merge_path(rel_path).merge(default_front);

        let front = front.build()?;

        let (file_path, url_path) = {
            let perma_attributes = permalink_attributes(&front, rel_path);
            let url_path =
                explode_permalink(&front.permalink, &perma_attributes)
                    .chain_err(|| format!("Failed to create permalink `{}`", front.permalink))?;
            let file_path = format_url_as_file(&url_path);
            (file_path, url_path)
        };

        let doc_attributes = document_attributes(&front, rel_path, url_path.as_ref());

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

    /// Renders liquid templates into HTML in the context of current document.
    ///
    /// Takes `content` string and returns rendered HTML. This function doesn't
    /// take `"extends"` attribute into account. This function can be used for
    /// rendering content or excerpt.
    fn render_html(&self,
                   content: &str,
                   globals: &liquid::Object,
                   parser: &template::LiquidParser,
                   syntax_theme: &str)
                   -> Result<String> {
        let template = parser.parse(content)?;
        let html = template.render(globals)?;

        let html = match self.front.format {
            cobalt_model::SourceFormat::Raw => html,
            cobalt_model::SourceFormat::Markdown => {
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
                          globals: &liquid::Object,
                          parser: &template::LiquidParser,
                          syntax_theme: &str)
                          -> Result<()> {
        let value = if let Some(excerpt_str) = self.front.excerpt.as_ref() {
            let excerpt = self.render_html(excerpt_str, globals, parser, syntax_theme)?;
            Value::Str(excerpt)
        } else if self.front.excerpt_separator.is_empty() {
            Value::Nil
        } else {
            let excerpt = extract_excerpt(&self.content,
                                          self.front.format,
                                          &self.front.excerpt_separator);
            let excerpt = self.render_html(&excerpt, globals, parser, syntax_theme)?;
            Value::Str(excerpt)
        };

        self.attributes.insert("excerpt".to_owned(), value);
        Ok(())
    }

    /// Renders the content and adds it to attributes of the document.
    ///
    /// When we say "content" we mean only this document without extended layout.
    pub fn render_content(&mut self,
                          globals: &liquid::Object,
                          parser: &template::LiquidParser,
                          syntax_theme: &str)
                          -> Result<()> {
        let content_html = self.render_html(&self.content, globals, parser, syntax_theme)?;
        self.attributes
            .insert("content".to_owned(), Value::Str(content_html.clone()));
        Ok(())
    }

    /// Renders the document to an HTML string.
    ///
    /// Side effects:
    ///
    /// * layout may be inserted to layouts cache
    pub fn render(&mut self,
                  globals: &liquid::Object,
                  parser: &template::LiquidParser,
                  layouts_dir: &Path,
                  layouts_cache: &mut HashMap<String, String>)
                  -> Result<String> {
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

            let template = parser
                .parse(layout_data_ref)
                .chain_err(|| format!("Failed to parse layout {:?}", layout))?;
            let content_html = template
                .render(globals)
                .chain_err(|| format!("Failed to render layout {:?}", layout))?;
            Ok(content_html)
        } else {
            let content_html = globals
                .get("page")
                .ok_or("Internal error: page isn't in globals")?
                .get(&liquid::Index::with_key("content"))
                .ok_or("Internal error: page.content isn't in globals")?
                .as_str()
                .ok_or("Internal error: bad content format")?
                .to_owned();

            Ok(content_html)
        }
    }

    pub fn render_dump(&self, dump: cobalt_model::Dump) -> Result<(String, String)> {
        match dump {
            cobalt_model::Dump::DocObject => {
                let content = serde_yaml::to_string(&self.attributes)?;
                Ok((content, "yml".to_owned()))
            }
            cobalt_model::Dump::DocTemplate => Ok((self.content.clone(), "liquid".to_owned())),
            cobalt_model::Dump::DocLinkObject => {
                let perma_attributes = permalink_attributes(&self.front, Path::new("<null>"));
                let content = serde_yaml::to_string(&perma_attributes)?;
                Ok((content, "yml".to_owned()))
            }
            cobalt_model::Dump::Document => {
                let cobalt_model = serde_yaml::to_string(&self.front)?;
                let content = self.content.clone();
                let ext = match self.front.format {
                    cobalt_model::SourceFormat::Raw => "liquid",
                    cobalt_model::SourceFormat::Markdown => "md",
                }.to_owned();
                let content = itertools::join(&[cobalt_model, "---".to_owned(), content], "\n");
                Ok((content, ext))
            }
        }
    }
}

fn extract_excerpt_raw(content: &str, excerpt_separator: &str) -> String {
    content
        .split(excerpt_separator)
        .next()
        .unwrap_or(content)
        .to_owned()
}

fn extract_excerpt_markdown(content: &str, excerpt_separator: &str) -> String {
    lazy_static!{
       static ref MARKDOWN_REF: Regex = Regex::new(r"(?m:^ {0,3}\[[^\]]+\]:.+$)").unwrap();
    }

    let mut trail = String::new();

    if MARKDOWN_REF.is_match(content) {
        for mat in MARKDOWN_REF.find_iter(content) {
            trail.push_str(mat.as_str());
            trail.push('\n');
        }
    }
    trail + content.split(excerpt_separator).next().unwrap_or(content)
}

fn extract_excerpt(content: &str,
                   format: cobalt_model::SourceFormat,
                   excerpt_separator: &str)
                   -> String {
    match format {
        cobalt_model::SourceFormat::Markdown => {
            extract_excerpt_markdown(content, excerpt_separator)
        }
        cobalt_model::SourceFormat::Raw => extract_excerpt_raw(content, excerpt_separator),
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
        let attributes = liquid::Object::new();
        let actual = explode_permalink("relative/path", &attributes).unwrap();
        assert_eq!(actual, "relative/path");
    }

    #[test]
    fn explode_permalink_absolute() {
        let attributes = liquid::Object::new();
        let actual = explode_permalink("/abs/path", &attributes).unwrap();
        assert_eq!(actual, "abs/path");
    }

    #[test]
    fn explode_permalink_blank_substitution() {
        let attributes = liquid::Object::new();
        let actual = explode_permalink("//path/middle//end", &attributes).unwrap();
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
