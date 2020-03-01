use std::clone::Clone;
use std::collections::HashMap;
use std::default::Default;
use std::path::{Path, PathBuf};

use failure::ResultExt;
use jsonfeed;
use liquid;
use liquid::value::Object;
use liquid::value::Value;
use regex::Regex;
use rss;

use crate::cobalt_model::Liquid;
use crate::cobalt_model::Markdown;
use crate::error::*;

fn document_attributes(
    front: &cobalt_model::page::Frontmatter,
    source_file: &Path,
    url_path: &str,
) -> Object {
    let categories = Value::Array(
        front
            .categories
            .iter()
            .cloned()
            .map(Value::scalar)
            .collect(),
    );
    // Reason for `file`:
    // - Allow access to assets in the original location
    // - Ease linking back to page's source
    let file: Object = vec![
        (
            "permalink".into(),
            Value::scalar(source_file.to_str().unwrap_or("").to_owned()),
        ),
        (
            "parent".into(),
            Value::scalar(
                source_file
                    .parent()
                    .and_then(Path::to_str)
                    .unwrap_or("")
                    .to_owned(),
            ),
        ),
    ]
    .into_iter()
    .collect();
    let attributes = vec![
        ("permalink".into(), Value::scalar(url_path.to_owned())),
        ("title".into(), Value::scalar(front.title.clone())),
        ("slug".into(), Value::scalar(front.slug.clone())),
        (
            "description".into(),
            Value::scalar(front.description.as_deref().unwrap_or("").to_owned()),
        ),
        ("categories".into(), categories),
        ("is_draft".into(), Value::scalar(front.is_draft)),
        ("weight".into(), Value::scalar(front.weight)),
        ("file".into(), Value::Object(file)),
        ("collection".into(), Value::scalar(front.collection.clone())),
        ("data".into(), Value::Object(front.data.clone())),
    ];
    let mut attributes: Object = attributes.into_iter().collect();

    if let Some(ref tags) = front.tags {
        let tags = Value::Array(tags.iter().cloned().map(Value::scalar).collect());
        attributes.insert("tags".into(), tags);
    }

    if let Some(ref published_date) = front.published_date {
        attributes.insert(
            "published_date".into(),
            Value::scalar(liquid::value::Date::from(*published_date)),
        );
    }

    attributes
}

#[derive(Debug, Clone)]
pub struct Document {
    pub url_path: String,
    pub src_path: PathBuf,
    pub dest_path: PathBuf,
    pub rel_path: PathBuf,
    pub content: String,
    pub attributes: Object,
    pub front: cobalt_model::page::Frontmatter,
    pub pagination: Option<cobalt_model::page::Pagination>,
}

impl Document {
    pub fn parse(
        src_path: &Path,
        rel_path: &Path,
        dest_root: &Path,
        default_front: &cobalt_config::Frontmatter,
    ) -> Result<Document> {
        trace!("Parsing {:?}", rel_path);
        let (front, pagination, content) =
            cobalt_model::page::derive_component(src_path, rel_path, default_front)?;
        let url = cobalt_model::url::derive_page_url(&front, rel_path)?;
        let dest = cobalt_model::url::derive_dest(dest_root, &url);

        let doc_attributes = document_attributes(&front, rel_path, &url.url);

        Ok(Document {
            url_path: url.url,
            src_path: src_path.to_owned(),
            dest_path: dest.fs_path,
            rel_path: rel_path.to_owned(),
            content: content.content,
            attributes: doc_attributes,
            front,
            pagination,
        })
    }

    /// Metadata for generating RSS feeds
    pub fn to_rss(&self, root_url: &str) -> Result<rss::Item> {
        let link = format!("{}/{}", root_url, &self.url_path);
        let guid = rss::GuidBuilder::default()
            .value(link.clone())
            .permalink(true)
            .build()
            .map_err(failure::err_msg)?;

        let item = rss::ItemBuilder::default()
            .title(Some(self.front.title.clone()))
            .link(Some(link))
            .guid(Some(guid))
            .pub_date(self.front.published_date.map(|date| date.to_rfc2822()))
            .description(self.description_to_str())
            .build()
            .map_err(failure::err_msg)?;
        Ok(item)
    }

    /// Metadata for generating JSON feeds
    pub fn to_jsonfeed(&self, root_url: &str) -> jsonfeed::Item {
        let link = format!("{}/{}", root_url, &self.url_path);

        jsonfeed::Item {
            id: link.clone(),
            url: Some(link),
            title: Some(self.front.title.clone()),
            content: jsonfeed::Content::Html(
                self.description_to_str().unwrap_or_else(|| "".into()),
            ),
            date_published: self.front.published_date.map(|date| date.to_rfc2822()),
            // TODO completely implement categories, see Issue 131
            tags: Some(self.front.categories.clone()),
            ..Default::default()
        }
    }

