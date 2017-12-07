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
        // Convert legacy frontmatter into frontmatter (with `data`)
        // In some cases, we need to remove some values due to processing done by later tools
        // Otherwise, we can remove the converted values because most frontmatter content gets
        // populated into the final attributes (see `document_attributes`).
        // Exceptions
        // - excerpt_separator: internal-only
        // - extends internal-only
        let mut unprocessed_attributes = legacy.0;
        cobalt_model::FrontmatterBuilder::new()
            .merge_title(unprocessed_attributes
                             .remove("title")
                             .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_description(unprocessed_attributes
                                   .remove("description")
                                   .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_excerpt(unprocessed_attributes
                               .remove("excerpt")
                               .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_categories(unprocessed_attributes.remove("categories").and_then(|v| {
                v.as_array()
                    .map(|v| v.iter().map(|v| v.to_string()).collect())
            }))
            .merge_slug(unprocessed_attributes
                            .remove("slug")
                            .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_permalink(unprocessed_attributes
                                 .remove("path")
                                 .and_then(|v| v.as_str().map(|s| convert_permalink(s.to_owned()))))
            .merge_draft(unprocessed_attributes
                             .remove("draft")
                             .and_then(|v| v.as_bool()))
            .merge_excerpt_separator(unprocessed_attributes
                                         .remove("excerpt_separator")
                                         .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_layout(unprocessed_attributes
                              .remove("extends")
                              .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_published_date(unprocessed_attributes.remove("date").and_then(|d| {
                d.as_str().and_then(cobalt_model::DateTime::parse)
            }))
            .merge_data(unprocessed_attributes)
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
