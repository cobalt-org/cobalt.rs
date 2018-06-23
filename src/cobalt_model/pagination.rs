use document::Document;
use liquid;
use liquid::{Array, Value};
use std::borrow::Cow;
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

struct PaginationCfg {
  //includes: Option<Array>,
  per_page: i32,
  permalink: String,
  order: Order,
  sort_by: Vec<String>,
  trails: Trails,
}

impl PaginationCfg {
  pub fn new(pagination_cfg: &HashMap<String, liquid::Value>) -> PaginationCfg {
    PaginationCfg {
      // includes: pagination_cfg
      //   .get("include")
      //   .and_then(|inc| inc.as_array().and_then(|arr| Some(arr.clone()))),
      per_page: pagination_cfg.get("per_page").map_or(10, |per| {
        per.as_scalar().map_or(10, |s| s.to_integer().unwrap_or(10))
      }),
      permalink: pagination_cfg.get("permalink").map_or(
        "{{parent}}/{{include}}/_p/{{num}}/".to_owned(),
        |p| {
          p.as_scalar()
            .map_or("{{parent}}/{{include}}/_p/{{num}}/".to_owned(), |p| {
              p.to_string()
            })
        },
      ),
      order: pagination_cfg.get("order").map_or(Order::Desc, |o| {
        o.as_scalar().map_or(Order::Desc, |o| match o.to_str() {
          Cow::Borrowed("Desc") => Order::Desc,
          Cow::Borrowed("Asc") => Order::Asc,
          _ => Order::Desc,
        })
      }),
      sort_by: pagination_cfg
        .get("sort_by")
        .map_or(vec!["published_date".to_owned()], |s| {
          s.as_array()
            .map_or(vec!["published_date".to_owned()], |arr| {
              arr
                .iter()
                .map(|sort| {
                  sort
                    .as_scalar()
                    .map_or("published_date".to_owned(), |conv| {
                      conv.clone().into_string()
                    })
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
}

pub fn is_pagination_enable(pagination: &liquid::Object) -> bool {
  !pagination.is_empty() && if let Some(include) = pagination.get("include") {
    include
      .as_scalar()
      .unwrap_or(&liquid::Scalar::from("None"))
      .to_str() != Cow::Borrowed("None")
  } else {
    false
  }
}

pub fn generate_paginators(
  doc: &mut Document,
  posts_data: &[liquid::Value],
) -> Vec<liquid::Object> {
  let mut paginators = Vec::new();
  if let Some(include) = doc.front.pagination.get("include") {
    process_index(&mut paginators, &include.to_str(), &posts_data, &doc)
  };

  paginators
}

fn process_index(
  paginators: &mut Vec<liquid::Object>,
  index: &str,
  posts_data: &[liquid::Value],
  doc: &Document,
) {
  let pagination_cfg = PaginationCfg::new(&doc.front.pagination);
  match index {
    "all" => {
      let mut all_posts = posts_data.to_vec();
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
    _ => {}
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

  fill_current_page_info(
    &mut paginator,
    page,
    &mut all_posts,
    per_page,
    &file_name,
  );

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
    Value::scalar(format!("./{}", file_name)),
  );
  paginator.insert(
    "last_page_path".to_owned(),
    Value::scalar(format!("./_p/{:02}_{}", total_pages, file_name)),
  );
}

fn range_for_page(per_page: i32, nb_posts: usize) -> Range<usize> {
  let nb_posts = nb_posts as i32;
  // make sure `end` is not beyond capacity
  let end = if per_page < nb_posts { per_page } else { nb_posts } as usize;
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
