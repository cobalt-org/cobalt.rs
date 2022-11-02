use std::clone::Clone;
use std::collections::HashMap;
use std::default::Default;
use std::path::Path;

use failure::ResultExt;
use lazy_static::lazy_static;
use liquid::model::Value;
use liquid::Object;
use liquid::ValueView;
use log::trace;
use regex::Regex;

use crate::cobalt_model;
use crate::cobalt_model::files;
use crate::cobalt_model::permalink;
use crate::cobalt_model::slug;
use crate::cobalt_model::Minify;
use crate::error::*;

pub struct RenderContext<'a> {
    pub parser: &'a cobalt_model::Liquid,
    pub markdown: &'a cobalt_model::Markdown,
    pub vimwiki: &'a cobalt_model::Vimwiki,
    pub globals: &'a Object,
    pub minify: Minify,
}

#[cfg(not(feature = "html-minifier"))]
fn minify_if_enabled(
    html: String,
    _context: &RenderContext,
    _file_path: &relative_path::RelativePath,
) -> Result<String> {
    Ok(html)
}

#[cfg(feature = "html-minifier")]
fn minify_if_enabled(
    html: String,
    context: &RenderContext<'_>,
    file_path: &relative_path::RelativePath,
) -> Result<String> {
    let extension = file_path.extension().unwrap_or_default();
    if context.minify.html && (extension == "html" || extension == "htm") {
        Ok(html_minifier::minify(html)?)
    } else {
        Ok(html)
    }
}

pub fn permalink_attributes(
    front: &cobalt_model::Frontmatter,
    dest_file: &relative_path::RelativePath,
) -> Object {
    let mut attributes = Object::new();

    attributes.insert(
        "parent".into(),
        Value::scalar(
            dest_file
                .parent()
                .unwrap_or_else(|| relative_path::RelativePath::new(""))
                .to_string(),
        ),
    );

    let filename = dest_file.file_stem().unwrap_or("").to_owned();
    attributes.insert("name".into(), Value::scalar(filename));

    attributes.insert("ext".into(), Value::scalar(".html"));

    // TODO(epage): Add `collection` (the collection's slug), see #257
    // or `parent.slug`, see #323

    attributes.insert("slug".into(), Value::scalar(front.slug.clone()));

    attributes.insert(
        "categories".into(),
        Value::scalar(itertools::join(
            front.categories.iter().map(slug::slugify),
            "/",
        )),
    );

    if let Some(ref date) = front.published_date {
        attributes.insert("year".into(), Value::scalar(date.year().to_string()));
        attributes.insert(
            "month".into(),
            Value::scalar(format!("{:02}", &date.month())),
        );
        attributes.insert("i_month".into(), Value::scalar(date.month().to_string()));
        attributes.insert("day".into(), Value::scalar(format!("{:02}", &date.day())));
        attributes.insert("i_day".into(), Value::scalar(date.day().to_string()));
        attributes.insert("hour".into(), Value::scalar(format!("{:02}", &date.hour())));
        attributes.insert(
            "minute".into(),
            Value::scalar(format!("{:02}", &date.minute())),
        );
        attributes.insert(
            "second".into(),
            Value::scalar(format!("{:02}", &date.second())),
        );
    }

    attributes.insert("data".into(), Value::Object(front.data.clone()));

    attributes
}

fn document_attributes(
    front: &cobalt_model::Frontmatter,
    source_file: &relative_path::RelativePath,
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
            Value::scalar(source_file.as_str().to_owned()),
        ),
        (
            "parent".into(),
            Value::scalar(
                source_file
                    .parent()
                    .map(relative_path::RelativePath::as_str)
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
        attributes.insert("published_date".into(), Value::scalar(*published_date));
    }

    attributes
}

#[derive(Debug, Clone)]
pub struct Document {
    pub url_path: String,
    pub file_path: relative_path::RelativePathBuf,
    pub content: liquid::model::KString,
    pub attributes: Object,
    pub front: cobalt_model::Frontmatter,
}

