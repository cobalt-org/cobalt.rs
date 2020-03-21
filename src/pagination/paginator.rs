use crate::document::Document;

use super::*;

#[derive(Default, Clone, Debug)]
pub struct Paginator {
    pub pages: Option<Vec<liquid::model::Value>>,
    pub indexes: Option<Vec<Paginator>>,
    pub index: usize,
    pub index_title: Option<liquid::model::Value>,
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
        config: &cobalt_model::page::Pagination,
        total_pages: usize,
        index_title: Option<&liquid::model::Value>,
    ) -> Result<()> {
        self.first_index_permalink = doc.url_path.to_string();
        self.last_index_permalink = interpret_permalink(&config, &doc, total_pages, index_title)?;
        Ok(())
    }

    pub fn set_current_index_info(
        &mut self,
        index: usize,
        all_pages: &[&liquid::model::Value],
        config: &cobalt_model::page::Pagination,
        doc: &Document,
        index_title: Option<&liquid::model::Value>,
    ) -> Result<()> {
        self.index = index;
        self.pages = Some(all_pages.iter().map(|p| (*p).clone()).collect());
        self.index_title = index_title.cloned();
        self.index_permalink = interpret_permalink(&config, &doc, index, index_title)?;
        Ok(())
    }

    pub fn set_previous_next_info(
        &mut self,
        index: usize,
        total_indexes: usize,
        doc: &Document,
        config: &cobalt_model::page::Pagination,
        index_title: Option<&liquid::model::Value>,
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

pub fn create_paginator(
    i: usize,
    total_indexes: usize,
    total_pages: usize,
    config: &cobalt_model::page::Pagination,
    doc: &Document,
    all_posts: &[&liquid::model::Value],
    index_title: Option<&liquid::model::Value>,
) -> Result<Paginator> {
    let index = i + 1;
    let mut paginator = Paginator::new(total_indexes, total_pages);

    paginator.set_first_last(&doc, &config, total_indexes, index_title)?;
    paginator.set_current_index_info(index, &all_posts, &config, &doc, index_title)?;
    paginator.set_previous_next_info(index, total_indexes, &doc, &config, index_title)?;

    Ok(paginator)
}

impl Into<liquid::Object> for Paginator {
    fn into(self) -> liquid::Object {
        let mut object = liquid::Object::new();
        // if no pages, means we have indexes instead, `tags` like cases for exemple
        if let Some(pages) = self.pages {
            object.insert("pages".into(), liquid::model::Value::Array(pages));
        }
        // list of available indexes, in `tags` like cases
        if let Some(indexes) = self.indexes {
            object.insert(
                "indexes".into(),
                liquid::model::Value::Array(
                    indexes
                        .into_iter()
                        .map(|paginator| {
                            let v: liquid::Object = paginator.into();
                            liquid::model::Value::Object(v)
                        })
                        .collect(),
                ),
            );
        }
        object.insert(
            "index".into(),
            liquid::model::Value::scalar(self.index as i32),
        );
        object.insert(
            "index_permalink".into(),
            liquid::model::Value::scalar(self.index_permalink),
        );
        if let Some(index_title) = self.index_title {
            object.insert("index_title".into(), index_title);
        }
        if let Some(previous_index_permalink) = self.previous_index_permalink {
            object.insert(
                "previous_index".into(),
                liquid::model::Value::scalar(self.previous_index as i32),
            );
            object.insert(
                "previous_index_permalink".into(),
                liquid::model::Value::scalar(previous_index_permalink),
            );
        }
        if let Some(next_index_permalink) = self.next_index_permalink {
            object.insert(
                "next_index".into(),
                liquid::model::Value::scalar(self.next_index as i32),
            );
            object.insert(
                "next_index_permalink".into(),
                liquid::model::Value::scalar(next_index_permalink),
            );
        }
        object.insert(
            "first_index_permalink".into(),
            liquid::model::Value::scalar(self.first_index_permalink),
        );
        object.insert(
            "last_index_permalink".into(),
            liquid::model::Value::scalar(self.last_index_permalink),
        );
        object.insert(
            "total_indexes".into(),
            liquid::model::Value::scalar(self.total_indexes as i32),
        );
        object.insert(
            "total_pages".into(),
            liquid::model::Value::scalar(self.total_pages as i32),
        );
        object
    }
}
