use liquid;
use std::borrow::Cow;
use std::vec::Vec;
use super::SortOrder;

// struct Trails {
//   before: i32,
//   after: i32,
// }

pub(crate) struct PaginationCfg {
  pub(crate) include: String,
  pub(crate) per_page: i32,
  pub(crate) permalink: String,
  pub(crate) order: SortOrder,
  pub(crate) sort_by: Vec<String>,
  //trails: Trails,
}

const DEFAULT_PERMALINK: &str = "{{parent}}/{{include}}/_p/{{num}}/";
const DEFAULT_SORT: &str = "published_date";

impl PaginationCfg {
  pub fn new(pagination_cfg: &liquid::Object) -> PaginationCfg {
    PaginationCfg {
      include: pagination_cfg
        .get("include")
        .map_or("None".to_string(), |inc| {
          inc
            .as_scalar()
            .map_or("None".to_string(), |i| i.clone().into_string())
        }),
      per_page: pagination_cfg.get("per_page").map_or(10, |per| {
        per.as_scalar().map_or(10, |s| s.to_integer().unwrap_or(10))
      }),
      permalink: pagination_cfg
        .get("permalink")
        .map_or(DEFAULT_PERMALINK.to_string(), |p| {
          p.as_scalar()
            .map_or(DEFAULT_PERMALINK.to_string(), |p| p.to_string())
        }),
      order: pagination_cfg.get("order").map_or(SortOrder::Desc, |o| {
        o.as_scalar().map_or(SortOrder::Desc, |o| match o.to_str() {
          Cow::Borrowed("Desc") => SortOrder::Desc,
          Cow::Borrowed("Asc") => SortOrder::Asc,
          _ => SortOrder::Desc,
        })
      }),
      sort_by: pagination_cfg
        .get("sort_by")
        .map_or(vec![DEFAULT_SORT.to_string()], |s| {
          s.as_array().map_or(vec![DEFAULT_SORT.to_string()], |arr| {
            arr
              .iter()
              .map(|sort| {
                sort
                  .as_scalar()
                  .map_or(DEFAULT_SORT.to_string(), |conv| conv.clone().into_string())
              })
              .collect()
          })
        }),
      // trails: Trails {
      //   before: 0,
      //   after: 0,
      // }, // TODO: read from front
    }
  }

  pub fn is_pagination_enable(&self) -> bool {
    self.include != "None"
  }
}

#[cfg(test)]
mod test_pagination_config {
    use super::*;

    #[test]
    fn test_new() {
        let mut pagination_front = liquid::Object::new();
        pagination_front.insert("include".to_owned(), liquid::Value::scalar("all"));
        pagination_front.insert("per_page".to_owned(), liquid::Value::scalar(5));
        pagination_front.insert("permalink".to_owned(), liquid::Value::scalar("{{parent}}/{{include}}/_p/{{num}}/"));
        pagination_front.insert("order".to_owned(), liquid::Value::scalar("Asc"));
        pagination_front.insert("sort_by".to_owned(), liquid::Value::Array(vec![liquid::Value::scalar("published_date"), liquid::Value::scalar("title")]));
        let pagination_cfg = PaginationCfg::new(&pagination_front);
        assert_eq!(pagination_cfg.include, "all".to_owned());
        assert_eq!(pagination_cfg.per_page, 5);
        assert_eq!(pagination_cfg.permalink, "{{parent}}/{{include}}/_p/{{num}}/".to_owned());
        assert_eq!(pagination_cfg.order, SortOrder::Asc);
        assert_eq!(pagination_cfg.sort_by, vec!["published_date", "title"]);
    }

    #[test]
    fn test_new_default() {
        let pagination_front = liquid::Object::new();
        let pagination_cfg = PaginationCfg::new(&pagination_front);
        assert_eq!(pagination_cfg.include, "None".to_owned());
        assert_eq!(pagination_cfg.per_page, 10);
        assert_eq!(pagination_cfg.permalink, "{{parent}}/{{include}}/_p/{{num}}/".to_owned());
        assert_eq!(pagination_cfg.order, SortOrder::Desc);
        assert_eq!(pagination_cfg.sort_by, vec!["published_date"]);
    }
}
