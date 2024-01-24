use crate::cobalt_model::pagination::DateIndex;
use crate::cobalt_model::DateTime;
use crate::document::Document;

use super::*;
use helpers::extract_scalar;
use paginator::Paginator;

#[derive(Debug, Clone)]
struct DateIndexHolder<'a> {
    value: u32,
    field: Option<DateIndex>,
    posts: Vec<&'a liquid::model::Value>,
    sub_date: Vec<DateIndexHolder<'a>>,
}

impl<'a> DateIndexHolder<'a> {
    fn new(value: u32, field: Option<DateIndex>) -> Self {
        DateIndexHolder {
            value,
            field,
            posts: vec![],
            sub_date: vec![],
        }
    }
}

fn extract_published_date(value: &'_ dyn liquid::ValueView) -> Option<DateTime> {
    let published_date = extract_scalar(value, "published_date")?;
    published_date.to_date_time()
}

pub fn create_dates_paginators(
    all_posts: &[&liquid::model::Value],
    doc: &Document,
    pagination_cfg: &PaginationConfig,
) -> Result<Vec<Paginator>> {
    let mut root_date = distribute_posts_by_dates(all_posts, pagination_cfg)?;
    walk_dates(&mut root_date, pagination_cfg, doc, None)
}

fn format_date_holder(d: &DateIndexHolder<'_>) -> liquid::model::Value {
    let field = d
        .field
        .expect("Should not be called with the root DateIndexHolder");
    let formatted = match field {
        DateIndex::Year => d.value.to_string(),
        _ => format!("{:02}", d.value),
    };
    liquid::model::Value::scalar(formatted)
}

fn date_fields_to_array(date: &[DateIndexHolder<'_>]) -> liquid::model::Array {
    date.iter().map(format_date_holder).collect()
}

fn walk_dates(
    date_holder: &mut DateIndexHolder<'_>,
    config: &PaginationConfig,
    doc: &Document,
    parent_dates: Option<Vec<DateIndexHolder<'_>>>,
) -> Result<Vec<Paginator>> {
    let mut cur_date_holder_paginators: Vec<Paginator> = vec![];
    let mut current_date = if let Some(parent_dates) = parent_dates {
        parent_dates
    } else {
        vec![]
    };
    if let Some(_field) = date_holder.field {
        sort_posts(&mut date_holder.posts, config);
        current_date.push(date_holder.clone());
        let index_title = liquid::model::Value::array(date_fields_to_array(&current_date));
        let cur_date_paginators =
            create_all_paginators(&date_holder.posts, doc, config, Some(&index_title))?;
        if !cur_date_paginators.is_empty() {
            cur_date_holder_paginators.extend(cur_date_paginators);
        } else {
            let p = Paginator {
                index_title: Some(index_title),
                ..Default::default()
            };
            cur_date_holder_paginators.push(p);
        }
    } else {
        cur_date_holder_paginators.push(Paginator::default());
    }
    for dh in &mut date_holder.sub_date {
        let mut sub_paginators_holder = walk_dates(dh, config, doc, Some(current_date.clone()))?;

        if let Some(indexes) = cur_date_holder_paginators[0].indexes.as_mut() {
            indexes.push(sub_paginators_holder[0].clone());
        } else {
            cur_date_holder_paginators[0].indexes = Some(vec![sub_paginators_holder[0].clone()]);
        }
        cur_date_holder_paginators.append(&mut sub_paginators_holder);
    }
    Ok(cur_date_holder_paginators)
}

fn find_or_create_date_holder_and_put_post<'a>(
    date_holder: &mut DateIndexHolder<'a>,
    published_date: &DateTime,
    wanted_field: DateIndex,
    post: &'a liquid::model::Value,
) {
    let value = get_date_field_value(published_date, wanted_field);
    let mut not_found = true;
    for dh in date_holder.sub_date.iter_mut() {
        let dh_field = dh
            .field
            .expect("Only root has None, we should always have a field");
        if dh_field < wanted_field {
            // not at the level we want but still need to find the correct parent
            // parent should have been created in a previous loop
            let parent_value = get_date_field_value(published_date, dh_field);
            if dh.value == parent_value {
                find_or_create_date_holder_and_put_post(dh, published_date, wanted_field, post);
                not_found = false;
            }
        } else if dh_field == wanted_field && dh.value == value {
            dh.posts.push(post);
            not_found = false;
        }
    }
    // not found create one
    if not_found {
        let mut holder = DateIndexHolder::new(value, Some(wanted_field));
        holder.posts.push(post);
        date_holder.sub_date.push(holder);
    }
}

fn get_date_field_value(date: &DateTime, field: DateIndex) -> u32 {
    match field {
        DateIndex::Year => {
            if date.year() < 0 {
                panic!("Negative year is not supported");
            }
            date.year() as u32
        }
        DateIndex::Month => date.month() as u32,
        DateIndex::Day => date.day() as u32,
        DateIndex::Hour => date.hour() as u32,
        DateIndex::Minute => date.minute() as u32,
    }
}

fn distribute_posts_by_dates<'a>(
    all_posts: &[&'a liquid::model::Value],
    pagination_cfg: &PaginationConfig,
) -> Result<DateIndexHolder<'a>> {
    let date_index = &pagination_cfg.date_index;
    let mut root = DateIndexHolder::new(0u32, None);
    for post in all_posts {
        if let Some(published_date) = extract_published_date(post.as_view()) {
            for idx in date_index {
                find_or_create_date_holder_and_put_post(&mut root, &published_date, *idx, post);
            }
        }
    }
    Ok(root)
}
