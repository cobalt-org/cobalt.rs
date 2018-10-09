use std::convert::Into;
use std::vec::Vec;

use super::SortOrder;

pub const DEFAULT_PERMALINK: &str = "{{parent}}/{{include}}/_p/{{num}}/";
pub const DEFAULT_SORT: &str = "published_date";
pub const DEFAULT_PER_PAGE: i32 = 10;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum Include {
    None,
    All,
}

impl Into<&'static str> for Include {
    fn into(self) -> &'static str {
        match self {
            Include::None => "",
            Include::All => "all",
        }
    }
}

impl Default for Include {
    fn default() -> Include {
        Include::None
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields, default)]
pub struct PaginationConfigBuilder {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Include>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permalink: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<SortOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<Vec<String>>,
}

impl PaginationConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_include<S: Into<Option<Include>>>(self, include: S) -> Self {
        Self {
            include: include.into(),
            ..self
        }
    }

    pub fn set_per_page<S: Into<Option<i32>>>(self, per_page: S) -> Self {
        Self {
            per_page: per_page.into(),
            ..self
        }
    }

    pub fn set_permalink<S: Into<Option<String>>>(self, permalink: S) -> Self {
        Self {
            permalink: permalink.into(),
            ..self
        }
    }

    pub fn set_order<S: Into<Option<SortOrder>>>(self, order: S) -> Self {
        Self {
            order: order.into(),
            ..self
        }
    }

    pub fn set_sort_by<S: Into<Option<Vec<String>>>>(self, sort_by: S) -> Self {
        Self {
            sort_by: sort_by.into(),
            ..self
        }
    }

    pub fn merge(mut self, secondary: &PaginationConfigBuilder) -> PaginationConfigBuilder {
        if self.include.is_none() {
            self.include = secondary.include;
        }
        if self.per_page.is_none() {
            self.per_page = secondary.per_page;
        }
        if self.permalink.is_none() {
            self.permalink = secondary.permalink.clone();
        }
        if self.order.is_none() {
            self.order = secondary.order;
        }
        if self.sort_by.is_none() {
            self.sort_by = secondary.sort_by.clone();
        }
        self
    }

    pub fn build(self) -> Option<PaginationConfig> {
        let Self {
            include,
            per_page,
            permalink,
            order,
            sort_by,
        } = self;

        let include = include.unwrap_or(Include::None);
        if include == Include::None {
            return None;
        }
        let per_page = per_page.unwrap_or(DEFAULT_PER_PAGE);
        let permalink = permalink.unwrap_or_else(|| DEFAULT_PERMALINK.to_owned());
        let order = order.unwrap_or(SortOrder::Desc);
        let sort_by = sort_by.unwrap_or_else(|| vec![DEFAULT_SORT.to_owned()]);
        Some(PaginationConfig {
            include,
            per_page,
            permalink,
            order,
            sort_by,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, default)]
pub struct PaginationConfig {
    pub include: Include,
    pub per_page: i32,
    pub permalink: String,
    pub order: SortOrder,
    pub sort_by: Vec<String>,
}

impl Default for PaginationConfig {
    fn default() -> PaginationConfig {
        PaginationConfig {
            include: Default::default(),
            per_page: DEFAULT_PER_PAGE,
            permalink: DEFAULT_PERMALINK.to_owned(),
            order: SortOrder::Desc,
            sort_by: vec![DEFAULT_SORT.to_owned()],
        }
    }
}
