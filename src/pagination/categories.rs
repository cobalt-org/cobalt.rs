use std::cmp::Ordering;

use crate::document::Document;

use super::*;
use helpers::extract_categories;
use paginator::Paginator;

#[derive(Debug)]
struct Category<'a> {
    cat_path: liquid::model::Array,
    posts: Vec<&'a liquid::model::Value>,
    sub_cats: Vec<Category<'a>>,
}

impl<'a> Category<'a> {
    fn new() -> Self {
        Category {
            cat_path: vec![],
            posts: vec![],
            sub_cats: vec![],
        }
    }

    fn with_path<'v>(path: impl Iterator<Item = &'v dyn liquid::ValueView>) -> Self {
        Category {
            cat_path: path.map(|v| v.to_value()).collect(),
            posts: vec![],
            sub_cats: vec![],
        }
    }

    fn add_post(&mut self, post: &'a liquid::model::Value) {
        self.posts.push(&post)
    }
}

fn compare_category_path<'a, C, S>(cur_path: C, seek: S) -> Ordering
where
    C: Iterator<Item = &'a dyn liquid::ValueView>,
    S: Iterator<Item = &'a dyn liquid::ValueView>,
{
    cur_path
        .map(liquid::model::value::ValueViewCmp::new)
        .partial_cmp(seek.map(liquid::model::value::ValueViewCmp::new))
        .expect("Arrays of same hierarchy level should be fully comparable")
}

fn is_leaf_category(cur_idx: usize, categories: &[&dyn liquid::ValueView]) -> bool {
    cur_idx == categories.len()
}

fn construct_cat_full_path<'v>(
    cur_idx: usize,
    categories: &[&'v dyn liquid::model::ValueView],
) -> Vec<&'v dyn liquid::model::ValueView> {
    categories[..cur_idx].to_vec()
}

fn next_category(cur_idx: usize) -> usize {
    cur_idx + 1
}

// construct a hierarchy of Categories with their posts from a list of categories
fn parse_categories_list<'a, 'b>(
    parent: &'b mut Category<'a>,
    cur_idx: usize,
    cur_post_categories: &[&dyn liquid::ValueView],
    post: &'a liquid::model::Value,
) -> Result<()> {
    if cur_idx <= cur_post_categories.len() {
        let cat_full_path = construct_cat_full_path(cur_idx, cur_post_categories);
        let mut cur_cat = if let Ok(idx) = parent.sub_cats.binary_search_by(|c| {
            compare_category_path(
                c.cat_path.iter().map(|v| v.as_view()),
                cat_full_path.iter().map(|v| *v),
            )
        }) {
            &mut parent.sub_cats[idx]
        } else {
            let last_idx = parent.sub_cats.len();
            parent
                .sub_cats
                .push(Category::with_path(cat_full_path.into_iter()));
            // need to sort for binary_search_by
            parent.sub_cats.sort_by(|c1, c2| {
                compare_category_path(
                    c1.cat_path.iter().map(|v| v.as_view()),
                    c2.cat_path.iter().map(|v| v.as_view()),
                )
            });
            &mut parent.sub_cats[last_idx]
        };

        if is_leaf_category(cur_idx, cur_post_categories) {
            cur_cat.add_post(&post);
        } else {
            parse_categories_list(
                &mut cur_cat,
                next_category(cur_idx),
                &cur_post_categories,
                &post,
            )?;
        }
    }
    Ok(())
}

fn distribute_posts_by_categories<'a>(
    all_posts: &[&'a liquid::model::Value],
) -> Result<Category<'a>> {
    let mut root = Category::new();
    for post in all_posts {
        if let Some(categories) = extract_categories(post.as_view()) {
            let categories: Vec<_> = categories.values().collect();
            parse_categories_list(&mut root, 1, categories.as_slice(), &post)?;
        }
    }
    Ok(root)
}

// walk the categories tree and construct Paginator for each node,
// filling `pages` and `indexes` accordingly
fn walk_categories<'a, 'b>(
    category: &'b mut Category<'a>,
    config: &cobalt_model::page::Pagination,
    doc: &Document,
) -> Result<Vec<Paginator>> {
    let mut cur_cat_paginators_holder: Vec<Paginator> = vec![];
    if !category.cat_path.is_empty() {
        sort_posts(&mut category.posts, &config);
        let cur_cat_paginators = create_all_paginators(
            &category.posts,
            doc,
            &config,
            Some(&liquid::model::Value::array(category.cat_path.clone())),
        )?;
        if !cur_cat_paginators.is_empty() {
            cur_cat_paginators_holder.extend(cur_cat_paginators.into_iter());
        } else {
            let mut p = Paginator::default();
            p.index_title = Some(liquid::model::Value::array(category.cat_path.clone()));
            cur_cat_paginators_holder.push(p);
        }
    } else {
        cur_cat_paginators_holder.push(Paginator::default());
    }
    for mut c in &mut category.sub_cats {
        let mut sub_paginators_holder = walk_categories(&mut c, &config, doc)?;

        if let Some(indexes) = cur_cat_paginators_holder[0].indexes.as_mut() {
            indexes.push(sub_paginators_holder[0].clone());
        } else {
            cur_cat_paginators_holder[0].indexes = Some(vec![sub_paginators_holder[0].clone()]);
        }
        cur_cat_paginators_holder.append(&mut sub_paginators_holder);
    }
    Ok(cur_cat_paginators_holder)
}

pub fn create_categories_paginators(
    all_posts: &[&liquid::model::Value],
    doc: &Document,
    pagination_cfg: &cobalt_model::page::Pagination,
) -> Result<Vec<Paginator>> {
    let mut root_cat = distribute_posts_by_categories(&all_posts)?;
    let paginators_holder = walk_categories(&mut root_cat, &pagination_cfg, doc)?;
    Ok(paginators_holder)
}

#[cfg(test)]
mod test {
    use super::*;

    use liquid::model::ArrayView;

    #[test]
    fn compare_category_path_test() {
        let a = liquid::model::array!(["A"]);
        let b = liquid::model::array!(["B"]);
        assert_eq!(
            Ordering::Less,
            compare_category_path(a.values(), b.values())
        );
        assert_eq!(
            Ordering::Greater,
            compare_category_path(b.values(), a.values())
        );
    }
}
