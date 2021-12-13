use std::fmt;

use cobalt_config::DateTime;
use cobalt_config::SourceFormat;
use liquid;
use serde::Serialize;

use super::pagination;
use crate::error::Result;

#[derive(Debug, Eq, PartialEq, Default, Clone, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Frontmatter {
    pub permalink: cobalt_config::Permalink,
    pub slug: kstring::KString,
    pub title: kstring::KString,
    pub description: Option<kstring::KString>,
    pub excerpt: Option<kstring::KString>,
    pub categories: Vec<kstring::KString>,
    pub tags: Option<Vec<kstring::KString>>,
    pub excerpt_separator: kstring::KString,
    pub published_date: Option<DateTime>,
    pub format: SourceFormat,
    pub templated: bool,
    pub layout: Option<kstring::KString>,
    pub is_draft: bool,
    pub weight: i32,
    pub collection: kstring::KString,
    pub data: liquid::Object,
    pub pagination: Option<pagination::PaginationConfig>,
}

impl Frontmatter {
    pub fn from_config(config: cobalt_config::Frontmatter) -> Result<Frontmatter> {
        let cobalt_config::Frontmatter {
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
        } = config;

        let collection = collection.unwrap_or_default();

        let permalink = permalink.unwrap_or_default();

        if let Some(ref tags) = tags {
            if tags.iter().any(|x| x.trim().is_empty()) {
                failure::bail!("Empty strings are not allowed in tags");
            }
        }
        let tags = if tags.as_ref().map(|t| t.len()).unwrap_or(0) == 0 {
            None
        } else {
            tags
        };
        let fm = Frontmatter {
            pagination: pagination
                .and_then(|p| pagination::PaginationConfig::from_config(p, &permalink)),
            permalink,
            slug: slug.ok_or_else(|| failure::err_msg("No slug"))?,
            title: title.ok_or_else(|| failure::err_msg("No title"))?,
            description,
            excerpt,
            categories: categories.unwrap_or_else(Vec::new),
            tags,
            excerpt_separator: excerpt_separator.unwrap_or_else(|| "\n\n".into()),
            published_date,
            format: format.unwrap_or_default(),
            #[cfg(feature = "preview_unstable")]
            templated: templated.unwrap_or(false),
            #[cfg(not(feature = "preview_unstable"))]
            templated: templated.unwrap_or(true),
            layout,
            is_draft: is_draft.unwrap_or(false),
            weight: weight.unwrap_or(0),
            collection,
            data,
        };

        if let Some(pagination) = &fm.pagination {
            if !pagination::is_date_index_sorted(&pagination.date_index) {
                failure::bail!("date_index is not correctly sorted: Year > Month > Day...");
            }
        }
        Ok(fm)
    }
}

impl fmt::Display for Frontmatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
