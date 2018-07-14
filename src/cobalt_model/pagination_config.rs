use std::convert::Into;
use std::vec::Vec;

use liquid;

use super::SortOrder;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum Include {
  None,
  #[serde(rename = "all")]
  All,
  // tags,
}

impl Into<liquid::Scalar> for Include {
  fn into(self) -> liquid::Scalar {
    match self {
      Include::None => liquid::Scalar::new(""),
      Include::All => liquid::Scalar::new("all"),
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

// #[cfg(test)]
// mod test_pagination_config {
//   use super::*;

//   #[test]
//   fn test_new() {
//     let mut pagination_front = liquid::Object::new();
//     pagination_front.insert("include".to_owned(), liquid::Value::scalar("all"));
//     pagination_front.insert("per_page".to_owned(), liquid::Value::scalar(5));
//     pagination_front.insert(
//       "permalink".to_owned(),
//       liquid::Value::scalar("{{parent}}/{{include}}/_p/{{num}}/"),
//     );
//     pagination_front.insert("order".to_owned(), liquid::Value::scalar("Asc"));
//     pagination_front.insert(
//       "sort_by".to_owned(),
//       liquid::Value::Array(vec![
//         liquid::Value::scalar("published_date"),
//         liquid::Value::scalar("title"),
//       ]),
//     );
//     let pagination_cfg = PaginationConfig::new(&pagination_front);
//     assert_eq!(pagination_cfg.include, "all".to_owned());
//     assert_eq!(pagination_cfg.per_page, 5);
//     assert_eq!(
//       pagination_cfg.permalink,
//       "{{parent}}/{{include}}/_p/{{num}}/".to_owned()
//     );
//     assert_eq!(pagination_cfg.order, SortOrder::Asc);
//     assert_eq!(pagination_cfg.sort_by, vec!["published_date", "title"]);
//   }

//   #[test]
//   fn test_new_default() {
//     let pagination_front = liquid::Object::new();
//     let pagination_cfg = PaginationConfig::new(&pagination_front);
//     assert_eq!(pagination_cfg.include, "None".to_owned());
//     assert_eq!(pagination_cfg.per_page, 10);
//     assert_eq!(
//       pagination_cfg.permalink,
//       "{{parent}}/{{include}}/_p/{{num}}/".to_owned()
//     );
//     assert_eq!(pagination_cfg.order, SortOrder::Desc);
//     assert_eq!(pagination_cfg.sort_by, vec!["published_date"]);
//   }
// }
