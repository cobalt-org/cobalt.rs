use liquid;

use super::super::frontmatter;
use super::super::datetime;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::Path;
use frontmatter::FrontmatterBuilder;
use super::jk_errors::{ErrorKind, Result};
use serde_yaml;
use regex::Regex;
use slug::slugify;
use std::ffi::OsStr;

lazy_static! {
    static ref FRONT_MATTER_DIVIDE: Regex = Regex::new(r"---\s*\r?\n").unwrap();
}

#[derive(Debug, Eq, PartialEq, Default, Clone, Serialize, Deserialize)]
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
            .merge_slug(custom_attributes
                            .remove("slug")
                            .and_then(|v| v.as_str().map(|s| s.to_owned()))
                            .or_else(|| {
                                         Some(slugify(custom_attributes
                                 .get("title")
                                 .unwrap_or(&liquid::Value::str("No Title"))
                                 .as_str()
                                 .unwrap()))
                                     })
                            .map(|s| s.to_owned()))
            .merge_title(custom_attributes
                             .remove("title")
                             .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_description(custom_attributes
                                   .remove("excerpt")
                                   .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_categories(custom_attributes
                                  .remove("categories")
                                  .and_then(|v| {
                                                v.as_array()
                                                    .map(|v| {
                                                             v.iter()
                                                                 .map(|v| v.to_string())
                                                                 .collect()
                                                         })
                                            }))
            .merge_permalink(custom_attributes
                                 .remove("permalink")
                                 .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_draft(custom_attributes
                             .remove("published")
                             .and_then(|v| v.as_bool().map(|b| !b)))
            .merge_layout(custom_attributes
                              .remove("layout")
                              .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_published_date(custom_attributes
                                      .remove("date")
                                      .and_then(|d| {
                                                    d.as_str().and_then(datetime::DateTime::parse)
                                                }))
            .merge_custom(custom_attributes)
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct JkDocument {
    pub front: Option<String>,
    pub content: String,
}

impl JkDocument {
    pub fn parse_string(doc: String) -> Result<JkDocument> {
        let (front, content) = split_document(&doc)?;
        Ok(JkDocument {
               front: front.map(|s| s.to_owned()),
               content: content.to_owned(),
           })
    }

    pub fn parse(source_file: &Path) -> Result<JkDocument> {
        let doc: String = read_file(source_file)?;
        JkDocument::parse_string(doc)
    }

    pub fn convert_front(front: String) -> Result<String> {
        let front_value: JkFrontmatterBuilder = serde_yaml::from_str(&front)?;
        let front_builder: FrontmatterBuilder = front_value.into();
        let front = front_builder.build()?;
        let mut converted = serde_yaml::to_string(&front)?;
        converted.drain(..4);
        Ok(converted)
    }

    pub fn convert(source_file: &Path, dest_dir: &Path) -> Result<()> {
        let doc = JkDocument::parse(source_file)?;
        let front = match doc.front {
            Some(front) => Some(JkDocument::convert_front(front)?),
            None => None,
        };

        if !dest_dir.exists() {
            create_dir_all(&dest_dir)?;
        }
        let dest_file = dest_dir.join(source_file.with_extension("md").file_name().unwrap());
        let mut dest = File::create(dest_file)?;
        let converted = match front {
            Some(front) => format!("{}\n---\n{}", &front, &doc.content),
            None => doc.content,
        };
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
                let file_path = file.path();
                let ext = file_path.extension().unwrap_or(OsStr::new(""));
                if file_path.is_file() {
                    if ext == "md" || ext == "markdown" {
                        JkDocument::convert(&file.path(), dest)?
                    } else {
                        warn!("unsupported file extension")
                    }
                } else {
                    warn!("sub directory parsing is not supported yet")
                }
            }
        }
        Ok(())
    } else {
        Err(ErrorKind::InternalError.into())
    }
}

fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = File::open(path.as_ref())?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    Ok(text)
}

fn split_document(content: &str) -> Result<(Option<&str>, &str)> {
    if FRONT_MATTER_DIVIDE.is_match(content) {
        let mut splits = FRONT_MATTER_DIVIDE.splitn(content, 3);
        let first = splits.next().unwrap_or("");
        let second = splits.next().unwrap_or("");
        let third = splits.next().unwrap_or("");

        if third.is_empty() {
            // only one "---"
            if first.is_empty() {
                Ok((None, second))
            } else {
                Ok((Some(first), second))
            }
        } else {
            // first should be empty
            if second.is_empty() {
                Ok((None, third))
            } else {
                Ok((Some(second), third))
            }
        }
    } else {
        Ok((None, content))
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    // can't have custom: the order of fields is not stable
    // can't use r# strings because of https://github.com/rust-lang-nursery/rustfmt/issues/878
    static CORRECT_CB_FRONT: &str = "path: /2017/05/03/test_post/\n\
                                     slug: \"test-post\"\n\
                                     title: test_post\n\
                                     description: ~\n\
                                     categories: \n  - cat1\n  - cat2\n\
                                     excerpt_separator: \"\\n\\n\"\n\
                                     published_date: ~\n\
                                     format: Raw\n\
                                     layout: post\n\
                                     is_draft: false\n";

    static CORRECT_JK_FRONT: &str = r#"id: 33
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
"#;

    static CORRECT_CONTENT: &str = "the content\n";

    #[test]
    fn parse_string_ok() {
        let correct_doc = format!("---\n{}---\n{}", CORRECT_JK_FRONT, CORRECT_CONTENT);

        let res = JkDocument::parse_string(correct_doc);
        assert!(res.is_ok());
        let doc = res.unwrap();
        assert_eq!(doc.content, CORRECT_CONTENT);
        assert_eq!(doc.front.unwrap(), CORRECT_JK_FRONT);
    }

    #[test]
    fn parse_string_no_front() {
        let res = JkDocument::parse_string(CORRECT_CONTENT.to_owned());
        assert!(res.is_ok());
        let doc = res.unwrap();
        assert_eq!(doc.content, CORRECT_CONTENT);
    }

    #[test]
    fn parse_string_no_front_starter() {
        let correct_doc = format!("{}---\n{}", CORRECT_JK_FRONT, CORRECT_CONTENT);
        let res = JkDocument::parse_string(correct_doc);
        assert!(res.is_ok());
        let doc = res.unwrap();
        assert_eq!(doc.content, CORRECT_CONTENT);
        assert_eq!(doc.front.unwrap(), CORRECT_JK_FRONT);
    }

    #[test]
    fn convert_front_ok() {
        let res = JkDocument::convert_front(CORRECT_JK_FRONT.to_owned());
        match res {
            Err(e) => println!("error convert: {:#?}", e),
            Ok(mut converted) => {
                // need to remove the custom part, the fields order is not stable
                let custom_offset = converted.find("custom").unwrap_or(converted.len());
                let all_but_custom: String = converted.drain(..custom_offset).collect();
                assert_eq!(all_but_custom, CORRECT_CB_FRONT);
                let customs_builder: FrontmatterBuilder =
                    serde_yaml::from_str(&converted).expect("serde yaml failed");

                let customs: frontmatter::Frontmatter = customs_builder
                    .merge_slug("dummy".to_owned())
                    .merge_title("dummy".to_owned())
                    .build()
                    .ok()
                    .expect("build failed");

                let expected: HashMap<String, liquid::Value> =
                    [("guid".to_owned(), liquid::Value::str("http://url.com/?p=33")),
                     ("id".to_owned(), liquid::Value::Num(33.0f32)),
                     ("author".to_owned(), liquid::Value::str("TheAuthor")),
                     ("tags".to_owned(),
                      liquid::Value::Array(vec![liquid::Value::str("tag1"),
                                                liquid::Value::str("tag2"),
                                                liquid::Value::str("tag3")]))]
                            .iter()
                            .cloned()
                            .collect();
                assert_eq!(expected, customs.custom);
            }
        }
    }
}
