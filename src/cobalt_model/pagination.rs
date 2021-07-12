use std::vec::Vec;

use cobalt_config::SortOrder;

pub use cobalt_config::DateIndex;
pub use cobalt_config::Include;

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, default)]
pub struct PaginationConfig {
    pub include: Include,
    pub per_page: i32,
    pub front_permalink: cobalt_config::Permalink,
    pub permalink_suffix: String,
    pub order: SortOrder,
    pub sort_by: Vec<String>,
    pub date_index: Vec<DateIndex>,
}

impl PaginationConfig {
    pub fn from_config(
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

// TODO to be replaced by a call to `is_sorted()` once it's stabilized
pub fn is_date_index_sorted(v: &[DateIndex]) -> bool {
    let mut copy = v.to_owned();
    copy.sort_unstable();
    copy.eq(v)
}
