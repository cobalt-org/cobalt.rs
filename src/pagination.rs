use std::cmp::Ordering;

use liquid;

use cobalt_model::pagination_config::Include;
use cobalt_model::pagination_config::PaginationConfig;
use cobalt_model::permalink;
use cobalt_model::SortOrder;
use document;
use document::Document;
use error::*;

pub struct Paginator {
    pub pages: Option<Vec<liquid::value::Value>>,
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
        self.first_index_permalink = doc.url_path.to_string();
        self.last_index_permalink = interpret_permalink(&config, &doc, total_pages)?;
        Ok(())
    }

    fn set_current_index_info(
        &mut self,
        index: usize,
        all_pages: &[&liquid::value::Value],
        config: &PaginationConfig,
        doc: &Document,
    ) -> Result<()> {
        self.index = index;
        self.pages = Some(all_pages.into_iter().map(|p| (*p).clone()).collect());
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

impl Into<liquid::value::Object> for Paginator {
    fn into(self) -> liquid::value::Object {
        let mut object = liquid::value::Object::new();
        // if no pages, means we have indexes instead, `tags` like cases for exemple
        if let Some(pages) = self.pages {
            object.insert("pages".into(), liquid::value::Value::Array(pages));
        }
        // list of available indexes, in `tags` like cases
        if let Some(indexes) = self.indexes {
            object.insert(
                "indexes".into(),
                liquid::value::Value::Array(
                    indexes.iter().cloned().map(liquid::value::Value::scalar).collect(),
                ),
            );
        }
        object.insert(
            "index".into(),
            liquid::value::Value::scalar(self.index as i32),
        );
        object.insert(
            "index_permalink".into(),
            liquid::value::Value::scalar(self.index_permalink),
        );
        if let Some(previous_index_permalink) = self.previous_index_permalink {
            object.insert(
                "previous_index".into(),
                liquid::value::Value::scalar(self.previous_index as i32),
            );
            object.insert(
                "previous_index_permalink".into(),
                liquid::value::Value::scalar(previous_index_permalink),
            );
        }
        if let Some(next_index_permalink) = self.next_index_permalink {
            object.insert(
                "next_index".into(),
                liquid::value::Value::scalar(self.next_index as i32),
            );
            object.insert(
                "next_index_permalink".into(),
                liquid::value::Value::scalar(next_index_permalink),
            );
        }
        object.insert(
            "first_index_permalink".into(),
            liquid::value::Value::scalar(self.first_index_permalink),
        );
        object.insert(
            "last_index_permalink".into(),
            liquid::value::Value::scalar(self.last_index_permalink),
        );
        object.insert(
            "total_indexes".into(),
            liquid::value::Value::scalar(self.total_indexes as i32),
        );
        object.insert(
            "total_pages".into(),
            liquid::value::Value::scalar(self.total_pages as i32),
        );
        object
    }
}

pub fn generate_paginators(
    doc: &mut Document,
    posts_data: &[liquid::value::Value],
) -> Result<Vec<Paginator>> {
    let config = doc
        .front
        .pagination
        .as_ref()
        .expect("Front should have pagination here.");
    let mut all_posts: Vec<_> = posts_data.iter().collect();
    match config.include {
        Include::All => {
            sort_posts(&mut all_posts, &config);
            create_all_paginators(&all_posts, &doc, &config)
        }
        Include::None => {
            unreachable!("PaginationConfigBuilder should have lead to a None for pagination.")
        }
    }
}

fn extract_value<'a>(a: &'a liquid::value::Value, key: &str) -> Option<&'a liquid::value::Scalar> {
    let key = liquid::value::Scalar::new(key.to_owned());
    a.get(&key).and_then(|sort_key| sort_key.as_scalar())
}

// sort posts by multiple criteria
fn sort_posts(posts: &mut Vec<&liquid::value::Value>, config: &PaginationConfig) {
    let order: fn(&liquid::value::Scalar, &liquid::value::Scalar) -> Ordering = match config.order {
        SortOrder::Desc => {
            |a, b: &liquid::value::Scalar| b.partial_cmp(a).unwrap_or(Ordering::Equal)
        }
        SortOrder::Asc => {
            |a: &liquid::value::Scalar, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)
        }
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
    })
}

fn create_all_paginators(
    all_posts: &[&liquid::value::Value],
    doc: &Document,
    pagination_cfg: &PaginationConfig,
) -> Result<Vec<Paginator>> {
    let total_pages = all_posts.len();
    // f32 used here in order to not lose information to ceil the result,
    // otherwise we can lose an index
    let total_indexes = (total_pages as f32 / pagination_cfg.per_page as f32).ceil() as usize;
    let paginators: Result<Vec<_>> = all_posts
        .chunks(pagination_cfg.per_page as usize)
        .enumerate()
        .map(|(i, chunk)| {
            create_paginator(i, total_indexes, total_pages, &pagination_cfg, &doc, &chunk)
        })
        .collect();
    paginators
}

fn create_paginator(
    i: usize,
    total_indexes: usize,
    total_pages: usize,
    config: &PaginationConfig,
    doc: &Document,
    all_posts: &[&liquid::value::Value],
) -> Result<Paginator> {
    let index = i + 1;
    let mut paginator = Paginator::new(total_indexes, total_pages);

    paginator.set_first_last(&doc, &config, total_indexes)?;
    paginator.set_current_index_info(index, &all_posts, &config, &doc)?;
    paginator.set_previous_next_info(index, total_indexes, &doc, &config)?;

    Ok(paginator)
}

fn pagination_attributes(index_num: i32, include: Include) -> liquid::value::Object {
    let i: &str = include.into();
    let attributes: liquid::value::Object = vec![
        ("num".into(), liquid::value::Value::scalar(index_num)),
        ("include".into(), liquid::value::Value::scalar(i)),
    ]
    .into_iter()
    .collect();
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
        attributes.extend(pagination_attr.into_iter());
        permalink::explode_permalink(&config.permalink, &attributes)?
    })
}
