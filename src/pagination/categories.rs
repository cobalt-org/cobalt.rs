use std::collections::BTreeMap;

use crate::document::Document;

use super::{
    PaginationConfig, Result, ValueView, create_all_paginators, helpers, paginator, sort_posts,
};
use helpers::extract_categories;
use paginator::Paginator;

pub(crate) fn create_categories_paginators(
    all_posts: &[&liquid::model::Value],
    doc: &Document,
    pagination_cfg: &PaginationConfig,
) -> Result<Vec<Paginator>> {
    let mut root_cat = distribute_posts_by_categories(all_posts)?;
    let paginators_holder = walk_categories(&mut root_cat, pagination_cfg, doc)?;
    Ok(paginators_holder)
}

fn distribute_posts_by_categories<'a>(
    all_posts: &[&'a liquid::model::Value],
) -> Result<Category<'a>> {
    let mut root = Category::new();
    for post in all_posts {
        if let Some(categories) = extract_categories(post.as_view()) {
            let categories: Vec<_> = categories.values().collect();
            parse_categories_list(&mut root, categories.as_slice(), post)?;
        }
    }
    Ok(root)
}

/// construct a hierarchy of Categories with their posts from a list of categories
fn parse_categories_list<'a>(
    mut parent: &mut Category<'a>,
    post_categories: &[&dyn ValueView],
    post: &'a liquid::model::Value,
) -> Result<()> {
    for i in 0..post_categories.len() {
        let cat_name = post_categories[i].to_kstr().to_string();
        parent = parent
            .sub_cats
            .entry(cat_name)
            .or_insert_with(|| Category::with_path(post_categories[0..=i].iter().copied()));
    }
    parent.add_post(post);
    Ok(())
}

#[derive(Default, Debug)]
struct Category<'a> {
    cat_path: liquid::model::Array,
    posts: Vec<&'a liquid::model::Value>,
    sub_cats: BTreeMap<String, Category<'a>>,
}

impl<'a> Category<'a> {
    fn new() -> Self {
        Default::default()
    }

    fn with_path<'v>(path: impl Iterator<Item = &'v dyn ValueView>) -> Self {
        let mut c = Self::new();
        c.cat_path = path.map(|v| v.to_value()).collect();
        c
    }

    fn add_post(&mut self, post: &'a liquid::model::Value) {
        self.posts.push(post);
    }
}

// walk the categories tree and construct Paginator for each node,
// filling `pages` and `indexes` accordingly
fn walk_categories(
    category: &mut Category<'_>,
    config: &PaginationConfig,
    doc: &Document,
) -> Result<Vec<Paginator>> {
    let mut cur_cat_paginators_holder: Vec<Paginator> = vec![];
    if !category.cat_path.is_empty() {
        sort_posts(&mut category.posts, config);
        let cur_cat_paginators = create_all_paginators(
            &category.posts,
            doc,
            config,
            Some(&liquid::model::Value::array(category.cat_path.clone())),
        )?;
        if !cur_cat_paginators.is_empty() {
            cur_cat_paginators_holder.extend(cur_cat_paginators);
        } else {
            let p = Paginator {
                index_title: Some(liquid::model::Value::array(category.cat_path.clone())),
                ..Default::default()
            };
            cur_cat_paginators_holder.push(p);
        }
    } else {
        cur_cat_paginators_holder.push(Paginator::default());
    }
    for c in category.sub_cats.values_mut() {
        let mut sub_paginators_holder = walk_categories(c, config, doc)?;

        if let Some(indexes) = cur_cat_paginators_holder[0].indexes.as_mut() {
            indexes.push(sub_paginators_holder[0].clone());
        } else {
            cur_cat_paginators_holder[0].indexes = Some(vec![sub_paginators_holder[0].clone()]);
        }
        cur_cat_paginators_holder.append(&mut sub_paginators_holder);
    }
    Ok(cur_cat_paginators_holder)
}
