use std::fmt;
use std::path;
use std::collections::HashMap;

use chrono::Datelike;
use liquid;
use regex;
use serde;
use serde_yaml;

use error::Result;

use super::datetime;
use super::slug;

const PATH_ALIAS: &str = "/{{parent}}/{{name}}{{ext}}";
lazy_static!{
    static ref PERMALINK_ALIASES: HashMap<&'static str, &'static str> = [
        ("path", PATH_ALIAS),
    ].iter().map(|&(k, v)| (k, v)).collect();
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SourceFormat {
    Raw,
    Markdown,
}

impl Default for SourceFormat {
    fn default() -> SourceFormat {
        SourceFormat::Raw
    }
}

// TODO(epage): Remove the serde traits and instead provide an impl based on if serde traits exist
pub trait Front
    : Default + fmt::Display + for<'de> serde::Deserialize<'de> + serde::Serialize
    {
    fn parse(content: &str) -> Result<Self> {
        let front: Self = serde_yaml::from_str(content)?;
        Ok(front)
    }

    fn to_string(&self) -> Result<String> {
        let mut converted = serde_yaml::to_string(self)?;
        converted.drain(..4);
        if converted == "{}" {
            converted.clear();
        }
        Ok(converted)
    }
}

#[derive(Debug, Eq, PartialEq, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct FrontmatterBuilder {
    #[serde(skip_serializing_if = "Option::is_none")] pub permalink: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub categories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")] pub excerpt_separator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_date: Option<datetime::DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")] pub format: Option<SourceFormat>,
    #[serde(skip_serializing_if = "Option::is_none")] pub layout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub is_draft: Option<bool>,
    #[serde(skip_serializing_if = "liquid::Object::is_empty")] pub data: liquid::Object,
    // Controlled by where the file is found.  We might allow control over the type at a later
    // point but we need to first define those semantics.
    #[serde(skip)] pub collection: Option<String>,
}

