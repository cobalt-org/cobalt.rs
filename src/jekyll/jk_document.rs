use std::ffi;
use std::fs;
use std::io::Write;
use std::path;

use liquid;
use regex;
use serde_yaml;

use files;
use frontmatter;
use datetime;
use legacy::wildwest;
use jekyll::jk_errors::{ErrorKind, Result};

lazy_static! {
    static ref FRONT_MATTER_DIVIDE: regex::Regex = regex::Regex::new(r"---\s*\r?\n").unwrap();
}

#[derive(Debug, Eq, PartialEq, Default, Clone, Serialize, Deserialize)]
pub struct JkFrontmatterBuilder(liquid::Object);

impl JkFrontmatterBuilder {
    pub fn new() -> JkFrontmatterBuilder {
        JkFrontmatterBuilder(liquid::Object::new())
    }
}

impl From<JkFrontmatterBuilder> for wildwest::FrontmatterBuilder {
    fn from(jk_front: JkFrontmatterBuilder) -> Self {
        // Convert jekyll frontmatter into frontmatter (with `custom`)
        let mut custom_attributes = jk_front.0;
        let front = frontmatter::FrontmatterBuilder::new()
            .merge_slug(custom_attributes
                            .remove("slug")
                            .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_title(custom_attributes
                             .remove("title")
                             .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_description(custom_attributes
                                   .remove("excerpt")
                                   .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_categories(custom_attributes.remove("categories").and_then(|v| {
                v.as_array()
                    .map(|v| v.iter().map(|v| v.to_string()).collect())
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
                                      .and_then(|d| d.as_str().and_then(datetime::DateTime::parse)))
            .merge_custom(custom_attributes);
        front.into()
    }
}

fn convert_document(doc: String) -> Result<wildwest::DocumentBuilder> {
    let (front, content) = split_document(&doc)?;
    let front: JkFrontmatterBuilder = front
        .map(|f| serde_yaml::from_str(f))
        .unwrap_or_else(|| Ok(JkFrontmatterBuilder::default()))?;
    let front: wildwest::FrontmatterBuilder = front.into();
    let content = content.to_owned();

    Ok(wildwest::DocumentBuilder { front, content })
}

fn convert_document_file(source_file: &path::Path, dest_dir: &path::Path) -> Result<()> {
    let doc = files::read_file(source_file)?;
    let doc = convert_document(doc)?;
    let doc = doc.to_string();
    let dest_file = dest_dir.join(source_file.with_extension("md").file_name().unwrap());

    if !dest_dir.exists() {
        fs::create_dir_all(&dest_dir)?;
    }
    let mut dest = fs::File::create(dest_file)?;
    dest.write_all(doc.as_bytes())?;
    Ok(())
}

pub fn convert_from_jk(source: &path::Path, dest: &path::Path) -> Result<()> {
    if dest.is_file() {
        Err(ErrorKind::CantOutputInFile.into())
    } else if source.is_file() {
        convert_document_file(source, dest)
    } else if source.is_dir() {
        for file in source.read_dir()? {
            if let Ok(file) = file {
                let file_path = file.path();
                let ext = file_path.extension().unwrap_or_else(|| ffi::OsStr::new(""));
                if file_path.is_file() {
                    if ext == "md" || ext == "markdown" {
                        convert_document_file(&file.path(), dest)?
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

fn split_document(content: &str) -> Result<(Option<&str>, &str)> {
    if FRONT_MATTER_DIVIDE.is_match(content) {
        let mut splits = FRONT_MATTER_DIVIDE.splitn(content, 3);
        let first = splits.next().unwrap_or("");
        let second = splits.next().unwrap_or("");
        let third = splits.next().unwrap_or("");

        if !first.is_empty() {
            bail!("Invalid leading text in frontmatter: {:?}", first);
        }
        if second.is_empty() {
            Ok((None, third))
        } else {
            Ok((Some(second), third))
        }
    } else {
        Ok((None, content))
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn split_document_empty_document() {
        let fixture = "";

        let (front, content) = split_document(fixture).unwrap();
        assert!(front.is_none());
        assert_eq!(content, "");
    }

    #[test]
    fn split_document_empty_sections() {
        let fixture = "---\n---\n";

        let (front, content) = split_document(fixture).unwrap();
        assert!(front.is_none());
        assert_eq!(content, "");
    }

    #[test]
    fn split_document_content_only() {
        let fixture = "Content\n";

        let (front, content) = split_document(fixture).unwrap();
        assert!(front.is_none());
        assert_eq!(content, "Content\n");
    }

    #[test]
    fn split_document_empty_front() {
        let fixture = "---\n---\nContent\n";

        let (front, content) = split_document(fixture).unwrap();
        assert!(front.is_none());
        assert_eq!(content, "Content\n");
    }

    #[test]
    fn split_document_empty_content() {
        let fixture = "---\ntitle: test_post\n---\n";

        let (front, content) = split_document(fixture).unwrap();
        assert_eq!(front.unwrap(), "title: test_post\n");
        assert_eq!(content, "");
    }

    #[test]
    fn split_document_all_sections() {
        let fixture = "---\ntitle: test_post\n---\nContent\n";

        let (front, content) = split_document(fixture).unwrap();
        assert_eq!(front.unwrap(), "title: test_post\n");
        assert_eq!(content, "Content\n");
    }

    #[test]
    fn frontmatter_empty() {
        let front = JkFrontmatterBuilder::default();
        let _front: wildwest::FrontmatterBuilder = front.into();

        // TODO(epage): Confirm jekyll defaults overrode cobalt defaults
    }

    static FIXTURE_FULL: &str = r#"title: test_post
date: 2017-05-03T20:55:07+00:00
layout: post
permalink: /2017/05/03/test_post/
categories:
  - cat1
  - cat2
"#;

    #[test]
    fn frontmatter_full() {
        let front: JkFrontmatterBuilder = serde_yaml::from_str(FIXTURE_FULL).unwrap();
        let front: wildwest::FrontmatterBuilder = front.into();
        let front = front.to_string();
        let front: liquid::Object = serde_yaml::from_str(&front).unwrap();

        let expected: liquid::Object =
            [("path".to_owned(), liquid::Value::str("/2017/05/03/test_post/")),
             ("title".to_owned(), liquid::Value::str("test_post")),
             ("extends".to_owned(), liquid::Value::str("post")),
             ("categories".to_owned(),
              liquid::Value::Array(vec![liquid::Value::str("cat1"), liquid::Value::str("cat2")]))]
                .iter()
                .cloned()
                .collect();
        assert_eq!(front, expected);
    }

    static FIXTURE_CUSTOM: &str = r#"id: 33
title: test_post
author: TheAuthor
guid: http://url.com/?p=33
tags:
  - tag1
  - tag2
  - tag3
"#;

    #[test]
    fn frontmatter_custom() {
        let front: JkFrontmatterBuilder = serde_yaml::from_str(FIXTURE_CUSTOM).unwrap();
        let front: wildwest::FrontmatterBuilder = front.into();
        let front = front.to_string();
        let front: liquid::Object = serde_yaml::from_str(&front).unwrap();

        let expected: liquid::Object = [("id".to_owned(), liquid::Value::Num(33.0f32)),
                                        ("title".to_owned(), liquid::Value::str("test_post")),
                                        ("author".to_owned(), liquid::Value::str("TheAuthor")),
                                        ("guid".to_owned(),
                                         liquid::Value::str("http://url.com/?p=33")),
                                        ("tags".to_owned(),
                                         liquid::Value::Array(vec![liquid::Value::str("tag1"),
                                                                   liquid::Value::str("tag2"),
                                                                   liquid::Value::str("tag3")]))]
            .iter()
            .cloned()
            .collect();
        assert_eq!(front, expected);
    }

    static FIXTURE_MINIMAL: &str = r#"title: test_post"#;
    static EXPECTED_MINIMAL: &str = r#"title: test_post"#;

    #[test]
    fn parse_string_ok() {
        let fixture = format!("---\n{}---\nthe content\n", FIXTURE_MINIMAL);

        let doc = convert_document(fixture).unwrap();
        let actual = doc.to_string();

        let expected = format!("{}\n---\nthe content\n", EXPECTED_MINIMAL);
        assert_eq!(actual, expected);
    }
}
