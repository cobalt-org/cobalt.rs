use std::collections::HashMap;

use crate::cobalt_model::pagination::PaginationConfig;
use crate::cobalt_model::slug;
use crate::document::Document;

use super::{Result, ValueView, all, helpers, paginator, sort_posts};
use helpers::extract_tags;
use paginator::Paginator;

#[allow(clippy::bind_instead_of_map)]
pub(crate) fn create_tags_paginators(
    all_posts: &[&liquid::model::Value],
    doc: &Document,
    config: &PaginationConfig,
) -> Result<Vec<Paginator>> {
    let mut per_tags = distribute_posts_by_tags(all_posts)?;
    walk_tags(&mut per_tags, config, doc)
}

fn distribute_posts_by_tags<'a>(
    all_posts: &[&'a liquid::model::Value],
) -> Result<HashMap<String, Vec<&'a liquid::model::Value>>> {
    let mut per_tags: HashMap<String, Vec<&'a liquid::model::Value>> = HashMap::new();
    for post in all_posts {
        if let Some(tags) = extract_tags(post.as_view()) {
            for tag in tags.values() {
                let tag = tag
                    .as_scalar()
                    .ok_or_else(|| anyhow::format_err!("Should have string tags"))?
                    .to_kstr()
                    .into_string();
                let cur_tag = per_tags.entry(tag).or_default();
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

fn walk_tags(
    per_tags: &mut HashMap<String, Vec<&liquid::model::Value>>,
    config: &PaginationConfig,
    doc: &Document,
) -> Result<Vec<Paginator>> {
    // create all other paginators
    let mut tag_paginators = TagPaginators::default();
    for (tag, posts) in per_tags.iter_mut() {
        sort_posts(posts, config);
        let cur_tag_paginators = all::create_all_paginators(
            posts,
            doc,
            config,
            Some(&liquid::model::Value::scalar(tag.to_owned())),
        )?;
        tag_paginators
            .firsts_of_tags
            .push(cur_tag_paginators[0].clone());
        tag_paginators
            .paginators
            .extend(cur_tag_paginators.into_iter());
    }

    tag_paginators.firsts_of_tags.sort_unstable_by_key(|p| {
        p.index_title
            .as_ref()
            .map(|index_title| slug::slugify(index_title.to_kstr()).to_lowercase())
    });
    let first = Paginator {
        indexes: Some(tag_paginators.firsts_of_tags),
        ..Default::default()
    };
    tag_paginators.paginators.insert(0, first);
    Ok(tag_paginators.paginators)
}