impl FrontmatterBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_permalink<S: Into<Option<String>>>(self, permalink: S) -> Self {
        Self {
            permalink: permalink.into(),
            ..self
        }
    }

    pub fn set_slug<S: Into<Option<String>>>(self, slug: S) -> Self {
        Self {
            slug: slug.into(),
            ..self
        }
    }

    pub fn set_title<S: Into<Option<String>>>(self, title: S) -> Self {
        Self {
            title: title.into(),
            ..self
        }
    }

    pub fn set_description<S: Into<Option<String>>>(self, description: S) -> Self {
        Self {
            description: description.into(),
            ..self
        }
    }

    pub fn set_excerpt<S: Into<Option<String>>>(self, excerpt: S) -> Self {
        Self {
            excerpt: excerpt.into(),
            ..self
        }
    }

    pub fn set_categories<S: Into<Option<Vec<String>>>>(self, categories: S) -> Self {
        Self {
            categories: categories.into(),
            ..self
        }
    }

    pub fn set_excerpt_separator<S: Into<Option<String>>>(self, excerpt_separator: S) -> Self {
        Self {
            excerpt_separator: excerpt_separator.into(),
            ..self
        }
    }

    pub fn set_published_date<D: Into<Option<datetime::DateTime>>>(
        self,
        published_date: D,
    ) -> Self {
        Self {
            published_date: published_date.into(),
            ..self
        }
    }

    #[cfg(test)]
    pub fn set_format<S: Into<Option<SourceFormat>>>(self, format: S) -> Self {
        Self {
            format: format.into(),
            ..self
        }
    }

    pub fn set_layout<S: Into<Option<String>>>(self, layout: S) -> Self {
        Self {
            layout: layout.into(),
            ..self
        }
    }

    pub fn set_draft<B: Into<Option<bool>>>(self, is_draft: B) -> Self {
        Self {
            is_draft: is_draft.into(),
            ..self
        }
    }

    pub fn set_collection<S: Into<Option<String>>>(self, collection: S) -> Self {
        Self {
            collection: collection.into(),
            ..self
        }
    }

    pub fn merge_permalink<S: Into<Option<String>>>(self, permalink: S) -> Self {
        self.merge(Self::new().set_permalink(permalink.into()))
    }

    pub fn merge_slug<S: Into<Option<String>>>(self, slug: S) -> Self {
        self.merge(Self::new().set_slug(slug.into()))
    }

    pub fn merge_title<S: Into<Option<String>>>(self, title: S) -> Self {
        self.merge(Self::new().set_title(title.into()))
    }

    pub fn merge_description<S: Into<Option<String>>>(self, description: S) -> Self {
        self.merge(Self::new().set_description(description.into()))
    }

    pub fn merge_excerpt<S: Into<Option<String>>>(self, excerpt: S) -> Self {
        self.merge(Self::new().set_excerpt(excerpt.into()))
    }

    pub fn merge_categories<S: Into<Option<Vec<String>>>>(self, categories: S) -> Self {
        self.merge(Self::new().set_categories(categories.into()))
    }

    pub fn merge_excerpt_separator<S: Into<Option<String>>>(self, excerpt_separator: S) -> Self {
        self.merge(Self::new().set_excerpt_separator(excerpt_separator.into()))
    }

    pub fn merge_published_date<D: Into<Option<datetime::DateTime>>>(
        self,
        published_date: D,
    ) -> Self {
        self.merge(Self::new().set_published_date(published_date.into()))
    }

    #[cfg(test)]
    pub fn merge_format<S: Into<Option<SourceFormat>>>(self, format: S) -> Self {
        self.merge(Self::new().set_format(format.into()))
    }

    pub fn merge_layout<S: Into<Option<String>>>(self, layout: S) -> Self {
        self.merge(Self::new().set_layout(layout.into()))
    }

    pub fn merge_draft<B: Into<Option<bool>>>(self, draft: B) -> Self {
        self.merge(Self::new().set_draft(draft.into()))
    }

    #[cfg(test)]
    pub fn merge_collection<S: Into<Option<String>>>(self, collection: S) -> Self {
        self.merge(Self::new().set_collection(collection.into()))
    }

    pub fn merge_data(self, other_data: liquid::Object) -> Self {
        let Self {
            permalink,
            slug,
            title,
            description,
            excerpt,
            categories,
            excerpt_separator,
            published_date,
            format,
            layout,
            is_draft,
            collection,
            data,
        } = self;
        Self {
            permalink: permalink,
            slug: slug,
            title: title,
            description: description,
            excerpt: excerpt,
            categories: categories,
            excerpt_separator: excerpt_separator,
            published_date: published_date,
            format: format,
            layout: layout,
            is_draft: is_draft,
            collection: collection,
            data: merge_objects(data, other_data),
        }
    }

    pub fn merge(self, other: Self) -> Self {
        let Self {
            permalink,
            slug,
            title,
            description,
            excerpt,
            categories,
            excerpt_separator,
            published_date,
            format,
            layout,
            is_draft,
            collection,
            data,
        } = self;
        let Self {
            permalink: other_permalink,
            slug: other_slug,
            title: other_title,
            description: other_description,
            excerpt: other_excerpt,
            categories: other_categories,
            excerpt_separator: other_excerpt_separator,
            published_date: other_published_date,
            format: other_format,
            layout: other_layout,
            is_draft: other_is_draft,
            collection: other_collection,
            data: other_data,
        } = other;
        Self {
            permalink: permalink.or_else(|| other_permalink),
            slug: slug.or_else(|| other_slug),
            title: title.or_else(|| other_title),
            description: description.or_else(|| other_description),
            excerpt: excerpt.or_else(|| other_excerpt),
            categories: categories.or_else(|| other_categories),
            excerpt_separator: excerpt_separator.or_else(|| other_excerpt_separator),
            published_date: published_date.or_else(|| other_published_date),
            format: format.or_else(|| other_format),
            layout: layout.or_else(|| other_layout),
            is_draft: is_draft.or_else(|| other_is_draft),
            collection: collection.or_else(|| other_collection),
            data: merge_objects(data, other_data),
        }
    }

    pub fn merge_path<P: AsRef<path::Path>>(self, relpath: P) -> Self {
        self.merge_path_ref(relpath.as_ref())
    }

    fn merge_path_ref(mut self, relpath: &path::Path) -> Self {
        if self.format.is_none() {
            let ext = relpath.extension().and_then(|os| os.to_str()).unwrap_or("");
            let format = match ext {
                "md" => SourceFormat::Markdown,
                _ => SourceFormat::Raw,
            };
            self.format = Some(format);
        }

        if self.published_date.is_none() || self.slug.is_none() {
            let file_stem = file_stem(relpath);
            let (file_date, file_stem) = parse_file_stem(file_stem);
            if self.published_date.is_none() {
                self.published_date = file_date;
            }
            if self.slug.is_none() {
                let slug = slug::slugify(file_stem);
                self.slug = Some(slug);
            }
        }

        if self.title.is_none() {
            let slug = self.slug
                .as_ref()
                .expect("slug has been unconditionally initialized");
            let title = slug::titleize_slug(slug);
            self.title = Some(title);
        }

        self
    }

    pub fn build(self) -> Result<Frontmatter> {
        let Self {
            permalink,
            slug,
            title,
            description,
            excerpt,
            categories,
            excerpt_separator,
            published_date,
            format,
            layout,
            is_draft,
            collection,
            data,
        } = self;

        let collection = collection.unwrap_or_else(|| "".to_owned());

        let permalink = permalink.unwrap_or_else(|| PATH_ALIAS.to_owned());
        let permalink = if !permalink.starts_with('/') {
            let resolved = *PERMALINK_ALIASES
                .get(permalink.as_str())
                .ok_or_else(|| format!("Unsupported permalink alias '{}'", permalink))?;
            resolved.to_owned()
        } else {
            permalink
        };

        let fm = Frontmatter {
            permalink: permalink,
            slug: slug.ok_or_else(|| "No slug")?,
            title: title.ok_or_else(|| "No title")?,
            description: description,
            excerpt: excerpt,
            categories: categories.unwrap_or_else(|| vec![]),
            excerpt_separator: excerpt_separator.unwrap_or_else(|| "\n\n".to_owned()),
            published_date: published_date,
            format: format.unwrap_or_else(SourceFormat::default),
            layout: layout,
            is_draft: is_draft.unwrap_or(false),
            collection: collection,
            data: data,
        };

        Ok(fm)
    }
}

