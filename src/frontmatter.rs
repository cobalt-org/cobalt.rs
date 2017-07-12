use std::path;
use liquid;

use error::Result;
use datetime;
use slug;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SourceFormat {
    Raw,
    Markdown,
}

impl Default for SourceFormat {
    fn default() -> SourceFormat {
        SourceFormat::Raw
    }
}

#[derive(Debug, Eq, PartialEq, Default, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct FrontmatterBuilder {
    pub path: Option<String>,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub categories: Option<Vec<String>>,
    pub excerpt_separator: Option<String>,
    pub published_date: Option<datetime::DateTime>,
    pub format: Option<SourceFormat>,
    pub layout: Option<String>,
    pub is_draft: Option<bool>,
    // Controlled by where the file is found.  We might allow control over the type at a later
    // point but we need to first define those semantics.
    #[serde(skip)]
    pub is_post: Option<bool>,
    pub custom: liquid::Object,
}

impl FrontmatterBuilder {
    pub fn new() -> FrontmatterBuilder {
        FrontmatterBuilder::default()
    }

    pub fn set_permalink<S: Into<Option<String>>>(self, permalink: S) -> FrontmatterBuilder {
        FrontmatterBuilder {
            path: permalink.into(),
            ..self
        }
    }

    pub fn set_slug<S: Into<Option<String>>>(self, slug: S) -> FrontmatterBuilder {
        FrontmatterBuilder {
            slug: slug.into(),
            ..self
        }
    }

    pub fn set_title<S: Into<Option<String>>>(self, title: S) -> FrontmatterBuilder {
        FrontmatterBuilder {
            title: title.into(),
            ..self
        }
    }

    #[cfg(test)]
    pub fn set_description<S: Into<Option<String>>>(self, description: S) -> FrontmatterBuilder {
        FrontmatterBuilder {
            description: description.into(),
            ..self
        }
    }

    #[cfg(test)]
    pub fn set_categories<S: Into<Option<Vec<String>>>>(self, categories: S) -> FrontmatterBuilder {
        FrontmatterBuilder {
            categories: categories.into(),
            ..self
        }
    }

    pub fn set_excerpt_separator<S: Into<Option<String>>>(self,
                                                          excerpt_separator: S)
                                                          -> FrontmatterBuilder {
        FrontmatterBuilder {
            excerpt_separator: excerpt_separator.into(),
            ..self
        }
    }

    pub fn set_published_date<D: Into<Option<datetime::DateTime>>>(self,
                                                                   published_date: D)
                                                                   -> FrontmatterBuilder {
        FrontmatterBuilder {
            published_date: published_date.into(),
            ..self
        }
    }

    #[cfg(test)]
    pub fn set_format<S: Into<Option<SourceFormat>>>(self, format: S) -> FrontmatterBuilder {
        FrontmatterBuilder {
            format: format.into(),
            ..self
        }
    }

    pub fn set_layout<S: Into<Option<String>>>(self, layout: S) -> FrontmatterBuilder {
        FrontmatterBuilder {
            layout: layout.into(),
            ..self
        }
    }

    pub fn set_draft<B: Into<Option<bool>>>(self, is_draft: B) -> FrontmatterBuilder {
        FrontmatterBuilder {
            is_draft: is_draft.into(),
            ..self
        }
    }

    pub fn set_post<B: Into<Option<bool>>>(self, is_post: B) -> FrontmatterBuilder {
        FrontmatterBuilder {
            is_post: is_post.into(),
            ..self
        }
    }

    pub fn merge_permalink<S: Into<Option<String>>>(self, permalink: S) -> FrontmatterBuilder {
        self.merge(FrontmatterBuilder::new().set_permalink(permalink.into()))
    }

    pub fn merge_slug<S: Into<Option<String>>>(self, slug: S) -> FrontmatterBuilder {
        self.merge(FrontmatterBuilder::new().set_slug(slug.into()))
    }

