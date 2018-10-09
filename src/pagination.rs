use std::cmp::Ordering;
use std::ops::Range;

use liquid;

use cobalt_model::pagination_config::Include;
use cobalt_model::pagination_config::PaginationConfig;
use cobalt_model::permalink;
use cobalt_model::SortOrder;
use document;
use document::Document;
use error::*;

pub struct Paginator {
  pub pages: Option<Vec<liquid::Value>>,
  pub indexes: Option<Vec<String>>,
  pub index: usize,
  pub index_permalink: String,
  pub previous_index: usize,
  pub previous_index_permalink: Option<String>,
  pub next_index: usize,
  pub next_index_permalink: Option<String>,
  pub first_index_permalink: String,
  pub last_index_permalink: String,
  pub total_indexes: usize,
  pub total_pages: usize,
}

impl Paginator {
  pub fn new(total_indexes: usize, total_pages: usize) -> Paginator {
    Paginator {
      pages: None,   // pages in current index
      indexes: None, // list of the available indexes, use when include is tags for instance
      index: 0,
      index_permalink: String::new(),
      previous_index: 0,
      previous_index_permalink: None,
      next_index: 0,
      next_index_permalink: None,
      first_index_permalink: String::new(),
      last_index_permalink: String::new(),
      total_indexes,
      total_pages,
    }
  }

  pub fn set_first_last(
    &mut self,
    doc: &Document,
    config: &PaginationConfig,
    total_pages: usize,
  ) -> Result<()> {
    self.first_index_permalink = format!("{}", doc.url_path);
    self.last_index_permalink = interpret_permalink(&config, &doc, total_pages)?;
    Ok(())
  }

  fn set_current_index_info(
    &mut self,
    index: usize,
    all_pages: &mut Vec<liquid::Value>,
    config: &PaginationConfig,
    doc: &Document,
  ) -> Result<()> {
    let nb_posts_left = all_pages.len();
    self.index = index;
    // `drain` is used to free up some memory as the pages are already put in a
    // paginator, no need to keep them in the `all_pages` vector
    self.pages = Some(
      all_pages
        .drain(range_for_page(config.per_page, nb_posts_left))
        .collect(),
    );
    self.index_permalink = interpret_permalink(&config, &doc, index)?;
    Ok(())
  }

  fn set_previous_next_info(
    &mut self,
    index: usize,
    total_indexes: usize,
    doc: &Document,
    config: &PaginationConfig,
  ) -> Result<()> {
    if index > 1 {
      // we have a previous index
      self.previous_index_permalink = Some(interpret_permalink(&config, &doc, index - 1)?);
      self.previous_index = index - 1;
    }

    if index < total_indexes {
      // we have a next index
      self.next_index = index + 1;
      self.next_index_permalink = Some(interpret_permalink(&config, &doc, index + 1)?);
    }
    Ok(())
  }
}

impl Into<liquid::Object> for Paginator {
  fn into(self) -> liquid::Object {
    let mut object = liquid::Object::new();
    // if no pages, means we have indexes instead, `tags` like cases for exemple
    if let Some(pages) = self.pages {
      object.insert("pages".to_owned(), liquid::Value::Array(pages));
    }
    // list of available indexes, in `tags` like cases
    if let Some(indexes) = self.indexes {
      object.insert(
        "indexes".to_owned(),
        liquid::Value::Array(indexes.iter().map(liquid::Value::scalar).collect()),
      );
    }
    object.insert("index".to_owned(), liquid::Value::scalar(self.index as i32));
    object.insert(
      "index_permalink".to_owned(),
      liquid::Value::scalar(self.index_permalink),
    );
    if let Some(previous_index_permalink) = self.previous_index_permalink {
      object.insert(
        "previous_index".to_owned(),
        liquid::Value::scalar(self.previous_index as i32),
      );
      object.insert(
        "previous_index_permalink".to_owned(),
        liquid::Value::scalar(previous_index_permalink),
      );
    }
    if let Some(next_index_permalink) = self.next_index_permalink {
      object.insert(
        "next_index".to_owned(),
        liquid::Value::scalar(self.next_index as i32),
      );
      object.insert(
        "next_index_permalink".to_owned(),
        liquid::Value::scalar(next_index_permalink),
      );
    }
    object.insert(
      "first_index_permalink".to_owned(),
      liquid::Value::scalar(self.first_index_permalink),
    );
    object.insert(
      "last_index_permalink".to_owned(),
      liquid::Value::scalar(self.last_index_permalink),
    );
    object.insert(
      "total_indexes".to_owned(),
      liquid::Value::scalar(self.total_indexes as i32),
    );
    object.insert(
      "total_pages".to_owned(),
      liquid::Value::scalar(self.total_pages as i32),
    );
    object
  }
}

