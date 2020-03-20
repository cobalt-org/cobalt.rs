use std::iter::FromIterator;
use std::path;
use std::vec::Vec;

use cobalt_config::DateIndex;
use cobalt_config::DateTime;
use cobalt_config::Include;
use cobalt_config::SortOrder;
use cobalt_config::SourceFormat;

use crate::Result;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Frontmatter {
    pub permalink: cobalt_config::Permalink,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub excerpt: Option<String>,
    pub categories: Vec<String>,
    pub tags: Option<Vec<String>>,
    pub excerpt_separator: String,
    pub published_date: Option<DateTime>,
    pub format: SourceFormat,
    pub layout: Option<String>,
    pub is_draft: bool,
    pub weight: i32,
    pub collection: String,
    pub data: liquid::Object,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pagination {
    pub include: Include,
    pub per_page: i32,
    pub front_permalink: cobalt_config::Permalink,
    pub permalink_suffix: String,
    pub order: SortOrder,
    pub sort_by: Vec<String>,
    pub date_index: Vec<DateIndex>,
}

impl Pagination {
    fn from_config(
        config: cobalt_config::Pagination,
        permalink: &cobalt_config::Permalink,
    ) -> Option<Self> {
        let config = config.merge(&cobalt_config::Pagination::with_defaults());
        let cobalt_config::Pagination {
            include,
            per_page,
            permalink_suffix,
            order,
            sort_by,
            date_index,
        } = config;
        let include = include.expect("default applied");
        let per_page = per_page.expect("default applied");
        let permalink_suffix = permalink_suffix.expect("default applied");
        let order = order.expect("default applied");
        let sort_by = sort_by.expect("default applied");
        let date_index = date_index.expect("default applied");

        if include == Include::None {
            return None;
        }
        Some(Self {
            include,
            per_page,
            front_permalink: permalink.to_owned(),
            permalink_suffix,
            order,
            sort_by,
            date_index,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawContent {
    pub content: String,
}

pub fn derive_component(
    src_path: &path::Path,
    rel_path: &path::Path,
    default_front: &cobalt_config::Frontmatter,
) -> Result<(Frontmatter, Option<Pagination>, RawContent)> {
    let content = std::fs::read_to_string(src_path)
        .map_err(|e| crate::Status::new("Failed to read page").with_internal(e))?;
    let content = String::from_iter(normalize_line_endings::normalized(content.chars()));
    let builder = cobalt_config::Document::parse(&content)?;
    let (front, content) = builder.into_parts();
    let front = front.merge_path(rel_path).merge(default_front);

    let (front, pagination) = convert_frontmatter(front)?;

    let content = RawContent { content };

    Ok((front, pagination, content))
}

fn convert_frontmatter(
    config: cobalt_config::Frontmatter,
) -> Result<(Frontmatter, Option<Pagination>)> {
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
        layout,
        is_draft,
        weight,
        collection,
        data,
        pagination,
    } = config;

    let collection = collection.unwrap_or_default();

    let permalink = permalink.unwrap_or_default();
    if let cobalt_config::Permalink::Explicit(permalink) = &permalink {
        if !permalink.starts_with('/') {
            return Err(crate::Status::new("Unsupported permalink alias")
                .context_with(|c| c.insert("alias", permalink.to_owned())));
        }
    }

    if let Some(ref tags) = tags {
        if tags.iter().any(|x| x.trim().is_empty()) {
            status::bail!("Empty strings are not allowed in tags");
        }
    }
    let tags = if tags.as_ref().map(|t| t.len()).unwrap_or(0) == 0 {
        None
    } else {
        tags
    };

    let pagination = pagination.and_then(|p| Pagination::from_config(p, &permalink));
    if let Some(pagination) = pagination.as_ref() {
        if !is_date_index_sorted(&pagination.date_index) {
            status::bail!("date_index is not correctly sorted: Year > Month > Day...");
        }
    }

    let fm = Frontmatter {
        permalink,
        slug: slug.ok_or_else(|| crate::Status::new("No slug"))?,
        title: title.ok_or_else(|| crate::Status::new("No title"))?,
        description,
        excerpt,
        categories: categories.unwrap_or_else(|| vec![]),
        tags,
        excerpt_separator: excerpt_separator.unwrap_or_else(|| "\n\n".to_owned()),
        published_date,
        format: format.unwrap_or_else(SourceFormat::default),
        layout,
        is_draft: is_draft.unwrap_or(false),
        weight: weight.unwrap_or(0),
        collection,
        data,
    };

    Ok((fm, pagination))
}

// TODO to be replaced by a call to `is_sorted()` once it's stabilized
fn is_date_index_sorted(v: &[DateIndex]) -> bool {
    let mut copy = v.to_vec();
    copy.sort_unstable();
    copy.as_slice().eq(v)
}