    pub fn merge_title<S: Into<Option<String>>>(self, title: S) -> FrontmatterBuilder {
        self.merge(FrontmatterBuilder::new().set_title(title.into()))
    }

    #[cfg(test)]
    pub fn merge_description<S: Into<Option<String>>>(self, description: S) -> FrontmatterBuilder {
        self.merge(FrontmatterBuilder::new().set_description(description.into()))
    }

    #[cfg(test)]
    pub fn merge_categories<S: Into<Option<Vec<String>>>>(self,
                                                          categories: S)
                                                          -> FrontmatterBuilder {
        self.merge(FrontmatterBuilder::new().set_categories(categories.into()))
    }

    pub fn merge_excerpt_separator<S: Into<Option<String>>>(self,
                                                            excerpt_separator: S)
                                                            -> FrontmatterBuilder {
        self.merge(FrontmatterBuilder::new().set_excerpt_separator(excerpt_separator.into()))
    }

    pub fn merge_published_date<D: Into<Option<datetime::DateTime>>>(self,
                                                                     published_date: D)
                                                                     -> FrontmatterBuilder {
        self.merge(FrontmatterBuilder::new().set_published_date(published_date.into()))
    }

    #[cfg(test)]
    pub fn merge_format<S: Into<Option<SourceFormat>>>(self, format: S) -> FrontmatterBuilder {
        self.merge(FrontmatterBuilder::new().set_format(format.into()))
    }

    pub fn merge_layout<S: Into<Option<String>>>(self, layout: S) -> FrontmatterBuilder {
        self.merge(FrontmatterBuilder::new().set_layout(layout.into()))
    }

    pub fn merge_draft<B: Into<Option<bool>>>(self, draft: B) -> FrontmatterBuilder {
        self.merge(FrontmatterBuilder::new().set_draft(draft.into()))
    }

    pub fn merge_post<B: Into<Option<bool>>>(self, post: B) -> FrontmatterBuilder {
        self.merge(FrontmatterBuilder::new().set_post(post.into()))
    }

    pub fn merge(self, other: FrontmatterBuilder) -> FrontmatterBuilder {
        let FrontmatterBuilder {
            path,
            slug,
            title,
            description,
            categories,
            excerpt_separator,
            published_date,
            format,
            layout,
            is_draft,
            is_post,
            custom,
        } = self;
        let FrontmatterBuilder {
            path: other_path,
            slug: other_slug,
            title: other_title,
            description: other_description,
            categories: other_categories,
            excerpt_separator: other_excerpt_separator,
            published_date: other_published_date,
            format: other_format,
            layout: other_layout,
            is_draft: other_is_draft,
            is_post: other_is_post,
            custom: other_custom,
        } = other;
        FrontmatterBuilder {
            path: path.or_else(|| other_path),
            slug: slug.or_else(|| other_slug),
            title: title.or_else(|| other_title),
            description: description.or_else(|| other_description),
            categories: categories.or_else(|| other_categories),
            excerpt_separator: excerpt_separator.or_else(|| other_excerpt_separator),
            published_date: published_date.or_else(|| other_published_date),
            format: format.or_else(|| other_format),
            layout: layout.or_else(|| other_layout),
            is_draft: is_draft.or_else(|| other_is_draft),
            is_post: is_post.or_else(|| other_is_post),
            custom: merge_objects(custom, &other_custom),
        }
    }

    pub fn merge_path<P: AsRef<path::Path>>(self, relpath: P) -> Result<FrontmatterBuilder> {
        self.merge_path_ref(relpath.as_ref())
    }

