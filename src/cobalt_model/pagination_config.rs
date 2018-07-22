use std::convert::Into;
use std::vec::Vec;

use super::SortOrder;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum Include {
  None,
  #[serde(rename = "all")]
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, default)]
pub struct PaginationConfig {
  pub include: Include,
  pub per_page: i32,
  pub permalink: String,
  pub order: SortOrder,
  pub sort_by: Vec<String>,
}

const DEFAULT_PERMALINK: &str = "{{parent}}/{{include}}/_p/{{num}}/";
const DEFAULT_SORT: &str = "published_date";
const DEFAULT_PER_PAGE: i32 = 10;

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

impl PaginationConfig {
  pub fn is_pagination_enable(&self) -> bool {
    self.include != Include::None
  }
}
