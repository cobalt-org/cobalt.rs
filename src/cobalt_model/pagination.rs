use document::Document;
use liquid;
use liquid::Value;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Range;
use std::vec::Vec;

enum Order {
  Desc,
  Asc,
}

struct Trails {
  before: i32,
  after: i32,
}

pub struct PaginationCfg {
  include: String,
  per_page: i32,
  permalink: String,
  order: Order,
  sort_by: Vec<String>,
  trails: Trails,
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
      trails: Trails {
        before: 0,
        after: 0,
      }, // TODO: read from front
    }
  }

  pub fn is_pagination_enable(&self) -> bool {
    self.include != "None"
  }
}

pub fn generate_paginators(
  doc: &mut Document,
  posts_data: &[liquid::Value],
  config: &PaginationCfg,
) -> Vec<liquid::Object> {
  let mut paginators = Vec::new();
  let mut all_posts = posts_data.to_vec();
  match config.include.as_str() {
    "all" => {
      sort_posts(&mut all_posts, &config);
      create_all_paginators(&mut paginators, &mut all_posts, &doc, &config);
    }
    _ => {}
  };

  paginators
}

fn extract_value(a: &liquid::Value, key: &String) -> Option<liquid::Scalar> {
  if let Some(attr) = a.as_object() {
    if let Some(sort_key) = attr.get(key) {
      if let Some(value) = sort_key.as_scalar() {
        Some(value.clone())
      } else {
        None
      }
    } else {
      None
    }
  } else {
    None
  }
}

fn sort_posts(posts: &mut Vec<liquid::Value>, config: &PaginationCfg) {
  posts.sort_by(|a, b| {
    let keys = config.sort_by.clone();
    let mut cmp = Ordering::Less;
    for k in keys {
      trace!("sort by {}", k);
      cmp = if let Some(a) = extract_value(a, &k) {
        if let Some(b) = extract_value(b, &k) {
          match config.order {
            Order::Desc => b.partial_cmp(&a).unwrap_or(Ordering::Equal),
            Order::Asc => a.partial_cmp(&b).unwrap_or(Ordering::Equal)
          }
        } else {
          Ordering::Greater
        }
      } else {
        Ordering::Less
      };
      if cmp != Ordering::Equal {
        return cmp;
      }
    }
    cmp
  });
}

fn create_all_paginators(
  paginators: &mut Vec<liquid::Object>,
  mut all_posts: &mut Vec<liquid::Value>,
  doc: &Document,
  pagination_cfg: &PaginationCfg,
) {
  let total_posts = all_posts.len() as i32;
  let total_pages = (total_posts as f32 / pagination_cfg.per_page as f32).ceil() as i32;
  for i in 0..total_pages {
    paginators.push(create_paginator(
      i,
      total_pages,
      total_posts,
      pagination_cfg.per_page,
      &doc,
      &mut all_posts,
    ));
  }
}

fn create_paginator(
  i: i32,
  total_pages: i32,
  total_posts: i32,
  per_page: i32,
  doc: &Document,
  mut all_posts: &mut Vec<liquid::Value>,
) -> liquid::Object {
  let page = i + 1;
  let mut paginator = liquid::Object::new();
  let file_name = doc
    .file_path
    .file_name()
    .map_or("index.html", |s| s.to_str().unwrap_or("index.html"));

  init_paginator_constants(
    &mut paginator,
    total_posts,
    total_pages,
    per_page,
    &file_name,
  );

  fill_current_page_info(&mut paginator, page, &mut all_posts, per_page, &file_name);

  fill_previous_next_info(&mut paginator, page, total_pages, &file_name);

  // TODO trails
  paginator
}

fn init_paginator_constants(
  paginator: &mut liquid::Object,
  total_posts: i32,
  total_pages: i32,
  per_page: i32,
  file_name: &str,
) {
  paginator.insert("per_page".to_owned(), Value::scalar(per_page));
  paginator.insert("total_posts".to_owned(), Value::scalar(total_posts));
  paginator.insert("total_pages".to_owned(), Value::scalar(total_pages));
  paginator.insert(
    "first_page_path".to_owned(),
    Value::scalar(format!("/{}", file_name)),
  );
  paginator.insert(
    "last_page_path".to_owned(),
    Value::scalar(format!("/_p/{:02}_{}", total_pages, file_name)),
  );
}

fn range_for_page(per_page: i32, nb_posts: usize) -> Range<usize> {
  let nb_posts = nb_posts as i32;
  // make sure `end` is not beyond capacity
  let end = if per_page < nb_posts {
    per_page
  } else {
    nb_posts
  } as usize;
  0..end
}

fn fill_current_page_info(
  paginator: &mut liquid::Object,
  page: i32,
  all_posts: &mut Vec<liquid::Value>,
  per_page: i32,
  file_name: &str,
) {
  let nb_posts_left = all_posts.len();
  paginator.insert("page".to_owned(), Value::scalar(page));
  paginator.insert(
    "posts".to_owned(),
    Value::Array(
      all_posts
        .drain(range_for_page(per_page, nb_posts_left))
        .collect(),
    ),
  );

  paginator.insert(
    "page_path".to_owned(),
    Value::scalar(if page == 1 {
      format!("{}", file_name)
    } else {
      format!("_p/{:02}_{}", page, file_name)
    }),
  );
}

fn fill_previous_next_info(
  paginator: &mut liquid::Object,
  page: i32,
  total_pages: i32,
  file_name: &str,
) {
  if page > 1 {
    // we have a previous page
    paginator.insert(
      "previous_page_path".to_owned(),
      Value::scalar(if page == 2 {
        format!("/{}", file_name)
      } else {
        format!("/_p/{:02}_{}", page - 1, file_name)
      }),
    );
    paginator.insert("previous_page".to_owned(), Value::scalar(page - 1));
  }
  if page < total_pages {
    // we have a next page
    paginator.insert(
      "next_page_path".to_owned(),
      Value::scalar(format!("/_p/{:02}_{}", page + 1, file_name)),
    );
    paginator.insert("next_page".to_owned(), Value::scalar(page + 1));
  }
}