    fn merge_path_ref(self, relpath: &path::Path) -> Result<FrontmatterBuilder> {
        let mut fm = self;

        if fm.format.is_none() {
            let ext = relpath
                .extension()
                .and_then(|os| os.to_str())
                .unwrap_or("");
            let format = match ext {
                "md" => SourceFormat::Markdown,
                "liquid" => SourceFormat::Raw,
                // TODO Evaluate making this an error when we break compatibility
                _ => SourceFormat::Raw,
            };
            fm.format = Some(format);
        }

        if fm.slug.is_none() {
            let file_stem = file_stem(relpath);
            let slug = slug::slugify(file_stem);
            fm.slug = Some(slug);
        }

        if fm.title.is_none() {
            let slug = fm.slug
                .as_ref()
                .expect("slug has been unconditionally initialized");
            let title = slug::titleize_slug(slug);
            fm.title = Some(title);
        }

        Ok(fm)
    }

    pub fn build(self) -> Result<Frontmatter> {
        let FrontmatterBuilder {
            path,
            slug,
            title,
            description,
            categories,
            excerpt_separator,
            published_date,
            format,
            layout,
            is_draft,
            is_post,
            custom,
        } = self;

        let is_post = is_post.unwrap_or(false);

        let path = path.unwrap_or_else(|| {
            let default_path = if is_post {
                "/:categories/:year/:month/:day/:slug.html"
            } else {
                "/:path/:slug.:output_ext"
            };

            default_path.to_owned()
        });

        let fm = Frontmatter {
            path: path,
            slug: slug.ok_or_else(|| "No slug")?,
            title: title.ok_or_else(|| "No title")?,
            description: description,
            categories: categories.unwrap_or_else(|| vec![]),
            excerpt_separator: excerpt_separator.unwrap_or_else(|| "\n\n".to_owned()),
            published_date: published_date,
            format: format.unwrap_or_else(SourceFormat::default),
            layout: layout,
            is_draft: is_draft.unwrap_or(false),
            is_post: is_post,
            custom: custom,
        };

        Ok(fm)
    }
}

#[derive(Debug, Eq, PartialEq, Default, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Frontmatter {
    pub path: String,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub categories: Vec<String>,
    pub excerpt_separator: String,
    pub published_date: Option<datetime::DateTime>,
    pub format: SourceFormat,
    pub layout: Option<String>,
    pub is_draft: bool,
    #[serde(skip)]
    pub is_post: bool,
    pub custom: liquid::Object,
}

/// Shallow merge of liquid::Object's
fn merge_objects(primary: liquid::Object, secondary: &liquid::Object) -> liquid::Object {
    let mut primary = primary;
    for (key, value) in secondary.iter() {
        primary
            .entry(key.to_owned())
            .or_insert_with(|| value.clone());
    }
    primary
}

/// The base-name without an extension.  Correlates to Jekyll's :name path tag
fn file_stem<P: AsRef<path::Path>>(p: P) -> String {
    file_stem_path(p.as_ref())
}