impl Document {
    pub fn parse(
        src_path: &Path,
        rel_path: &relative_path::RelativePath,
        default_front: cobalt_config::Frontmatter,
    ) -> Result<Document> {
        trace!("Parsing {:?}", rel_path);
        let content = files::read_file(src_path)?;
        let builder = cobalt_config::Document::parse(&content)?;
        let (front, content) = builder.into_parts();
        let front = front.merge_path(rel_path).merge(&default_front);

        let front = cobalt_model::Frontmatter::from_config(front)?;

        let (file_path, url_path) = {
            let perma_attributes = permalink_attributes(&front, rel_path);
            let url_path =
                permalink::explode_permalink(front.permalink.as_str(), &perma_attributes)
                    .with_context(|_| {
                        failure::format_err!("Failed to create permalink `{}`", front.permalink)
                    })?;
            let file_path = permalink::format_url_as_file(&url_path);
            (file_path, url_path)
        };

        let doc_attributes = document_attributes(&front, rel_path, url_path.as_ref());

        Ok(Document {
            url_path,
            file_path,
            content,
            attributes: doc_attributes,
            front,
        })
    }

    /// Metadata for generating RSS feeds
    pub fn to_rss(&self, root_url: &str) -> Result<rss::Item> {
        let link = format!("{}/{}", root_url, &self.url_path);
        let guid = rss::GuidBuilder::default()
            .value(link.clone())
            .permalink(true)
            .build();

        let item = rss::ItemBuilder::default()
            .title(Some(self.front.title.as_str().to_owned()))
            .link(Some(link))
            .guid(Some(guid))
            .pub_date(self.front.published_date.map(|date| date.to_rfc2822()))
            .description(self.description_to_str())
            .build();
        Ok(item)
    }

    /// Metadata for generating JSON feeds
    pub fn to_jsonfeed(&self, root_url: &str) -> jsonfeed::Item {
        let link = format!("{}/{}", root_url, &self.url_path);

        jsonfeed::Item {
            id: link.clone(),
            url: Some(link),
            title: Some(self.front.title.as_str().to_owned()),
            content: jsonfeed::Content::Html(
                self.description_to_str().unwrap_or_else(|| "".into()),
            ),
            date_published: self.front.published_date.map(|date| date.to_rfc2822()),
            // TODO completely implement categories, see Issue 131
            tags: Some(
                self.front
                    .categories
                    .iter()
                    .map(|s| s.as_str().to_owned())
                    .collect(),
            ),
            ..Default::default()
        }
    }

