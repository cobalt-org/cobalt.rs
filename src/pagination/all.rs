use crate::cobalt_model::pagination::PaginationConfig;
use crate::document::Document;

use super::{Result, paginator};
use paginator::Paginator;

pub(crate) fn create_all_paginators(
    all_posts: &[&liquid::model::Value],
    doc: &Document,
    config: &PaginationConfig,
    index_title: Option<&liquid::model::Value>,
) -> Result<Vec<Paginator>> {
    let total_pages = all_posts.len();
    // f32 used here in order to not lose information to ceil the result,
    // otherwise we can lose an index
    let total_indexes = (total_pages as f32 / config.per_page as f32).ceil() as usize;

    if total_pages == 0 {
        return Ok(vec![paginator::create_paginator(
            0,
            total_indexes,
            total_pages,
            config,
            doc,
            &[],
            index_title,
        )?]);
    }

    let paginators: Result<Vec<_>> = all_posts
        .chunks(config.per_page as usize)
        .enumerate()
        .map(|(i, chunk)| {
            paginator::create_paginator(
                i,
                total_indexes,
                total_pages,
                config,
                doc,
                chunk,
                index_title,
            )
        })
        .collect();
    paginators
}
