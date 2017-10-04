use liquid;

use super::super::frontmatter;
use super::super::datetime;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::Path;
use super::super::document::{read_file, split_document};
use frontmatter::FrontmatterBuilder;
use error::{ErrorKind, Result};
use serde_yaml;

#[derive(Debug, Eq, PartialEq, Default, Clone)]
#[derive(Serialize, Deserialize)]
pub struct JkFrontmatterBuilder(liquid::Object);

impl JkFrontmatterBuilder {
    pub fn new() -> JkFrontmatterBuilder {
        JkFrontmatterBuilder(liquid::Object::new())
    }
}

impl From<JkFrontmatterBuilder> for frontmatter::FrontmatterBuilder {
    fn from(jk_front: JkFrontmatterBuilder) -> Self {
        // Convert jekyll frontmatter into frontmatter (with `custom`)
        let mut custom_attributes = jk_front.0;
        frontmatter::FrontmatterBuilder::new()
            .merge_title(custom_attributes.remove("title").and_then(|v| {
                v.as_str().map(|s| s.to_owned())
            }))
            .merge_slug(custom_attributes.remove("permalink").and_then(|v| {
                v.as_str().map(|s| s.to_owned())
            }))
            .merge_description(custom_attributes.remove("excerpt").and_then(|v| {
                v.as_str().map(|s| s.to_owned())
            }))
            .merge_categories(custom_attributes.remove("categories").and_then(|v| {
                v.as_array().map(
                    |v| v.iter().map(|v| v.to_string()).collect(),
                )
            }))
            .merge_permalink(custom_attributes.remove("permalink").and_then(|v| {
                v.as_str().map(|s| s.to_owned())
            }))
            .merge_draft(custom_attributes.remove("published").and_then(|v| {
                v.as_bool().map(|b| !b)
            }))
            .merge_layout(custom_attributes.remove("layout").and_then(|v| {
                v.as_str().map(|s| s.to_owned())
            }))
            .merge_published_date(custom_attributes.remove("date").and_then(|d| {
                d.as_str().and_then(datetime::DateTime::parse)
            }))
            .merge_custom(custom_attributes)
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct JkDocument {
    pub front: Option<String>,
    pub content: Option<String>,
}

impl JkDocument {
    pub fn parse_string(doc: String) -> Result<JkDocument> {
        let (front, content) = split_document(&doc)?;
        if front != None {
            Err(ErrorKind::MissingFrontStart.into())
        } else {
            match split_document(content)? {
                (None, _) => Err(ErrorKind::MissingFrontmatter.into()),
                (Some(front), content) => Ok(JkDocument {
                    front: Some(front.to_owned()),
                    content: Some(content.to_owned()),
                }),
            }
        }
    }

    pub fn parse(source_file: &Path) -> Result<JkDocument> {
        let doc: String = read_file(source_file)?;
        JkDocument::parse_string(doc)
    }

    pub fn convert_front(front: Option<String>) -> Result<String> {
        let front_value: JkFrontmatterBuilder = front
            .map(|s| serde_yaml::from_str(&s))
            .map_or(Ok(None), |r| r.map(Some))?
            .unwrap_or_else(JkFrontmatterBuilder::new);

        let front_builder: FrontmatterBuilder = front_value.into();
        let front = front_builder.build()?;
        let mut converted = serde_yaml::to_string(&front)?;
        converted.drain(..4);
        Ok(converted)
    }

    pub fn convert(source_file: &Path, dest_dir: &Path) -> Result<()> {
        let doc = JkDocument::parse(source_file)?;
        let front = JkDocument::convert_front(doc.front)?;
        if !dest_dir.exists() {
            create_dir_all(&dest_dir)?;
        }
        let dest_file = dest_dir.join(source_file.file_name().unwrap());
        let mut dest = File::create(dest_file)?;
        let converted = format!("{}\n---\n{}", &front, &doc.content.unwrap());
        dest.write_all(converted.as_bytes())?;
        Ok(())
    }
}

pub fn convert_from_jk(source: &Path, dest: &Path) -> Result<()> {
    if dest.is_file() {
        Err(ErrorKind::CantOutputInFile.into())
    } else if source.is_file() {
        JkDocument::convert(source, dest)
    } else if source.is_dir() {
        for file in source.read_dir()? {
            if let Ok(file) = file {
                JkDocument::convert(&file.path(), dest)?
            }
        }
        Ok(())
    } else {
        Err(ErrorKind::InternalError.into())
    }
}


#[cfg(test)]
mod test {
    use super::*;

    // can't have custom: the order of fields is not stable
    fn get_correct_cb_front() -> &'static str {
        r#"path: "/:path/:filename:output_ext"
slug: /2017/05/03/test_post/
title: test_post
description: ~
categories:
  - cat1
  - cat2
excerpt_separator: "\n\n"
published_date: ~
format: Raw
layout: post
is_draft: false
"#
    }

    fn get_correct_jk_front() -> &'static str {
        r#"id: 33
title: test_post
date: 2017-05-03T20:55:07+00:00
author: TheAuthor
layout: post
guid: http://url.com/?p=33
permalink: /2017/05/03/test_post/
categories:
  - cat1
  - cat2
tags:
  - tag1
  - tag2
  - tag3
"#
    }

    fn get_correct_content() -> &'static str {
        "the content\n"
    }

    #[test]
    fn parse_string_ok() {
        let correct_front = get_correct_jk_front();
        let correct_content = get_correct_content();
        let correct_doc = format!("---\n{}---\n{}", correct_front, correct_content);

        let res = JkDocument::parse_string(correct_doc);
        assert!(res.is_ok());
        let doc = res.unwrap();
        assert_eq!(doc.content.unwrap(), correct_content);
        assert_eq!(doc.front.unwrap(), correct_front);
    }

    #[test]
    fn parse_string_no_front() {
        let res = JkDocument::parse_string(get_correct_content().to_owned());
        assert!(res.is_err());
        //assert_eq!(res.unwrap_err(), ErrorKind::MissingFrontmatter);
    }

    #[test]
    fn parse_string_no_front_starter() {
        let correct_front = get_correct_jk_front();
        let correct_content = get_correct_content();
        let correct_doc = format!("{}---\n{}", correct_front, correct_content);
        let res = JkDocument::parse_string(correct_doc);

        assert!(res.is_err());
        //assert_eq!(res.unwrap_err(), ErrorKind::MissingFrontStart);
    }

    #[test]
    fn convert_front_ok() {
        let correct_front = get_correct_jk_front();
        let res = JkDocument::convert_front(Some(correct_front.to_owned()));
        match res {
            Err(e) => println!("error convert: {:#?}", e),
            Ok(mut converted) => {
                // need to remove the custom part, the fields order is not stable
                let custom_offset = converted.find("custom").unwrap_or(converted.len());
                let all_but_custom: String = converted.drain(..custom_offset).collect();
                assert_eq!(all_but_custom, get_correct_cb_front())
            }
        }
    }
}