fn file_stem_path(p: &path::Path) -> String {
    p.file_stem()
        .map(|os| os.to_string_lossy().into_owned())
        .unwrap_or_else(|| "".to_owned())
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn file_stem_absolute_path() {
        let input = path::PathBuf::from("/embedded/path/___filE-worlD-__09___.md");
        let actual = file_stem(input.as_path());
        assert_eq!(actual, "___filE-worlD-__09___");
    }

    #[test]
    fn frontmatter_title_from_path() {
        let front = FrontmatterBuilder::new()
            .merge_path("./parent/file.md")
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(front.title, "File");
    }

    #[test]
    fn frontmatter_slug_from_md_path() {
        let front = FrontmatterBuilder::new()
            .merge_path("./parent/file.md")
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(front.slug, "file");
    }

    #[test]
    fn frontmatter_markdown_from_path() {
        let front = FrontmatterBuilder::new()
            .merge_path("./parent/file.md")
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(front.format, SourceFormat::Markdown);
    }

    #[test]
    fn frontmatter_raw_from_path() {
        let front = FrontmatterBuilder::new()
            .merge_path("./parent/file.liquid")
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(front.format, SourceFormat::Raw);
    }

    #[test]
    fn frontmatter_global_merge() {
        let empty = FrontmatterBuilder::new();
        let a = FrontmatterBuilder {
            path: Some("path a".to_owned()),
            slug: Some("slug a".to_owned()),
            title: Some("title a".to_owned()),
            description: Some("description a".to_owned()),
            categories: Some(vec!["a".to_owned(), "b".to_owned()]),
            excerpt_separator: Some("excerpt_separator a".to_owned()),
            published_date: Some(datetime::DateTime::default()),
            format: Some(SourceFormat::Markdown),
            layout: Some("layout a".to_owned()),
            is_draft: Some(true),
            is_post: Some(false),
            custom: liquid::Object::new(),
        };
        let b = FrontmatterBuilder {
            path: Some("path b".to_owned()),
            slug: Some("slug b".to_owned()),
            title: Some("title b".to_owned()),
            description: Some("description b".to_owned()),
            categories: Some(vec!["b".to_owned(), "a".to_owned()]),
            excerpt_separator: Some("excerpt_separator b".to_owned()),
            published_date: Some(datetime::DateTime::default()),
            format: Some(SourceFormat::Raw),
            layout: Some("layout b".to_owned()),
            is_draft: Some(true),
            is_post: Some(false),
            custom: liquid::Object::new(),
        };

        let merge_b_into_a = a.clone().merge(b.clone());
        assert_eq!(merge_b_into_a, a);

        let merge_empty_into_a = a.clone().merge(empty.clone());
        assert_eq!(merge_empty_into_a, a);

        let merge_a_into_empty = empty.clone().merge(a.clone());
        assert_eq!(merge_a_into_empty, a);
    }

    #[test]
    fn frontmatter_local_merge() {
        let a = FrontmatterBuilder {
            path: Some("path a".to_owned()),
            slug: Some("slug a".to_owned()),
            title: Some("title a".to_owned()),
            description: Some("description a".to_owned()),
            categories: Some(vec!["a".to_owned(), "b".to_owned()]),
            excerpt_separator: Some("excerpt_separator a".to_owned()),
            published_date: None,
            format: Some(SourceFormat::Markdown),
            layout: Some("layout a".to_owned()),
            is_draft: Some(true),
            is_post: Some(false),
            custom: liquid::Object::new(),
        };

        let merge_b_into_a = a.clone()
            .merge_permalink("path b".to_owned())
            .merge_slug("slug b".to_owned())
            .merge_title("title b".to_owned())
            .merge_description("description b".to_owned())
            .merge_categories(vec!["a".to_owned(), "b".to_owned()])
            .merge_excerpt_separator("excerpt_separator b".to_owned())
            .merge_format(SourceFormat::Raw)
            .merge_layout("layout b".to_owned())
            .merge_draft(true)
            .merge_post(false);
        assert_eq!(merge_b_into_a, a);

        let merge_empty_into_a = a.clone()
            .merge_permalink(None)
            .merge_slug(None)
            .merge_title(None)
            .merge_description(None)
            .merge_categories(None)
            .merge_excerpt_separator(None)
            .merge_format(None)
            .merge_layout(None)
            .merge_draft(None)
            .merge_post(None);
        assert_eq!(merge_empty_into_a, a);

        let merge_a_into_empty = FrontmatterBuilder::new()
            .merge_permalink("path a".to_owned())
            .merge_slug("slug a".to_owned())
            .merge_title("title a".to_owned())
            .merge_description("description a".to_owned())
            .merge_categories(vec!["a".to_owned(), "b".to_owned()])
            .merge_excerpt_separator("excerpt_separator a".to_owned())
            .merge_format(SourceFormat::Markdown)
            .merge_layout("layout a".to_owned())
            .merge_draft(true)
            .merge_post(false);
        assert_eq!(merge_a_into_empty, a);
    }

    #[test]
    fn frontmatter_defaults() {
        FrontmatterBuilder::new()
            .set_title("Title".to_owned())
            .set_slug("Slug".to_owned())
            .build()
            .unwrap();
    }
}
