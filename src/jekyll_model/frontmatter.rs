use std::fmt;

use liquid;
use serde_yaml;

use error::*;
use cobalt_model;

#[derive(Debug, Eq, PartialEq, Default, Clone, Serialize, Deserialize)]
pub struct FrontmatterBuilder(liquid::Object);

impl FrontmatterBuilder {
    pub fn new() -> FrontmatterBuilder {
        FrontmatterBuilder(liquid::Object::new())
    }

    pub fn parse(content: &str) -> Result<Self> {
        let front: Self = serde_yaml::from_str(content)?;
        Ok(front)
    }

    fn to_string(&self) -> Result<String> {
        let mut converted = serde_yaml::to_string(self)?;
        converted.drain(..4);
        Ok(converted)
    }
}

impl fmt::Display for FrontmatterBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let converted = self.to_string().map_err(|_| fmt::Error)?;
        write!(f, "{}", converted)
    }
}

impl From<FrontmatterBuilder> for cobalt_model::FrontmatterBuilder {
    fn from(jk_front: FrontmatterBuilder) -> Self {
        // Convert jekyll frontmatter into frontmatter (with `custom`)
        let mut custom_attributes = jk_front.0;
        cobalt_model::FrontmatterBuilder::new()
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
            .merge_published_date(custom_attributes.remove("date").and_then(|d| {
                d.as_str().and_then(cobalt_model::DateTime::parse)
            }))
            .merge_custom(custom_attributes)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn frontmatter_empty() {
        let front = FrontmatterBuilder::default();
        let _front: cobalt_model::FrontmatterBuilder = front.into();

        // TODO(epage): Confirm jekyll defaults overrode cobalt defaults
    }
}
