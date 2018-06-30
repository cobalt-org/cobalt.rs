use cobalt_model::pagination_config::{Order, PaginationCfg};
use cobalt_model::permalink;
use document;
use document::Document;
use liquid;
use std::cmp::Ordering;
use std::ops::Range;

pub(crate) fn generate_paginators(
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
      cmp = if let Some(a) = extract_value(a, &k) {
        if let Some(b) = extract_value(b, &k) {
          match config.order {
            Order::Desc => b.partial_cmp(&a).unwrap_or(Ordering::Equal),
            Order::Asc => a.partial_cmp(&b).unwrap_or(Ordering::Equal),
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
      &pagination_cfg,
      &doc,
      &mut all_posts,
    ));
  }
}

fn create_paginator(
  i: i32,
  total_pages: i32,
  total_posts: i32,
  config: &PaginationCfg,
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
    &config,
    &file_name,
    &doc,
  );

  fill_current_page_info(
    &mut paginator,
    page,
    &mut all_posts,
    &config,
    &file_name,
    &doc,
  );

  fill_previous_next_info(&mut paginator, page, total_pages, &file_name, &doc, &config);

  // TODO trails
  paginator
}

fn pagination_attributes(page_num: i32, include: &String) -> liquid::Object {
  let mut attributes = liquid::Object::new();
  attributes.insert("num".to_owned(), liquid::Value::scalar(page_num));
  attributes.insert("include".to_owned(), liquid::Value::scalar(include));
  attributes
}

fn interpret_permalink(
  config: &PaginationCfg,
  doc: &Document,
  page_num: i32,
  file_name: &str,
) -> String {
  let mut attributes = document::permalink_attributes(&doc.front, &doc.file_path);
  let pagination_attr = pagination_attributes(page_num, &config.include);
  pagination_attr.into_iter().for_each(|(k, v)| {
    attributes.insert(k, v);
  });
  let permalink = permalink::explode_permalink(&config.permalink, &attributes);
  permalink
    .and_then(|mut p| {
      p.push_str(file_name);
      Ok(p)
    })
    .unwrap_or(file_name.to_owned())
}

fn init_paginator_constants(
  paginator: &mut liquid::Object,
  total_posts: i32,
  total_pages: i32,
  config: &PaginationCfg,
  file_name: &str,
  doc: &Document,
) {
  paginator.insert(
    "per_page".to_owned(),
    liquid::Value::scalar(config.per_page),
  );
  paginator.insert("total_posts".to_owned(), liquid::Value::scalar(total_posts));
  paginator.insert("total_pages".to_owned(), liquid::Value::scalar(total_pages));
  paginator.insert(
    "first_page_path".to_owned(),
    liquid::Value::scalar(format!("/{}", doc.file_path.to_string_lossy())),
  );
  paginator.insert(
    "last_page_path".to_owned(),
    liquid::Value::scalar(interpret_permalink(&config, &doc, total_pages, &file_name)),
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
  config: &PaginationCfg,
  file_name: &str,
  doc: &Document,
) {
  let nb_posts_left = all_posts.len();
  paginator.insert("page".to_owned(), liquid::Value::scalar(page));
  paginator.insert(
    "posts".to_owned(),
    liquid::Value::Array(
      all_posts
        .drain(range_for_page(config.per_page, nb_posts_left))
        .collect(),
    ),
  );

  paginator.insert(
    "page_path".to_owned(),
    liquid::Value::scalar(if page == 1 {
      format!("{}", file_name)
    } else {
      let mut path = interpret_permalink(&config, &doc, page, &file_name);
      if path.starts_with('/') {
        path.remove(0);
      }
      path
    }),
  );
}

fn fill_previous_next_info(
  paginator: &mut liquid::Object,
  page: i32,
  total_pages: i32,
  file_name: &str,
  doc: &Document,
  config: &PaginationCfg,
) {
  if page > 1 {
    // we have a previous page
    paginator.insert(
      "previous_page_path".to_owned(),
      liquid::Value::scalar(if page == 2 {
        format!("/{}", file_name)
      } else {
        interpret_permalink(&config, &doc, page - 1, &file_name)
      }),
    );
    paginator.insert("previous_page".to_owned(), liquid::Value::scalar(page - 1));
  }
  if page < total_pages {
    // we have a next page
    paginator.insert(
      "next_page_path".to_owned(),
      liquid::Value::scalar(interpret_permalink(&config, &doc, page + 1, &file_name)),
    );
    paginator.insert("next_page".to_owned(), liquid::Value::scalar(page + 1));
  }
}
