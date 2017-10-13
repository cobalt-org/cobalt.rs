use std::default::Default;

use liquid;

use super::super::config;
use super::super::frontmatter;
use super::super::datetime;

#[derive(Debug, Eq, PartialEq, Default, Clone)]
#[derive(Serialize, Deserialize)]
pub struct FrontmatterBuilder(liquid::Object);

impl FrontmatterBuilder {
    pub fn new() -> FrontmatterBuilder {
        FrontmatterBuilder(liquid::Object::new())
    }
}

impl From<FrontmatterBuilder> for frontmatter::FrontmatterBuilder {
    fn from(legacy: FrontmatterBuilder) -> Self {
        // Convert legacy frontmatter into frontmatter (with `custom`)
        // In some cases, we need to remove some values due to processing done by later tools
        // Otherwise, we can remove the converted values because most frontmatter content gets
        // populated into the final attributes (see `document_attributes`).
        // Exceptions
        // - excerpt_separator: internal-only
        // - extends internal-only
        let mut custom_attributes = legacy.0;
        frontmatter::FrontmatterBuilder::new()
            .merge_title(custom_attributes
                             .remove("title")
                             .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_description(custom_attributes
                                   .remove("description")
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
            .merge_slug(custom_attributes
                            .remove("slug")
                            .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_permalink(custom_attributes
                                 .remove("path")
                                 .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_draft(custom_attributes
                             .remove("draft")
                             .and_then(|v| v.as_bool()))
            .merge_excerpt_separator(custom_attributes
                                         .remove("excerpt_separator")
                                         .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_layout(custom_attributes
                              .remove("extends")
                              .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_published_date(custom_attributes
                                      .remove("date")
                                      .and_then(|d| {
                                                    d.as_str().and_then(datetime::DateTime::parse)
                                                }))
            .merge_custom(custom_attributes)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SassOutputStyle {
    Nested,
    Expanded,
    Compact,
    Compressed,
}

#[derive(Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SassOptions {
    pub import_dir: String,
    pub style: SassOutputStyle,
}

impl Default for SassOptions {
    fn default() -> SassOptions {
        SassOptions {
            import_dir: "_sass".to_owned(),
            style: SassOutputStyle::Nested,
        }
    }
}

#[derive(Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyntaxHighlight {
    pub theme: String,
}

impl Default for SyntaxHighlight {
    fn default() -> SyntaxHighlight {
        SyntaxHighlight { theme: "base16-ocean.dark".to_owned() }
    }
}

#[derive(Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct GlobalConfig {
    pub source: String,
    pub dest: String,
    pub layouts: String,
    pub drafts: String,
    pub include_drafts: bool,
    pub posts: String,
    pub post_path: Option<String>,
    pub post_order: String,
    pub template_extensions: Vec<String>,
    pub rss: Option<String>,
    pub jsonfeed: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub ignore: Vec<String>,
    pub excerpt_separator: String,
    pub syntax_highlight: SyntaxHighlight,
    pub sass: SassOptions,
}

impl Default for GlobalConfig {
    fn default() -> GlobalConfig {
        GlobalConfig {
            source: "./".to_owned(),
            dest: "./".to_owned(),
            layouts: "_layouts".to_owned(),
            drafts: "_drafts".to_owned(),
            include_drafts: false,
            posts: "posts".to_owned(),
            post_path: None,
            post_order: "desc".to_owned(),
            template_extensions: vec!["md".to_owned(), "liquid".to_owned()],
            rss: None,
            jsonfeed: None,
            name: None,
            description: None,
            link: None,
            ignore: vec![],
            excerpt_separator: "\n\n".to_owned(),
            syntax_highlight: SyntaxHighlight::default(),
            sass: SassOptions::default(),
        }
    }
}

impl From<GlobalConfig> for config::Config {
    fn from(legacy: GlobalConfig) -> Self {
        let GlobalConfig {
            source,
            dest,
            layouts,
            drafts,
            include_drafts,
            posts,
            post_path,
            post_order,
            template_extensions,
            rss,
            jsonfeed,
            name,
            description,
            link,
            ignore,
            excerpt_separator,
            syntax_highlight,
            sass,
        } = legacy;

        let post_order = match post_order.as_ref() {
            "asc" | "Asc" => config::SortOrder::Asc,
            _ => config::SortOrder::Desc,
        };

        let syntax_highlight = config::SyntaxHighlight { theme: syntax_highlight.theme };
        let sass = config::SassOptions {
            import_dir: sass.import_dir,
            style: match sass.style {
                SassOutputStyle::Nested => config::SassOutputStyle::Nested,
                SassOutputStyle::Expanded => config::SassOutputStyle::Expanded,
                SassOutputStyle::Compact => config::SassOutputStyle::Compact,
                SassOutputStyle::Compressed => config::SassOutputStyle::Compressed,
            },
        };

        config::Config {
            source: source,
            dest: dest,
            layouts: layouts,
            drafts: drafts,
            include_drafts: include_drafts,
            posts: posts,
            post_path: post_path,
            post_order: post_order,
            template_extensions: template_extensions,
            rss: rss,
            jsonfeed: jsonfeed,
            name: name,
            description: description,
            link: link,
            ignore: ignore,
            excerpt_separator: excerpt_separator,
            dump: vec![],
            syntax_highlight: syntax_highlight,
            sass: sass,
        }
    }
}
