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
            .merge_categories(custom_attributes.remove("categories").and_then(|v| {
                v.as_array()
                    .map(|v| v.iter().map(|v| v.to_string()).collect())
            }))
            .merge_slug(custom_attributes
                            .remove("slug")
                            .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_permalink(custom_attributes
                                 .remove("path")
                                 .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_draft(custom_attributes.remove("draft").and_then(|v| v.as_bool()))
            .merge_excerpt_separator(custom_attributes
                                         .remove("excerpt_separator")
                                         .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_layout(custom_attributes
                              .remove("extends")
                              .and_then(|v| v.as_str().map(|s| s.to_owned())))
            .merge_published_date(custom_attributes
                                      .remove("date")
                                      .and_then(|d| d.as_str().and_then(datetime::DateTime::parse)))
            .merge_custom(custom_attributes)
    }
}

impl From<frontmatter::FrontmatterBuilder> for FrontmatterBuilder {
    fn from(internal: frontmatter::FrontmatterBuilder) -> Self {
        let mut legacy = liquid::Object::new();

        let frontmatter::FrontmatterBuilder {
            path,
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
        if let Some(path) = path {
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
    pub style: SassOutputStyle,
}

impl Default for SassOptions {
    fn default() -> SassOptions {
        SassOptions { style: SassOutputStyle::Nested }
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

impl From<GlobalConfig> for config::ConfigBuilder {
    fn from(legacy: GlobalConfig) -> Self {
        let GlobalConfig {
            source,
            dest,
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
            style: match sass.style {
                SassOutputStyle::Nested => config::SassOutputStyle::Nested,
                SassOutputStyle::Expanded => config::SassOutputStyle::Expanded,
                SassOutputStyle::Compact => config::SassOutputStyle::Compact,
                SassOutputStyle::Compressed => config::SassOutputStyle::Compressed,
            },
            ..Default::default()
        };

        let site = config::SiteBuilder {
            name: name,
            description: description,
            base_url: link,
            ..Default::default()
        };

        config::ConfigBuilder {
            source: source,
            destination: dest,
            drafts: drafts,
            include_drafts: include_drafts,
            posts: posts,
            post_path: post_path,
            post_order: post_order,
            template_extensions: template_extensions,
            rss: rss,
            jsonfeed: jsonfeed,
            site: site,
            ignore: ignore,
            excerpt_separator: excerpt_separator,
            dump: vec![],
            syntax_highlight: syntax_highlight,
            sass: sass,
            ..Default::default()
        }
    }
}
