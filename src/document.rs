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

#[cfg(all(feature="syntax-highlight", not(windows)))]
use syntax_highlight::{initialize_codeblock, decorate_markdown};

use liquid::{Renderable, LiquidOptions, Context, Value, LocalTemplateRepository};

use frontmatter;
use datetime;
use pulldown_cmark as cmark;
use liquid;

lazy_static!{
    static ref DATE_VARIABLES: Regex =
        Regex::new(":(year|month|i_month|day|i_day|short_year|hour|minute|second)").unwrap();
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

/// Formats a user specified custom path, adding custom parameters
/// and "exploding" the URL.
fn format_path(p: &str,
               attributes: &HashMap<String, Value>,
               date: &Option<datetime::DateTime>)
               -> Result<String> {
    let mut p = p.to_owned();

    if DATE_VARIABLES.is_match(&p) {
        let date = try!(date.ok_or(format!("Can not format file path without a valid date ({:?})",
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
        p = match *val {
            Value::Str(ref v) => p.replace(&(String::from(":") + key), v),
            Value::Num(ref v) => p.replace(&(String::from(":") + key), &v.to_string()),
            _ => p,
        }
    }

    let trimmed = p.trim_left_matches('/');
    Ok(trimmed.to_owned())
}

fn format_permalink_path<S: AsRef<str>>(permalink: S) -> PathBuf {
    format_permalink_path_str(permalink.as_ref())
}

fn format_permalink_path_str(permalink: &str) -> PathBuf {
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
                 is_post: bool,
                 post_path: &Option<String>)
                 -> Result<Document> {
        let content = read_document(root_path, source_file)?;
        let (front, content) = split_document(&content)?;
        let mut attributes = front
            .map(|s| serde_yaml::from_str(s))
            .map_or(Ok(None), |r| r.map(Some))?
            .unwrap_or_else(|| liquid::Object::new());

        let front = frontmatter::FrontmatterBuilder::new()
            .merge_title(attributes
                             .get("title")
                             .and_then(|v| v.as_str())
                             .map(|s| s.to_owned()))
            .merge_slug(attributes
                            .get("slug")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_owned()))
            .merge_draft(attributes.get("draft").and_then(|v| v.as_bool()))
            .merge_post(attributes.get("is_post").and_then(|v| v.as_bool()))
            .merge_post(is_post)
            .merge_layout(attributes
                              .get("extends")
                              .and_then(|v| v.as_str())
                              .map(|s| s.to_owned()))
            .merge_published_date(attributes
                                      .get("date")
                                      .and_then(|d| d.as_str())
                                      .and_then(datetime::DateTime::parse))
            .merge_path(dest_file)?
            .build()?;

        attributes
            .entry("is_post".to_owned())
            .or_insert_with(|| Value::Bool(front.is_post));

        attributes
            .entry("title".to_owned())
            .or_insert_with(|| Value::str(&front.title));

        let mut path_buf = PathBuf::from(dest_file);
        path_buf.set_extension("html");

        // if the user specified a custom path override
        // format it and push it over the original file name
        // TODO replace "date", "pretty", "ordinal" and "none"
        // for Jekyl compatibility
        let mut url_path = path_buf
            .to_str()
            .ok_or_else(|| format!("Invalid path {:?}", path_buf))?
            .to_owned()
            .replace("\\", "/");
        if let Some(path) = attributes.get("path").and_then(|p| p.as_str()) {
            url_path = format_path(path, &attributes, &front.published_date)?;
            path_buf = format_permalink_path(&url_path);
        } else if is_post {
            // check if there is a global setting for post paths
            if let Some(ref path) = *post_path {
                url_path = format_path(path, &attributes, &front.published_date)?;
                path_buf = format_permalink_path(&url_path);
            }
        };

        // Swap back slashes to forward slashes to ensure the URL's are valid on Windows
        attributes.insert("path".to_owned(), Value::str(&url_path));

        Ok(Document::new(url_path, path_buf, content.to_string(), attributes, front))
    }


    /// Metadata for generating RSS feeds
    pub fn to_rss(&self, root_url: &str) -> rss::Item {
        let link = root_url.to_owned() + &self.url_path;
        let guid = rss::Guid {
            value: link.clone(),
            is_perma_link: true,
        };

        rss::Item {
            title: self.title_to_str(),
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
        let cat: Vec<_> = self.attributes
            .get("categories")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .iter()
            .map(|v| v.to_string())
            .collect();

        Item {
            id: link.clone(),
            url: Some(link),
            title: Some(self.title_to_str().unwrap_or("unknown title".into())),
            content: Content::Html(self.description_to_str().unwrap_or_else(|| "".into())),
            date_published: self.front.published_date.map(|date| date.to_rfc2822()),
            // TODO completely implement categories, see Issue 131
            tags: Some(cat),
            ..Default::default()
        }
    }

    fn title_to_str(&self) -> Option<String> {
        self.attributes
            .get("title")
            .and_then(|s| s.as_str())
            .map(|s| s.to_owned())
    }
    fn description_to_str(&self) -> Option<String> {
        self.attributes
            .get("description")
            .or_else(|| self.attributes.get("excerpt"))
            .or_else(|| self.attributes.get("content"))
            .and_then(|s| s.as_str())
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
    #[cfg(all(feature="syntax-highlight", not(windows)))]
    fn render_html(&self, content: &str, context: &mut Context, source: &Path) -> Result<String> {
        let mut options = LiquidOptions::default();
        options.template_repository = Box::new(LocalTemplateRepository::new(source.to_owned()));
        let highlight: Box<liquid::Block> = Box::new(initialize_codeblock);
        options.blocks.insert("highlight".to_string(), highlight);
        let template = try!(liquid::parse(content, options));
        let html = try!(template.render(context)).unwrap_or_default();

        let html = match self.front.format {
            frontmatter::SourceFormat::Raw => html,
            frontmatter::SourceFormat::Markdown => {
                let mut buf = String::new();
                let parser = cmark::Parser::new(&html);
                #[cfg(feature="syntax-highlight")]
                cmark::html::push_html(&mut buf, decorate_markdown(parser));
                #[cfg(not(feature="syntax-highlight"))]
                cmark::html::push_html(&mut buf, parser);
                buf
            }
        };
        Ok(html.to_owned())
    }
    #[cfg(any(not(feature="syntax-highlight"), windows))]
    fn render_html(&self, content: &str, context: &mut Context, source: &Path) -> Result<String> {
        let mut options = LiquidOptions::default();
        options.template_repository = Box::new(LocalTemplateRepository::new(source.to_owned()));
        let template = liquid::parse(content, options)?;
        let html = template.render(context)?.unwrap_or_default();

        let html = match self.front.format {
            frontmatter::SourceFormat::Raw => html,
            frontmatter::SourceFormat::Markdown => {
                let mut buf = String::new();
                let parser = cmark::Parser::new(&html);
                cmark::html::push_html(&mut buf, parser);
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
                  layouts_cache: &mut HashMap<String, String>)
                  -> Result<String> {
        let content_html = try!(self.render_html(&self.content, context, source));
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
    fn format_permalink_path_absolute() {
        let actual = format_permalink_path("/hello/world.html");
        assert_eq!(actual, Path::new("hello/world.html"));
    }

    #[test]
    fn format_permalink_path_no_explode() {
        let actual = format_permalink_path("/hello/world.custom");
        assert_eq!(actual, Path::new("hello/world.custom"));
    }

    #[test]
    fn format_permalink_path_explode() {
        let actual = format_permalink_path("/hello/world");
        assert_eq!(actual, Path::new("hello/world/index.html"));
    }
}