    pub fn to_sitemap<T: std::io::Write>(
        &self,
        root_url: &str,
        writer: &mut sitemap::writer::UrlSetWriter<T>,
    ) -> Result<()> {
        let link = format!("{}/{}", root_url, &self.url_path);
        let mut url = sitemap::structs::UrlEntry::builder();
        url = url.loc(link);
        if let Some(date) = self.front.published_date {
            url = url.lastmod(*date);
        }
        writer.url(url)?;
        Ok(())
    }

    fn description_to_str(&self) -> Option<String> {
        self.front
            .description
            .clone()
            .or_else(|| {
                self.attributes.get("excerpt").and_then(|excerpt| {
                    if excerpt.is_nil() {
                        None
                    } else {
                        Some(excerpt.render().to_string())
                    }
                })
            })
            .or_else(|| {
                self.attributes
                    .get("content")
                    .map(|s| s.render().to_string())
            })
    }

    /// Renders liquid templates into HTML in the context of current document.
    ///
    /// Takes `content` string and returns rendered HTML. This function doesn't
    /// take `"extends"` attribute into account. This function can be used for
    /// rendering content or excerpt.
    fn render_html(
        &self,
        content: &str,
        globals: &Object,
        parser: &Liquid,
        markdown: &Markdown,
    ) -> Result<String> {
        let template = parser.parse(content)?;
        let html = template.render(globals)?;

        let html = match self.front.format {
            cobalt_config::SourceFormat::Raw => html,
            cobalt_config::SourceFormat::Markdown => markdown.parse(&html)?,
        };
        Ok(html)
    }

    /// Renders the excerpt and adds it to attributes of the document.
    ///
    /// The excerpt is either taken from the `excerpt` frontmatter setting, if
    /// given, or extracted from the content, if `excerpt_separator` is not
    /// empty. When neither condition applies, the excerpt is set to the `Nil`
    /// value.
    pub fn render_excerpt(
        &mut self,
        globals: &Object,
        parser: &Liquid,
        markdown: &Markdown,
    ) -> Result<()> {
        let value = if let Some(excerpt_str) = self.front.excerpt.as_ref() {
            let excerpt = self.render_html(excerpt_str, globals, parser, markdown)?;
            Value::scalar(excerpt)
        } else if self.front.excerpt_separator.is_empty() {
            Value::Nil
        } else {
            let excerpt = extract_excerpt(
                &self.content,
                self.front.format,
                &self.front.excerpt_separator,
            );
            let excerpt = self.render_html(&excerpt, globals, parser, markdown)?;
            Value::scalar(excerpt)
        };

        self.attributes.insert("excerpt".into(), value);
        Ok(())
    }

    /// Renders the content and adds it to attributes of the document.
    ///
    /// When we say "content" we mean only this document without extended layout.
    pub fn render_content(
        &mut self,
        globals: &Object,
        parser: &Liquid,
        markdown: &Markdown,
    ) -> Result<()> {
        let content_html = self.render_html(&self.content, globals, parser, markdown)?;
        self.attributes
            .insert("content".into(), Value::scalar(content_html));
        Ok(())
    }

    /// Renders the document to an HTML string.
    ///
    /// Side effects:
    ///
    /// * layout may be inserted to layouts cache
    pub fn render(
        &mut self,
        globals: &Object,
        parser: &Liquid,
        layouts: &HashMap<String, String>,
    ) -> Result<String> {
        if let Some(ref layout) = self.front.layout {
            let layout_data_ref = layouts.get(layout).ok_or_else(|| {
                failure::format_err!(
                    "Layout {} does not exist (referenced in {}).",
                    layout,
                    self.src_path.display()
                )
            })?;

            let template = parser
                .parse(layout_data_ref)
                .with_context(|_| failure::format_err!("Failed to parse layout {:?}", layout))?;
            let content_html = template
                .render(globals)
                .with_context(|_| failure::format_err!("Failed to render layout {:?}", layout))?;
            Ok(content_html)
        } else {
            let content_html = globals
                .get("page")
                .ok_or_else(|| failure::err_msg("Internal error: page isn't in globals"))?
                .get(&liquid::value::Scalar::new("content"))
                .ok_or_else(|| failure::err_msg("Internal error: page.content isn't in globals"))?
                .render()
                .to_string();

            Ok(content_html)
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
    lazy_static! {
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

fn extract_excerpt(
    content: &str,
    format: cobalt_config::SourceFormat,
    excerpt_separator: &str,
) -> String {
    match format {
        cobalt_config::SourceFormat::Markdown => {
            extract_excerpt_markdown(content, excerpt_separator)
        }
        cobalt_config::SourceFormat::Raw => extract_excerpt_raw(content, excerpt_separator),
    }
}
