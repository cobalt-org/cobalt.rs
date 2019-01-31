use std::cmp::Ordering;

use liquid;

use crate::cobalt_model::pagination_config::Include;
use crate::cobalt_model::pagination_config::PaginationConfig;
use crate::cobalt_model::permalink;
use crate::cobalt_model::slug;
use crate::cobalt_model::SortOrder;
use crate::document;
use crate::document::Document;
use crate::error::*;

use std::collections::HashMap;

#[derive(Default, Clone, Debug)]
pub struct Paginator {
    pub pages: Option<Vec<liquid::value::Value>>,
    pub indexes: Option<Vec<Paginator>>,
    pub index: usize,
    pub index_title: Option<liquid::value::Value>,
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
            index_title: None,
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
        index_title: Option<&liquid::value::Value>,
    ) -> Result<()> {
        self.first_index_permalink = doc.url_path.to_string();
        self.last_index_permalink = interpret_permalink(&config, &doc, total_pages, index_title)?;
        Ok(())
    }

    fn set_current_index_info(
        &mut self,
        index: usize,
        all_pages: &[&liquid::value::Value],
        config: &PaginationConfig,
        doc: &Document,
        index_title: Option<&liquid::value::Value>,
    ) -> Result<()> {
        self.index = index;
        self.pages = Some(all_pages.into_iter().map(|p| (*p).clone()).collect());
        self.index_title = index_title.map(|i| i.clone());
        self.index_permalink = interpret_permalink(&config, &doc, index, index_title)?;
        Ok(())
    }

    fn set_previous_next_info(
        &mut self,
        index: usize,
        total_indexes: usize,
        doc: &Document,
        config: &PaginationConfig,
        index_title: Option<&liquid::value::Value>,
    ) -> Result<()> {
        if index > 1 {
            // we have a previous index
            self.previous_index_permalink =
                Some(interpret_permalink(&config, &doc, index - 1, index_title)?);
            self.previous_index = index - 1;
        }

        if index < total_indexes {
            // we have a next index
            self.next_index = index + 1;
            self.next_index_permalink =
                Some(interpret_permalink(&config, &doc, index + 1, index_title)?);
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
                    indexes
                        .into_iter()
                        .map(|paginator| {
                            let v: liquid::value::Object = paginator.into();
                            liquid::value::Value::Object(v)
                        })
                        .collect(),
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
        if let Some(index_title) = self.index_title {
            object.insert("index_title".into(), index_title);
        }
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
            create_all_paginators(&all_posts, &doc, &config, None)
        }
        Include::Tags => create_tags_paginators(&all_posts, &doc, &config),
        Include::None => {
            unreachable!("PaginationConfigBuilder should have lead to a None for pagination.")
        }
    }
}

fn extract_value<'a>(a: &'a liquid::value::Value, key: &str) -> Option<&'a liquid::value::Value> {
    let key = liquid::value::Scalar::new(key.to_owned());
    a.get(&key)
}

fn extract_scalar<'a>(a: &'a liquid::value::Value, key: &str) -> Option<&'a liquid::value::Scalar> {
    extract_value(a, key).and_then(|sort_key| sort_key.as_scalar())
}

fn extract_tags<'a>(value: &'a liquid::value::Value) -> Option<&'a liquid::value::Array> {
    extract_value(value, "tags").and_then(|tags| tags.as_array())
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
            cmp = match (extract_scalar(a, &k), extract_scalar(b, &k)) {
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

fn distribute_posts_by_tags<'a>(
    all_posts: &[&'a liquid::value::Value],
) -> Result<HashMap<String, Vec<&'a liquid::value::Value>>> {
    let mut per_tags: HashMap<String, Vec<&'a liquid::value::Value>> = HashMap::new();
    for post in all_posts {
        if let Some(tags) = extract_tags(post) {
            for tag in tags {
                let tag = tag
                    .as_scalar()
                    .ok_or_else(|| failure::err_msg("Should have string tags"))?
                    .to_str();
                // add_to_tag(&tag.to_string(), post, &mut per_tags)
                let cur_tag = per_tags.entry(tag.to_string()).or_insert(vec![]);
                cur_tag.push(post);
            }
        }
    }
    Ok(per_tags)
}

