use std::cmp::Ordering;

use liquid::ValueView;

use crate::cobalt_model::pagination::Include;
use crate::cobalt_model::pagination::PaginationConfig;
use crate::cobalt_model::permalink;
use crate::cobalt_model::slug;
use crate::cobalt_model::SortOrder;

use crate::document;
use crate::document::Document;
use crate::error::Result;

mod categories;
mod dates;
mod helpers;
mod paginator;
mod tags;

use paginator::Paginator;

pub fn generate_paginators(
    doc: &mut Document,
    posts_data: &[liquid::model::Value],
) -> Result<Vec<Paginator>> {
    let config = doc
        .front
        .pagination
        .as_ref()
        .expect("Front should have pagination here.");
    let mut all_posts: Vec<_> = posts_data.iter().collect();
    match config.include {
        Include::All => {
            sort_posts(&mut all_posts, config);
            create_all_paginators(&all_posts, doc, config, None)
        }
        Include::Tags => tags::create_tags_paginators(&all_posts, doc, config),
        Include::Categories => categories::create_categories_paginators(&all_posts, doc, config),
        Include::Dates => dates::create_dates_paginators(&all_posts, doc, config),
        Include::None => {
            unreachable!("PaginationConfigBuilder should have lead to a None for pagination.")
        }
    }
}

fn create_all_paginators(
    all_posts: &[&liquid::model::Value],
    doc: &Document,
    pagination_cfg: &PaginationConfig,
    index_title: Option<&liquid::model::Value>,
) -> Result<Vec<Paginator>> {
    let total_pages = all_posts.len();
    // f32 used here in order to not lose information to ceil the result,
    // otherwise we can lose an index
    let total_indexes = (total_pages as f32 / pagination_cfg.per_page as f32).ceil() as usize;
    let paginators: Result<Vec<_>> = all_posts
        .chunks(pagination_cfg.per_page as usize)
        .enumerate()
        .map(|(i, chunk)| {
            paginator::create_paginator(
                i,
                total_indexes,
                total_pages,
                pagination_cfg,
                doc,
                chunk,
                index_title,
            )
        })
        .collect();
    paginators
}

// sort posts by multiple criteria
fn sort_posts(posts: &mut Vec<&liquid::model::Value>, config: &PaginationConfig) {
    let order: fn(liquid::model::ScalarCow<'_>, liquid::model::ScalarCow<'_>) -> Ordering =
        match config.order {
            SortOrder::Desc => {
                |a, b: liquid::model::ScalarCow<'_>| b.partial_cmp(&a).unwrap_or(Ordering::Equal)
            }
            SortOrder::Asc => {
                |a: liquid::model::ScalarCow<'_>, b| a.partial_cmp(&b).unwrap_or(Ordering::Equal)
            }
            SortOrder::None => {
                // when built, order is set like this:
                // `order.unwrap_or(SortOrder::Desc);` so it's unreachable
                unreachable!(
                    "Sort order should have default value when constructing PaginationConfig"
                )
            }
        };
    posts.sort_by(|a, b| {
        let keys = &config.sort_by;
        let mut cmp = Ordering::Less;
        for k in keys {
            cmp = match (
                helpers::extract_scalar(a.as_view(), k),
                helpers::extract_scalar(b.as_view(), k),
            ) {
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

fn pagination_attributes(page_num: i32) -> liquid::Object {
    let attributes: liquid::Object = vec![("num".into(), liquid::model::Value::scalar(page_num))]
        .into_iter()
        .collect();
    attributes
}

fn index_to_string(index: &liquid::model::Value) -> String {
    if let Some(index) = index.as_array() {
        // categories
        let mut s: String = index
            .values()
            .map(|i| {
                let mut s = slug::slugify(i.to_kstr().into_string());
                s.push('/');
                s
            })
            .collect();
        s.pop(); // remove last '/'
        s
    } else {
        slug::slugify(index.to_kstr().into_string())
    }
}

fn interpret_permalink(
    config: &PaginationConfig,
    doc: &Document,
    page_num: usize,
    index: Option<&liquid::model::Value>,
) -> Result<String> {
    let mut attributes = document::permalink_attributes(&doc.front, &doc.file_path);
    let permalink = permalink::explode_permalink(&config.front_permalink, &attributes)?;
    let permalink_path = std::path::Path::new(&permalink);
    let pagination_root = permalink_path.extension().map_or_else(
        || permalink.clone(),
        |os_str| {
            permalink
                .trim_end_matches(&format!(".{}", os_str.to_string_lossy()))
                .to_string()
        },
    );
    let interpreted_permalink = if page_num == 1 {
        index.map_or_else(
            || doc.url_path.clone(),
            |index| {
                if pagination_root.is_empty() {
                    index_to_string(index)
                } else {
                    format!("{}/{}", pagination_root, index_to_string(index))
                }
            },
        )
    } else {
        let pagination_attr = pagination_attributes(page_num as i32);
        attributes.extend(pagination_attr.into_iter());
        let index = index.map_or_else(
            || {
                if config.include != Include::All {
                    unreachable!("Include is not All and no index");
                }
                "all".to_string()
            },
            |index| index_to_string(index),
        );
        if pagination_root.is_empty() {
            format!(
                "{}/{}",
                index,
                permalink::explode_permalink(&config.permalink_suffix, &attributes)?
            )
        } else {
            format!(
                "{}/{}/{}",
                pagination_root,
                index,
                permalink::explode_permalink(&config.permalink_suffix, &attributes)?
            )
        }
    };
    Ok(interpreted_permalink)
}
