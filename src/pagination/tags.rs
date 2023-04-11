use std::collections::HashMap;

use crate::cobalt_model::pagination::PaginationConfig;
use crate::cobalt_model::slug;
use crate::document::Document;

use super::*;
use helpers::extract_tags;
use paginator::Paginator;

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
                let cur_tag = per_tags.entry(tag).or_insert_with(Vec::new);
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

#[allow(clippy::bind_instead_of_map)]
pub fn create_tags_paginators(
    all_posts: &[&liquid::model::Value],
    doc: &Document,
    pagination_cfg: &PaginationConfig,
) -> Result<Vec<Paginator>> {
    let mut per_tags = distribute_posts_by_tags(all_posts)?;

    // create all other paginators
    let mut tag_paginators: TagPaginators = per_tags
        .iter_mut()
        .try_fold(TagPaginators::default(), |mut acc, (tag, posts)| {
            sort_posts(posts, pagination_cfg);
            let cur_tag_paginators = create_all_paginators(
                posts,
                doc,
                pagination_cfg,
                Some(&liquid::model::Value::scalar(tag.to_owned())),
            )?;
            acc.firsts_of_tags.push(cur_tag_paginators[0].clone());
            acc.paginators.extend(cur_tag_paginators.into_iter());
            Ok(acc)
        })
        .or_else(std::result::Result::<_, anyhow::Error>::Err)?;

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