pub fn generate_paginators(
  doc: &mut Document,
  posts_data: &[liquid::Value],
) -> Result<Vec<Paginator>> {
  let config = doc.front.pagination.as_ref().unwrap();
  let mut all_posts = posts_data.to_vec();
  match config.include {
    Include::All => {
      sort_posts(&mut all_posts, &config);
      create_all_paginators(&mut all_posts, &doc, &config)
    }
    Include::None => Ok(Vec::new()), // user can set `include: None`
  }
}

fn extract_value(a: &liquid::Value, key: &str) -> Option<liquid::Scalar> {
  a.get(&key.into())
    .and_then(|sort_key| sort_key.as_scalar().cloned())
}

// sort posts by multiple criteria
fn sort_posts(posts: &mut Vec<liquid::Value>, config: &PaginationConfig) {
  // Boxing needed to compile, don't really understand why
  let order: Box<Fn(liquid::Scalar, liquid::Scalar) -> Ordering> = match config.order {
    SortOrder::Desc => {
      Box::new(|a, b: liquid::Scalar| b.partial_cmp(&a).unwrap_or(Ordering::Equal))
    }
    SortOrder::Asc => Box::new(|a: liquid::Scalar, b| a.partial_cmp(&b).unwrap_or(Ordering::Equal)),
    SortOrder::None => {
      // when built, order is set like this:
      // `order.unwrap_or(SortOrder::Desc);` so it's unreachable
      unreachable!("Sort order should have default value when constructing PaginationConfig")
    }
  };
  posts.sort_by(|a, b| {
    let keys = &config.sort_by;
    let mut cmp = Ordering::Less;
    for k in keys {
      cmp = match (extract_value(a, &k), extract_value(b, &k)) {
        (Some(a), Some(b)) => order(a, b),
        (None, None) => Ordering::Equal,
        (_, None) => Ordering::Greater,
        (None, _) => Ordering::Less,
      };
      if cmp != Ordering::Equal {
        return cmp;
      }
    }
    cmp
  });
}

fn create_all_paginators(
  mut all_posts: &mut Vec<liquid::Value>,
  doc: &Document,
  pagination_cfg: &PaginationConfig,
) -> Result<Vec<Paginator>> {
  let total_pages = all_posts.len();
  // f32 used here in order to not lose information to ceil the result,
  // otherwise we can lose an index
  let total_indexes = (total_pages as f32 / pagination_cfg.per_page as f32).ceil() as usize;
  let mut paginators = Vec::with_capacity(total_indexes);
  // keeping `for` loop instead of `map` as asked in PR because I need `i`
  // anyway and it keep the code more understandable. The issue was
  // pre-allocating  which is done with `with_capacity`
  for i in 0..total_indexes {
    paginators.push(create_paginator(
      i,
      total_indexes,
      total_pages,
      &pagination_cfg,
      &doc,
      &mut all_posts,
    )?);
  }
  Ok(paginators)
}

fn create_paginator(
  i: usize,
  total_indexes: usize,
  total_pages: usize,
  config: &PaginationConfig,
  doc: &Document,
  mut all_posts: &mut Vec<liquid::Value>,
) -> Result<Paginator> {
  let index = i + 1;
  let mut paginator = Paginator::new(total_indexes, total_pages);

  paginator.set_first_last(&doc, &config, total_indexes)?;
  paginator.set_current_index_info(index, &mut all_posts, &config, &doc)?;
  paginator.set_previous_next_info(index, total_indexes, &doc, &config)?;

  Ok(paginator)
}

fn pagination_attributes(index_num: i32, include: Include) -> liquid::Object {
  let mut attributes = liquid::Object::new();
  attributes.insert("num".to_owned(), liquid::Value::scalar(index_num));
  let i: &str = include.into();
  attributes.insert("include".to_owned(), liquid::Value::scalar(i));
  attributes
}

fn interpret_permalink(
  config: &PaginationConfig,
  doc: &Document,
  index_num: usize,
) -> Result<String> {
  Ok(if index_num == 1 {
    doc.url_path.clone()
  } else {
    let mut attributes = document::permalink_attributes(&doc.front, &doc.file_path);
    let pagination_attr = pagination_attributes(index_num as i32, config.include);
    pagination_attr.into_iter().for_each(|(k, v)| {
      attributes.insert(k, v);
    });

    permalink::explode_permalink(
      &config.permalink,
      &attributes,
    )?
  })
}

fn range_for_page(per_page: i32, nb_posts: usize) -> Range<usize> {
  let nb_posts = nb_posts as i32;
  // make sure `end` is not beyond capacity
  0..if per_page < nb_posts {
    per_page as usize
  } else {
    nb_posts as usize
  } 
}

#[cfg(test)]
mod test_pagination {
  use super::*;

  #[test]
  fn test_extract_value() {
    let mut obj = liquid::Object::new();
    obj.insert("key".to_owned(), liquid::Value::scalar("toto"));
    let value = liquid::Value::Object(obj);
    let expected: liquid::Scalar = liquid::Scalar::new("toto");
    assert_eq!(Some(expected), extract_value(&value, &"key".to_owned()));
  }
}
