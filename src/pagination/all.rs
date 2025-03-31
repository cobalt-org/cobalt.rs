use crate::cobalt_model::pagination::PaginationConfig;
use crate::document::Document;

use super::{Result, paginator};
use paginator::Paginator;

pub(crate) fn create_all_paginators(
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
