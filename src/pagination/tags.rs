use std::collections::HashMap;

use crate::cobalt_model::pagination_config::PaginationConfig;
use crate::cobalt_model::slug;
use crate::document::Document;

use super::*;
use helpers::extract_tags;
use paginator::Paginator;

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
                let cur_tag = per_tags.entry(tag.to_string()).or_insert_with(|| vec![]);
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

pub fn create_tags_paginators(
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
        .or_else(std::result::Result::<_, failure::Error>::Err)?;

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