    // Metadata for generating Atom feeds
    pub fn to_atom(&self, root_url: &str) -> Result<atom_syndication::Entry> {
        let link = format!("{}/{}", root_url, &self.url_path);

        // Try updated_date first; then try published_date
        let updated = self.front
            .updated_date
            .or(self.front.published_date)
            .map(|date|
                atom_syndication::FixedDateTime::parse_from_rfc2822(&date.to_rfc2822()).unwrap()
            )
            .ok_or_else(|| failure::err_msg(
                format!("Atom feed can only be generated if `published_date' or `updated_date' is added to the frontmatter of {}", self.url_path)
            ))?;

        let authors = self
            .front
            .authors
            .as_ref()
            .ok_or_else(|| failure::err_msg(
                format!("Atom feed requires authors. Please add it to the default frontmatter or the frontmatter of {}", self.url_path)
            ))?
            .iter()
            .map(|au| {
                atom_syndication::PersonBuilder::default()
                    .name(au.to_string())
                    .build()
            })
            .collect();

        let entry = atom_syndication::Entry {
            id: link.clone(),
            title: atom_syndication::Text::plain(self.front.title.to_string()),
            updated,
            summary: self
                .description_to_str()
                .map(|s| atom_syndication::Text::html(s)),
            published: self.front.published_date.map(|date| {
                atom_syndication::FixedDateTime::parse_from_rfc2822(&date.to_rfc2822()).unwrap()
            }),
            authors,
            links: vec![atom_syndication::Link {
                href: link.clone(),
                ..Default::default()
            }],
            categories: self
                .front
                .categories
                .iter()
                .map(|c| atom_syndication::Category {
                    term: c.as_str().to_owned(),
                    scheme: None,
                    label: None,
                })
                .collect(),
            ..Default::default()
        };

        Ok(entry)
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
            let date = vimwiki::vendor::chrono::DateTime::parse_from_rfc2822(&date.to_rfc2822())
                .expect("chrono/time compatible RFC 2822 implementations");
            url = url.lastmod(date);
        }
        writer.url(url)?;
        Ok(())
    }

    fn description_to_str(&self) -> Option<String> {
        self.front
            .description
            .as_ref()
            .map(|s| s.as_str().to_owned())
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
    fn render_html(&self, content: &str, context: &RenderContext<'_>) -> Result<String> {
        let html = if self.front.templated {
            let template = context.parser.parse(content)?;
            template.render(context.globals)?
        } else {
            content.to_owned()
        };

        let html = match self.front.format {
            cobalt_model::SourceFormat::Raw => html,
            cobalt_model::SourceFormat::Markdown => context.markdown.parse(&html)?,
            cobalt_model::SourceFormat::Vimwiki => context.vimwiki.parse(&html)?,
        };

        Ok(html)
    }

    /// Renders the excerpt and adds it to attributes of the document.
    ///
    /// The excerpt is either taken from the `excerpt` frontmatter setting, if
    /// given, or extracted from the content, if `excerpt_separator` is not
    /// empty. When neither condition applies, the excerpt is set to the `Nil`
    /// value.
    pub fn render_excerpt(&mut self, context: &RenderContext<'_>) -> Result<()> {
        let value = if let Some(excerpt_str) = self.front.excerpt.as_ref() {
            let excerpt = self.render_html(excerpt_str, context)?;
            Value::scalar(excerpt)
        } else if self.front.excerpt_separator.is_empty() {
            Value::Nil
        } else {
            let excerpt = extract_excerpt(
                &self.content,
                self.front.format,
                &self.front.excerpt_separator,
            );
            let excerpt = self.render_html(&excerpt, context)?;
            Value::scalar(excerpt)
        };

        self.attributes.insert("excerpt".into(), value);
        Ok(())
    }

    /// Renders the content and adds it to attributes of the document.
    ///
    /// When we say "content" we mean only this document without extended layout.
    pub fn render_content(&mut self, context: &RenderContext<'_>) -> Result<()> {
        let content_html = self.render_html(&self.content, context)?;
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
        context: &RenderContext<'_>,
        layouts: &HashMap<String, String>,
    ) -> Result<String> {
        if let Some(ref layout) = self.front.layout {
            let layout_data_ref = layouts.get(layout.as_str()).ok_or_else(|| {
                failure::format_err!(
                    "Layout {} does not exist (referenced in {}).",
                    layout,
                    self.file_path
                )
            })?;

            let template = context
                .parser
                .parse(layout_data_ref)
                .with_context(|_| failure::format_err!("Failed to parse layout {:?}", layout))?;
            let content_html = template
                .render(context.globals)
                .with_context(|_| failure::format_err!("Failed to render layout {:?}", layout))?;
            let content_html = minify_if_enabled(content_html, context, &self.file_path)?;
            Ok(content_html)
        } else {
            let path = &[
                liquid::model::KStringCow::from_static("page").into(),
                liquid::model::KStringCow::from_static("content").into(),
            ];
            let content_html = liquid::model::try_find(context.globals, path)
                .ok_or_else(|| failure::err_msg("Internal error: page isn't in globals"))?
                .render()
                .to_string();

            let content_html = minify_if_enabled(content_html, context, &self.file_path)?;
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
    format: cobalt_model::SourceFormat,
    excerpt_separator: &str,
) -> String {
    match format {
        cobalt_model::SourceFormat::Markdown => {
            extract_excerpt_markdown(content, excerpt_separator)
        }
        cobalt_model::SourceFormat::Vimwiki | cobalt_model::SourceFormat::Raw => {
            extract_excerpt_raw(content, excerpt_separator)
        }
    }
}
