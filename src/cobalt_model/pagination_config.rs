use liquid;
use std::borrow::Cow;
use std::collections::HashMap;
use std::vec::Vec;

pub(crate) enum Order {
  Desc,
  Asc,
}

// struct Trails {
//   before: i32,
//   after: i32,
// }

pub(crate) struct PaginationCfg {
  pub(crate) include: String,
  pub(crate) per_page: i32,
  pub(crate) permalink: String,
  pub(crate) order: Order,
  pub(crate) sort_by: Vec<String>,
  //trails: Trails,
}

const DEFAULT_PERMALINK: &str = "{{parent}}/{{include}}/_p/{{num}}/";
const DEFAULT_SORT: &str = "published_date";

impl PaginationCfg {
  pub fn new(pagination_cfg: &HashMap<String, liquid::Value>) -> PaginationCfg {
    trace!("Parsing pagination cfg: {:#?}", pagination_cfg);
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
      order: pagination_cfg.get("order").map_or(Order::Desc, |o| {
        o.as_scalar().map_or(Order::Desc, |o| match o.to_str() {
          Cow::Borrowed("Desc") => Order::Desc,
          Cow::Borrowed("Asc") => Order::Asc,
          _ => Order::Desc,
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
