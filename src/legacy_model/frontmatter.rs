use std::fmt;

use liquid;

use cobalt_model;

#[derive(Debug, Eq, PartialEq, Default, Clone)]
#[derive(Serialize, Deserialize)]
pub struct FrontmatterBuilder(liquid::Object);

impl FrontmatterBuilder {
    pub fn new() -> Self {
        FrontmatterBuilder(liquid::Object::new())
    }

    pub fn with_object(obj: liquid::Object) -> Self {
        FrontmatterBuilder(obj)
    }

    pub fn object(self) -> liquid::Object {
        self.0
    }
}

impl From<FrontmatterBuilder> for cobalt_model::FrontmatterBuilder {
    fn from(legacy: FrontmatterBuilder) -> Self {
        // Convert legacy frontmatter into frontmatter (with `custom`)
        // In some cases, we need to remove some values due to processing done by later tools
        // Otherwise, we can remove the converted values because most frontmatter content gets
        // populated into the final attributes (see `document_attributes`).
        // Exceptions
        // - excerpt_separator: internal-only
        // - extends internal-only
        let mut custom_attributes = legacy.0;
        cobalt_model::FrontmatterBuilder::new()
            .merge_title(custom_attributes
                             .remove("title")
                             .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_description(custom_attributes
                                   .remove("description")
                                   .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_categories(custom_attributes.remove("categories").and_then(|v| {
                v.as_array()
                    .map(|v| v.iter().map(|v| v.to_string()).collect())
            }))
            .merge_slug(custom_attributes
                            .remove("slug")
                            .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_permalink(custom_attributes
                                 .remove("path")
                                 .and_then(|v| v.as_str().map(|s| convert_permalink(s.to_owned()))))
            .merge_draft(custom_attributes.remove("draft").and_then(|v| v.as_bool()))
            .merge_excerpt_separator(custom_attributes
                                         .remove("excerpt_separator")
                                         .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_layout(custom_attributes
                              .remove("extends")
                              .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_published_date(custom_attributes.remove("date").and_then(|d| {
                d.as_str().and_then(cobalt_model::DateTime::parse)
            }))
            .merge_custom(custom_attributes)
    }
}

impl From<cobalt_model::FrontmatterBuilder> for FrontmatterBuilder {
    fn from(internal: cobalt_model::FrontmatterBuilder) -> Self {
        let mut legacy = liquid::Object::new();

        let cobalt_model::FrontmatterBuilder {
            permalink,
            slug,
            title,
            description,
            categories,
            excerpt_separator,
            published_date,
            format: _format,
            layout,
            is_draft,
            is_post: _is_post,
            custom,
        } = internal;
        if let Some(path) = permalink {
            legacy.insert("path".to_owned(), liquid::Value::Str(path));
        }
        if let Some(slug) = slug {
            legacy.insert("slug".to_owned(), liquid::Value::Str(slug));
        }
        if let Some(title) = title {
            legacy.insert("title".to_owned(), liquid::Value::Str(title));
        }
        if let Some(description) = description {
            legacy.insert("description".to_owned(), liquid::Value::Str(description));
        }
        if let Some(categories) = categories {
            legacy.insert("categories".to_owned(),
                          liquid::Value::Array(categories
                                                   .into_iter()
                                                   .map(liquid::Value::Str)
                                                   .collect()));
        }
        if let Some(excerpt_separator) = excerpt_separator {
            legacy.insert("excerpt_separator".to_owned(),
                          liquid::Value::Str(excerpt_separator));
        }
        if let Some(date) = published_date {
            legacy.insert("date".to_owned(), liquid::Value::Str(date.format()));
        }
        if let Some(extends) = layout {
            legacy.insert("extends".to_owned(), liquid::Value::Str(extends));
        }
        if let Some(draft) = is_draft {
            legacy.insert("draft".to_owned(), liquid::Value::Bool(draft));
        }
        for (key, value) in custom {
            legacy.insert(key, value);
        }

        FrontmatterBuilder(legacy)
    }
}

impl fmt::Display for FrontmatterBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let converted = cobalt_model::Front::to_string(self)
            .map_err(|_| fmt::Error)?;
        write!(f, "{}", converted)
    }
}

impl cobalt_model::Front for FrontmatterBuilder {}

fn convert_permalink(mut perma: String) -> String {
    if perma.starts_with('/') {
        perma
    } else {
        perma.insert(0, '/');
        perma
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_permalink_empty() {
        assert_eq!(convert_permalink("".into()), "/");
    }

    #[test]
    fn convert_permalink_abs() {
        assert_eq!(convert_permalink("/root".into()), "/root");
    }

    #[test]
    fn convert_permalink_rel() {
        assert_eq!(convert_permalink("rel".into()), "/rel");
    }
}