#[derive(Default, Debug)]
struct TagPaginators {
    firsts_of_tags: Vec<Paginator>,
    paginators: Vec<Paginator>,
}

fn create_tags_paginators(
    all_posts: &[&liquid::value::Value],
    doc: &Document,
    pagination_cfg: &PaginationConfig,
) -> Result<Vec<Paginator>> {
    let mut per_tags = distribute_posts_by_tags(&all_posts)?;

    // create all other paginators
    let mut tag_paginators: TagPaginators = per_tags
        .iter_mut()
        .try_fold(TagPaginators::default(), |mut acc, (tag, posts)| {
            sort_posts(posts, &pagination_cfg);
            let cur_tag_paginators = create_all_paginators(
                posts,
                doc,
                &pagination_cfg,
                Some(&liquid::value::Value::scalar(tag.to_owned())),
            )?;
            acc.firsts_of_tags.push(cur_tag_paginators[0].clone());
            acc.paginators.extend(cur_tag_paginators.into_iter());
            Ok(acc)
        })
        .or_else(|e: Error| Err(e))?;

    tag_paginators.firsts_of_tags.sort_unstable_by_key(|p| {
        if let Some(ref index_title) = p.index_title {
            Some(slug::slugify(index_title.to_str()).to_lowercase())
        } else {
            None
        }
    });
    let mut first = Paginator::default();
    first.indexes = Some(tag_paginators.firsts_of_tags);
    tag_paginators.paginators.insert(0, first);
    Ok(tag_paginators.paginators)
}

fn create_all_paginators(
    all_posts: &[&liquid::value::Value],
    doc: &Document,
    pagination_cfg: &PaginationConfig,
    index_title: Option<&liquid::value::Value>,
) -> Result<Vec<Paginator>> {
    let total_pages = all_posts.len();
    // f32 used here in order to not lose information to ceil the result,
    // otherwise we can lose an index
    let total_indexes = (total_pages as f32 / pagination_cfg.per_page as f32).ceil() as usize;
    let paginators: Result<Vec<_>> = all_posts
        .chunks(pagination_cfg.per_page as usize)
        .enumerate()
        .map(|(i, chunk)| {
            create_paginator(
                i,
                total_indexes,
                total_pages,
                &pagination_cfg,
                &doc,
                &chunk,
                index_title,
            )
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
    index_title: Option<&liquid::value::Value>,
) -> Result<Paginator> {
    let index = i + 1;
    let mut paginator = Paginator::new(total_indexes, total_pages);

    paginator.set_first_last(&doc, &config, total_indexes, index_title)?;
    paginator.set_current_index_info(index, &all_posts, &config, &doc, index_title)?;
    paginator.set_previous_next_info(index, total_indexes, &doc, &config, index_title)?;

    Ok(paginator)
}

fn pagination_attributes(page_num: i32, include: String) -> liquid::value::Object {
    let attributes: liquid::value::Object = vec![
        ("num".into(), liquid::value::Value::scalar(page_num)),
        ("include".into(), liquid::value::Value::scalar(include)),
    ]
    .into_iter()
    .collect();
    attributes
}

fn interpret_permalink(
    config: &PaginationConfig,
    doc: &Document,
    page_num: usize,
    index: Option<&liquid::value::Value>,
) -> Result<String> {
    Ok(if page_num == 1 {
        if let Some(index) = index {
            let include: &str = config.include.into();
            format!("{}/{}", include, slug::slugify(index.to_str()))
        } else {
            doc.url_path.clone()
        }
    } else {
        let mut attributes = document::permalink_attributes(&doc.front, &doc.file_path);
        let include: &str = config.include.into();
        let include: String = if let Some(index) = index {
            format!("{}/{}", include, slug::slugify(index.to_str()))
        } else {
            include.to_owned()
        };
        let pagination_attr = pagination_attributes(page_num as i32, include);
        attributes.extend(pagination_attr.into_iter());
        permalink::explode_permalink(&config.permalink, &attributes)?
    })
}