impl fmt::Display for FrontmatterBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let converted = Front::to_string(self).map_err(|_| fmt::Error)?;
        write!(f, "{}", converted)
    }
}

impl Front for FrontmatterBuilder {}

#[derive(Debug, Eq, PartialEq, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Frontmatter {
    pub permalink: String,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub excerpt: Option<String>,
    pub categories: Vec<String>,
    pub excerpt_separator: String,
    pub published_date: Option<datetime::DateTime>,
    pub format: SourceFormat,
    pub layout: Option<String>,
    pub is_draft: bool,
    pub collection: String,
    pub data: liquid::Object,
}

impl Front for Frontmatter {}

impl fmt::Display for Frontmatter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let converted = Front::to_string(self).map_err(|_| fmt::Error)?;
        write!(f, "{}", converted)
    }
}

/// Shallow merge of `liquid::Object`'s
fn merge_objects(mut primary: liquid::Object, secondary: liquid::Object) -> liquid::Object {
    for (key, value) in secondary {
        primary
            .entry(key.to_owned())
            .or_insert_with(|| value.clone());
    }
    primary
}

/// The base-name without an extension.  Correlates to Jekyll's :name path tag
fn file_stem<P: AsRef<path::Path>>(p: P) -> String {
    file_stem_path(p.as_ref())
}

fn file_stem_path(p: &path::Path) -> String {
    p.file_stem()
        .map(|os| os.to_string_lossy().into_owned())
        .unwrap_or_else(|| "".to_owned())
}

