use std::convert::Into;
use std::vec::Vec;

use super::*;

const DEFAULT_PER_PAGE: i32 = 10;
const DEFAULT_PERMALINK: &str = "{{num}}/";
const DEFAULT_SORT: &str = "published_date";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields, default)]
pub struct Pagination {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Include>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permalink_suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<SortOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_index: Option<Vec<DateIndex>>,
}

impl Pagination {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn with_defaults() -> Self {
        Self {
            include: Some(Include::None),
            per_page: Some(DEFAULT_PER_PAGE),
            permalink_suffix: Some(DEFAULT_PERMALINK.to_owned()),
            order: Some(SortOrder::Desc),
            sort_by: Some(vec![DEFAULT_SORT.to_owned()]),
            date_index: Some(vec![DateIndex::Year, DateIndex::Month]),
        }
    }

    pub fn merge(self, other: &Self) -> Self {
        let Pagination {
            include,
            per_page,
            permalink_suffix,
            order,
            sort_by,
            date_index,
        } = self;
        Self {
            include: include.or(other.include),
            per_page: per_page.or(other.per_page),
            permalink_suffix: permalink_suffix.or_else(|| other.permalink_suffix.clone()),
            order: order.or(other.order),
            sort_by: sort_by.or_else(|| other.sort_by.clone()),
            date_index: date_index.or_else(|| other.date_index.clone()),
        }
    }
}

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub enum Include {
    None,
    All,
    Tags,
    Categories,
    Dates,
}

impl Into<&'static str> for Include {
    fn into(self) -> &'static str {
        match self {
            Include::None => "",
            Include::All => "all",
            Include::Tags => "tags",
            Include::Categories => "categories",
            Include::Dates => "dates",
        }
    }
}

impl Default for Include {
    fn default() -> Include {
        Include::None
    }
}

#[derive(
    Copy, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub enum DateIndex {
    Year,
    Month,
    Day,
    Hour,
    Minute,
}
