use std::fmt;
use std::path;

use super::*;

#[derive(Debug, Eq, PartialEq, Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "unstable", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "unstable"), non_exhaustive)]
pub struct Frontmatter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permalink: Option<Permalink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt_separator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_date: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<SourceFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub templated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_draft: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i32>,
    #[serde(skip_serializing_if = "liquid_core::Object::is_empty")]
    pub data: liquid_core::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<Pagination>,
    // Controlled by where the file is found.  We might allow control over the type at a later
    // point but we need to first define those semantics.
    #[serde(skip)]
    pub collection: Option<String>,
}

impl Frontmatter {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn merge_path(mut self, relpath: &path::Path) -> Self {
        if let Some(name) = relpath.file_name().and_then(|f| f.to_str()) {
            let mut split_name = crate::path::split_ext(name);

            #[cfg(feature = "preview_unstable")]
            if split_name.1 == Some("liquid") {
                self.templated.get_or_insert(true);
                split_name = crate::path::split_ext(split_name.0);
            } else {
                self.templated.get_or_insert(false);
            }

            let format = match split_name.1 {
                Some("md") => SourceFormat::Markdown,
                Some("wiki") => SourceFormat::Vimwiki,
                _ => SourceFormat::Raw,
            };
            self.format.get_or_insert(format);

            while split_name.1.is_some() {
                split_name = crate::path::split_ext(split_name.0);
            }

            if self.published_date.is_none() || self.slug.is_none() {
                let file_stem = split_name.0;
                let (file_date, file_stem) = crate::path::parse_file_stem(file_stem);
                if self.published_date.is_none() {
                    self.published_date = file_date;
                }
                if self.slug.is_none() {
                    let slug = crate::path::slugify(file_stem);
                    if self.title.is_none() {
                        self.title = Some(crate::path::titleize_slug(&slug));
                    }
                    self.slug = Some(slug);
                }
            }
        }

        self
    }

    pub fn merge(self, other: &Self) -> Self {
        let Self {
            permalink,
            slug,
            title,
            description,
            excerpt,
            categories,
            tags,
            excerpt_separator,
            published_date,
            format,
            templated,
            layout,
            is_draft,
            weight,
            collection,
            data,
            pagination,
        } = self;
        Self {
            permalink: permalink.or_else(|| other.permalink.clone()),
            slug: slug.or_else(|| other.slug.clone()),
            title: title.or_else(|| other.title.clone()),
            description: description.or_else(|| other.description.clone()),
            excerpt: excerpt.or_else(|| other.excerpt.clone()),
            categories: categories.or_else(|| other.categories.clone()),
            tags: tags.or_else(|| other.tags.clone()),
            excerpt_separator: excerpt_separator.or_else(|| other.excerpt_separator.clone()),
            published_date: published_date.or(other.published_date),
            format: format.or(other.format),
            templated: templated.or(other.templated),
            layout: layout.or_else(|| other.layout.clone()),
            is_draft: is_draft.or(other.is_draft),
            weight: weight.or(other.weight),
            collection: collection.or_else(|| other.collection.clone()),
            data: merge_objects(data, &other.data),
            pagination: merge_pagination(pagination, &other.pagination),
        }
    }
}

impl fmt::Display for Frontmatter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let converted = serde_yaml::to_string(self).expect("should always be valid");
        let subset = converted
            .strip_prefix("---")
            .unwrap_or_else(|| converted.as_str())
            .trim();
        let converted = if subset == "{}" { "" } else { subset };
        if converted.is_empty() {
            Ok(())
        } else {
            write!(f, "{}", converted)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "unstable", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "unstable"), non_exhaustive)]
pub enum Permalink {
    Alias(PermalinkAlias),
    Explicit(String),
}

impl Permalink {
    pub fn as_str(&self) -> &str {
        match self {
            Permalink::Alias(PermalinkAlias::Path) => "/{{parent}}/{{name}}{{ext}}",
            Permalink::Explicit(path) => path.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[cfg_attr(not(feature = "unstable"), non_exhaustive)]
pub enum PermalinkAlias {
    Path,
}

impl std::ops::Deref for Permalink {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for Permalink {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Default for Permalink {
    fn default() -> Self {
        Permalink::Alias(PermalinkAlias::Path)
    }
}

impl fmt::Display for Permalink {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "preview_unstable", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "unstable", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "unstable"), non_exhaustive)]
pub enum SourceFormat {
    Raw,
    Markdown,
    Vimwiki,
    #[cfg(not(feature = "unstable"))]
    #[doc(hidden)]
    #[serde(other)]
    Unknown,
}

impl Default for SourceFormat {
    fn default() -> SourceFormat {
        SourceFormat::Raw
    }
}

/// Shallow merge of `liquid_core::Object`'s
fn merge_objects(
    mut primary: liquid_core::Object,
    secondary: &liquid_core::Object,
) -> liquid_core::Object {
    for (key, value) in secondary {
        primary
            .entry(key.to_owned())
            .or_insert_with(|| value.clone());
    }
    primary
}

fn merge_pagination(
    primary: Option<Pagination>,
    secondary: &Option<Pagination>,
) -> Option<Pagination> {
    if let Some(primary) = primary {
        if let Some(secondary) = secondary {
            Some(primary.merge(secondary))
        } else {
            Some(primary)
        }
    } else {
        secondary.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn display_empty() {
        let front = Frontmatter::empty();
        assert_eq!(&front.to_string(), "");
    }

    #[test]
    fn display_slug() {
        let front = Frontmatter {
            slug: Some("foo".to_owned()),
            ..Default::default()
        };
        assert_eq!(&front.to_string(), "slug: foo");
    }

    #[test]
    fn display_permalink_alias() {
        let front = Frontmatter {
            permalink: Some(Permalink::Alias(PermalinkAlias::Path)),
            ..Default::default()
        };
        assert_eq!(&front.to_string(), "permalink: path");
    }

    #[test]
    fn display_permalink_explicit() {
        let front = Frontmatter {
            permalink: Some(Permalink::Explicit("foo".to_owned())),
            ..Default::default()
        };
        assert_eq!(&front.to_string(), "permalink: foo");
    }
}
