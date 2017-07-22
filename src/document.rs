use std::fs::File;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::{Path, PathBuf};
use std::default::Default;
use error::Result;
use chrono::{Datelike, Timelike};
use std::io::Read;
use regex::Regex;
use rss;
use jsonfeed;
use jsonfeed::Item;
use jsonfeed::Content;
use serde_yaml;
use itertools;

use syntax_highlight::{initialize_codeblock, decorate_markdown};

use liquid::{Renderable, LiquidOptions, Context, Value, LocalTemplateRepository};

use config;
use frontmatter;
use datetime;
use pulldown_cmark as cmark;
use liquid;

lazy_static!{
    static ref FRONT_MATTER_DIVIDE: Regex = Regex::new(r"---\s*\r?\n").unwrap();
    static ref MARKDOWN_REF: Regex = Regex::new(r"(?m:^ {0,3}\[[^\]]+\]:.+$)").unwrap();
}

fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = File::open(path.as_ref())?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    Ok(text)
}

fn read_document<PB: Into<PathBuf>, P: AsRef<Path>>(root: PB, relpath: P) -> Result<String> {
    let path = root.into().join(relpath);
    let mut file = File::open(path)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    Ok(text)
}

fn split_document(content: &str) -> Result<(Option<&str>, &str)> {
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
    // TODO replace "date", "pretty", "ordinal" and "none"
    // for Jekyl compatibility

    let mut attributes = HashMap::new();

    attributes.insert(":path".to_owned(), format_path_variable(dest_file));

    let filename = dest_file.file_stem().and_then(|s| s.to_str());
    if let Some(filename) = filename {
        attributes.insert(":filename".to_owned(), filename.to_owned());
    }

    attributes.insert(":slug".to_owned(), front.slug.clone());

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
        attributes.insert("date".to_owned(),
                          liquid::Value::Str(published_date.format()));
    }
    attributes.insert("draft".to_owned(), liquid::Value::Bool(front.is_draft));
    attributes.insert("is_post".to_owned(), liquid::Value::Bool(front.is_post));

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

    pub fn parse(root_path: &Path,
                 source_file: &Path,
                 dest_file: &Path,
                 default_front: frontmatter::FrontmatterBuilder)
                 -> Result<Document> {
        trace!("Parsing {:?}", source_file);
        let content = read_document(root_path, source_file)?;
        let (front, content) = split_document(&content)?;
        let mut custom_attributes = front
            .map(|s| serde_yaml::from_str(s))
            .map_or(Ok(None), |r| r.map(Some))?
            .unwrap_or_else(liquid::Object::new);

        // Convert legacy frontmatter into frontmatter (with `custom`)
        // In some cases, we need to remove them to successfully run perma_attributes
        // Otherwise, we can remove the converted values because most frontmatter content gets
        // populated into the final attributes (see `document_attributes`).
        // Exceptions
        // - excerpt_separator: internal-only
        // - extends internal-only
        let mut front = frontmatter::FrontmatterBuilder::new()
            .merge_title(custom_attributes
                             .remove("title")
                             .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_description(custom_attributes
                                   .remove("description")
                                   .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_categories(custom_attributes
                                  .remove("categories")
                                  .and_then(|v| {
                                                v.as_array()
                                                    .map(|v| {
                                                             v.iter()
                                                                 .map(|v| v.to_string())
                                                                 .collect()
                                                         })
                                            }))
            .merge_slug(custom_attributes
                            .remove("slug")
                            .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_permalink(custom_attributes
                                 .remove("path")
                                 .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_draft(custom_attributes
                             .remove("draft")
                             .and_then(|v| v.as_bool()))
            .merge_excerpt_separator(custom_attributes
                                         .remove("excerpt_separator")
                                         .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_layout(custom_attributes
                              .remove("extends")
                              .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_published_date(custom_attributes
                                      .remove("date")
                                      .and_then(|d| {
                                                    d.as_str().and_then(datetime::DateTime::parse)
                                                }))
            .merge_path(dest_file)?
            .merge(default_front);

        front = front.merge_custom(custom_attributes);

        let front = front.build()?;

        let perma_attributes = permalink_attributes(&front, dest_file);
        let (file_path, url_path) = {
            let permalink = front.path.as_ref();
            let url_path = explode_permalink(permalink, perma_attributes);
            let file_path = format_url_as_file(&url_path);
            (file_path, url_path)
        };

        let doc_attributes = document_attributes(&front,
                                                 source_file.to_str().unwrap_or(""),
                                                 url_path.as_ref());

        Ok(Document::new(url_path,
                         file_path,
                         content.to_string(),
                         doc_attributes,
                         front))
    }


    /// Metadata for generating RSS feeds
    pub fn to_rss(&self, root_url: &str) -> rss::Item {
        let link = root_url.to_owned() + &self.url_path;
        let guid = rss::Guid {
            value: link.clone(),
            is_perma_link: true,
        };

        rss::Item {
            title: Some(self.front.title.clone()),
            link: Some(link),
            guid: Some(guid),
            pub_date: self.front.published_date.map(|date| date.to_rfc2822()),
            description: self.description_to_str(),
            ..Default::default()
        }
    }

    /// Metadata for generating JSON feeds
    pub fn to_jsonfeed(&self, root_url: &str) -> jsonfeed::Item {
        let link = root_url.to_owned() + &self.url_path;

        Item {
            id: link.clone(),
            url: Some(link),
            title: Some(self.front.title.clone()),
            content: Content::Html(self.description_to_str().unwrap_or_else(|| "".into())),
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
    fn render_html(&self,
                   content: &str,
                   context: &mut Context,
                   source: &Path,
                   syntax_theme: &str)
                   -> Result<String> {
        let mut options = LiquidOptions::default();
        options.template_repository = Box::new(LocalTemplateRepository::new(source.to_owned()));
        let highlight: Box<liquid::Block> = {
            let syntax_theme = syntax_theme.to_owned();
            Box::new(move |_, args, tokens, _| initialize_codeblock(args, tokens, &syntax_theme))
        };
        options.blocks.insert("highlight".to_string(), highlight);
        let template = try!(liquid::parse(content, options));
        let html = try!(template.render(context)).unwrap_or_default();

        let html = match self.front.format {
            frontmatter::SourceFormat::Raw => html,
            frontmatter::SourceFormat::Markdown => {
                let mut buf = String::new();
                let parser = cmark::Parser::new(&html);
                cmark::html::push_html(&mut buf, decorate_markdown(parser, syntax_theme));
                buf
            }
        };
        Ok(html.to_owned())
    }

    /// Extracts references iff markdown content.
    pub fn extract_markdown_references(&self, excerpt_separator: &str) -> String {
        let mut trail = String::new();

        if self.front.format == frontmatter::SourceFormat::Markdown &&
           MARKDOWN_REF.is_match(&self.content) {
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
                          syntax_theme: &str)
                          -> Result<()> {
        let excerpt_html = {
            let excerpt_attr = self.attributes
                .get("excerpt")
                .and_then(|attr| attr.as_str());

            let excerpt_separator = &self.front.excerpt_separator;

            if let Some(excerpt_str) = excerpt_attr {
                try!(self.render_html(excerpt_str, context, source, syntax_theme))
            } else if excerpt_separator.is_empty() {
                try!(self.render_html("", context, source, syntax_theme))
            } else {
                try!(self.render_html(&self.extract_markdown_references(excerpt_separator),
                                      context,
                                      source,
                                      syntax_theme))
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
                  source: &Path,
                  layouts_dir: &Path,
                  layouts_cache: &mut HashMap<String, String>,
                  syntax_theme: &str)
                  -> Result<String> {
        let content_html = try!(self.render_html(&self.content, context, source, syntax_theme));
        self.attributes
            .insert("content".to_owned(), Value::Str(content_html.clone()));
        context.set_val("content", Value::Str(content_html.clone()));

        if let Some(ref layout) = self.front.layout {
            let layout_data_ref = match layouts_cache.entry(layout.to_owned()) {
                Entry::Vacant(vacant) => {
                    let layout_data = try!(read_file(layouts_dir.join(layout)).map_err(|e| {
                        format!("Layout {} can not be read (defined in {:?}): {}",
                                layout,
                                self.file_path,
                                e)
                    }));
                    vacant.insert(layout_data)
                }
                Entry::Occupied(occupied) => occupied.into_mut(),
            };

            let mut options = LiquidOptions::default();
            options.template_repository =
                Box::new(LocalTemplateRepository::new(source.to_owned()));
            let template = try!(liquid::parse(layout_data_ref, options));
            Ok(try!(template.render(context)).unwrap_or_default())
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
                    }
                    .to_owned();
                let content = itertools::join(&[frontmatter, "---".to_owned(), content], "\n");
                Ok((content, ext))
            }
        }
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