fn parse_file_stem(stem: String) -> (Option<datetime::DateTime>, String) {
    lazy_static!{
       static ref DATE_PREFIX_REF: regex::Regex =
           regex::Regex::new(r"^(\d{4})-(\d{1,2})-(\d{1,2})[- ](.*)$")
           .unwrap();
    }

    let parts = DATE_PREFIX_REF.captures(&stem).and_then(|caps| {
        let year: i32 = caps.get(1)
            .expect("unconditional capture")
            .as_str()
            .parse()
            .expect("regex gets back an integer");
        let month: u32 = caps.get(2)
            .expect("unconditional capture")
            .as_str()
            .parse()
            .expect("regex gets back an integer");
        let day: u32 = caps.get(3)
            .expect("unconditional capture")
            .as_str()
            .parse()
            .expect("regex gets back an integer");
        let published = datetime::DateTime::default()
            .with_year(year)
            .and_then(|d| d.with_month(month))
            .and_then(|d| d.with_day(day));
        published.map(|p| {
            (
                Some(p),
                caps.get(4)
                    .expect("unconditional capture")
                    .as_str()
                    .to_owned(),
            )
        })
    });

    parts.unwrap_or((None, stem))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn file_stem_absolute_path() {
        let input = path::PathBuf::from("/embedded/path/___filE-worlD-__09___.md");
        let actual = file_stem(input.as_path());
        assert_eq!(actual, "___filE-worlD-__09___");
    }

    #[test]
    fn parse_file_stem_empty() {
        assert_eq!(parse_file_stem("".to_owned()), (None, "".to_owned()));
    }

    #[test]
    fn parse_file_stem_none() {
        assert_eq!(
            parse_file_stem("First Blog Post".to_owned()),
            (None, "First Blog Post".to_owned())
        );
    }

    #[test]
    fn parse_file_stem_out_of_range_month() {
        assert_eq!(
            parse_file_stem("2017-30-5 First Blog Post".to_owned()),
            (None, "2017-30-5 First Blog Post".to_owned())
        );
    }

    #[test]
    fn parse_file_stem_out_of_range_day() {
        assert_eq!(
            parse_file_stem("2017-3-50 First Blog Post".to_owned()),
            (None, "2017-3-50 First Blog Post".to_owned())
        );
    }

    #[test]
    fn parse_file_stem_single_digit() {
        assert_eq!(
            parse_file_stem("2017-3-5 First Blog Post".to_owned()),
            (
                Some(
                    datetime::DateTime::default()
                        .with_year(2017)
                        .unwrap()
                        .with_month(3)
                        .unwrap()
                        .with_day(5)
                        .unwrap()
                ),
                "First Blog Post".to_owned()
            )
        );
    }

    #[test]
    fn parse_file_stem_double_digit() {
        assert_eq!(
            parse_file_stem("2017-12-25 First Blog Post".to_owned()),
            (
                Some(
                    datetime::DateTime::default()
                        .with_year(2017)
                        .unwrap()
                        .with_month(12)
                        .unwrap()
                        .with_day(25)
                        .unwrap()
                ),
                "First Blog Post".to_owned()
            )
        );
    }

    #[test]
    fn parse_file_stem_double_digit_leading_zero() {
        assert_eq!(
            parse_file_stem("2017-03-05 First Blog Post".to_owned()),
            (
                Some(
                    datetime::DateTime::default()
                        .with_year(2017)
                        .unwrap()
                        .with_month(3)
                        .unwrap()
                        .with_day(5)
                        .unwrap()
                ),
                "First Blog Post".to_owned()
            )
        );
    }

    #[test]
    fn parse_file_stem_dashed() {
        assert_eq!(
            parse_file_stem("2017-3-5-First-Blog-Post".to_owned()),
            (
                Some(
                    datetime::DateTime::default()
                        .with_year(2017)
                        .unwrap()
                        .with_month(3)
                        .unwrap()
                        .with_day(5)
                        .unwrap()
                ),
                "First-Blog-Post".to_owned()
            )
        );
    }

    #[test]
    fn frontmatter_title_from_path() {
        let front = FrontmatterBuilder::new()
            .merge_path("./parent/file.md")
            .build()
            .unwrap();
        assert_eq!(front.title, "File");
    }

    #[test]
    fn frontmatter_slug_from_md_path() {
        let front = FrontmatterBuilder::new()
            .merge_path("./parent/file.md")
            .build()
            .unwrap();
        assert_eq!(front.slug, "file");
    }

    #[test]
    fn frontmatter_markdown_from_path() {
        let front = FrontmatterBuilder::new()
            .merge_path("./parent/file.md")
            .build()
            .unwrap();
        assert_eq!(front.format, SourceFormat::Markdown);
    }

    #[test]
    fn frontmatter_raw_from_path() {
        let front = FrontmatterBuilder::new()
            .merge_path("./parent/file.liquid")
            .build()
            .unwrap();
        assert_eq!(front.format, SourceFormat::Raw);
    }

    #[test]
    fn frontmatter_global_merge() {
        let empty = FrontmatterBuilder::new();
        let a = FrontmatterBuilder {
            permalink: Some("permalink a".to_owned()),
            slug: Some("slug a".to_owned()),
            title: Some("title a".to_owned()),
            description: Some("description a".to_owned()),
            excerpt: Some("excerpt a".to_owned()),
            categories: Some(vec!["a".to_owned(), "b".to_owned()]),
            excerpt_separator: Some("excerpt_separator a".to_owned()),
            published_date: Some(datetime::DateTime::default()),
            format: Some(SourceFormat::Markdown),
            layout: Some("layout a".to_owned()),
            is_draft: Some(true),
            collection: Some("pages".to_owned()),
            data: liquid::Object::new(),
        };
        let b = FrontmatterBuilder {
            permalink: Some("permalink b".to_owned()),
            slug: Some("slug b".to_owned()),
            title: Some("title b".to_owned()),
            description: Some("description b".to_owned()),
            excerpt: Some("excerpt b".to_owned()),
            categories: Some(vec!["b".to_owned(), "a".to_owned()]),
            excerpt_separator: Some("excerpt_separator b".to_owned()),
            published_date: Some(datetime::DateTime::default()),
            format: Some(SourceFormat::Raw),
            layout: Some("layout b".to_owned()),
            is_draft: Some(true),
            collection: Some("posts".to_owned()),
            data: liquid::Object::new(),
        };

        let merge_b_into_a = a.clone().merge(b.clone());
        assert_eq!(merge_b_into_a, a);

        let merge_empty_into_a = a.clone().merge(empty.clone());
        assert_eq!(merge_empty_into_a, a);

        let merge_a_into_empty = empty.clone().merge(a.clone());
        assert_eq!(merge_a_into_empty, a);
    }

    #[test]
    fn frontmatter_local_merge() {
        let a = FrontmatterBuilder {
            permalink: Some("permalink a".to_owned()),
            slug: Some("slug a".to_owned()),
            title: Some("title a".to_owned()),
            description: Some("description a".to_owned()),
            excerpt: Some("excerpt a".to_owned()),
            categories: Some(vec!["a".to_owned(), "b".to_owned()]),
            excerpt_separator: Some("excerpt_separator a".to_owned()),
            published_date: None,
            format: Some(SourceFormat::Markdown),
            layout: Some("layout a".to_owned()),
            is_draft: Some(true),
            collection: Some("pages".to_owned()),
            data: liquid::Object::new(),
        };

        let merge_b_into_a = a.clone()
            .merge_permalink("permalink b".to_owned())
            .merge_slug("slug b".to_owned())
            .merge_title("title b".to_owned())
            .merge_description("description b".to_owned())
            .merge_excerpt("excerpt b".to_owned())
            .merge_categories(vec!["a".to_owned(), "b".to_owned()])
            .merge_excerpt_separator("excerpt_separator b".to_owned())
            .merge_format(SourceFormat::Raw)
            .merge_layout("layout b".to_owned())
            .merge_draft(true)
            .merge_collection("posts".to_owned());
        assert_eq!(merge_b_into_a, a);

        let merge_empty_into_a = a.clone()
            .merge_permalink(None)
            .merge_slug(None)
            .merge_title(None)
            .merge_description(None)
            .merge_excerpt(None)
            .merge_categories(None)
            .merge_excerpt_separator(None)
            .merge_format(None)
            .merge_layout(None)
            .merge_draft(None)
            .merge_collection(None);
        assert_eq!(merge_empty_into_a, a);

        let merge_a_into_empty = FrontmatterBuilder::new()
            .merge_permalink("permalink a".to_owned())
            .merge_slug("slug a".to_owned())
            .merge_title("title a".to_owned())
            .merge_description("description a".to_owned())
            .merge_excerpt("excerpt a".to_owned())
            .merge_categories(vec!["a".to_owned(), "b".to_owned()])
            .merge_excerpt_separator("excerpt_separator a".to_owned())
            .merge_format(SourceFormat::Markdown)
            .merge_layout("layout a".to_owned())
            .merge_draft(true)
            .merge_collection("pages".to_owned());
        assert_eq!(merge_a_into_empty, a);
    }

    #[test]
    fn frontmatter_defaults() {
        FrontmatterBuilder::new()
            .set_title("Title".to_owned())
            .set_slug("Slug".to_owned())
            .build()
            .unwrap();
    }
}
