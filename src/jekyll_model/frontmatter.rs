use std::fmt;

use liquid;
use serde_yaml;

use error::*;
use cobalt_model;
use super::Permalink;
use super::Part;
use super::VARIABLES;

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
        let mut unprocessed_attributes = jk_front.0;
        cobalt_model::FrontmatterBuilder::new()
            .merge_slug(unprocessed_attributes
                            .remove("slug")
                            .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_title(unprocessed_attributes
                             .remove("title")
                             .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_description(unprocessed_attributes
                                   .remove("excerpt")
                                   .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_categories(unprocessed_attributes.remove("categories").and_then(|v| {
                v.as_array()
                    .map(|v| v.iter().map(|v| v.to_string()).collect())
            }))
            .merge_permalink(unprocessed_attributes
                                 .remove("permalink")
                                 .and_then(|v| v.as_str().map(convert_permalink)))
            .merge_draft(unprocessed_attributes
                             .remove("published")
                             .and_then(|v| v.as_bool().map(|b| !b)))
            .merge_layout(unprocessed_attributes
                              .remove("layout")
                              .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_published_date(unprocessed_attributes.remove("date").and_then(|d| {
                d.as_str().and_then(cobalt_model::DateTime::parse)
            }))
            .merge_data(unprocessed_attributes)
    }
}

fn migrate_variable(var: String) -> Part {
    let native_variable = {
        let name: &str = &var;
        VARIABLES.contains(&name)
    };
    let var = match var.as_str() {
        "path" => "parent".to_owned(),
        "filename" => "name".to_owned(),
        "output_ext" => "ext".to_owned(),
        x => x.to_owned(),
    };
    let variable = if native_variable {
        format!("{{{{ {} }}}}", var)
    } else {
        format!("{{{{ data.{} }}}}", var)
    };

    Part::Constant(variable)
}

fn convert_permalink(perma: &str) -> String {
    let perma = Permalink::parse(perma);
    let perma = perma.resolve(&migrate_variable);
    perma.to_string()
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

    #[test]
    fn migrate_variable_known() {
        let fixture = "path".to_owned();
        let expected = Part::Constant("{{ parent }}".to_owned());
        let actual = migrate_variable(fixture);
        assert_eq!(actual, expected);
    }

    #[test]
    fn migrate_variable_unknown() {
        let fixture = "gobbly/gook".to_owned();
        let expected = Part::Constant("{{ data.gobbly/gook }}".to_owned());
        let actual = migrate_variable(fixture);
        assert_eq!(actual, expected);
    }

    #[test]
    fn convert_permalink_empty() {
        assert_eq!(convert_permalink(""), "/".to_owned());
    }

    #[test]
    fn convert_permalink_abs() {
        assert_eq!(convert_permalink("/root"), "/root".to_owned());
    }

    #[test]
    fn convert_permalink_rel() {
        assert_eq!(convert_permalink("rel"), "/rel".to_owned());
    }

    #[test]
    fn convert_permalink_known_variable() {
        assert_eq!(convert_permalink("hello/:path/world/:i_day/"),
                   "/hello/{{ parent }}/world/{{ i_day }}/".to_owned());
    }

    #[test]
    fn convert_permalink_unknown_variable() {
        assert_eq!(convert_permalink("hello/:party/world/:i_day/"),
                   "/hello/{{ data.party/world/ }}{{ i_day }}/".to_owned());
    }
}
