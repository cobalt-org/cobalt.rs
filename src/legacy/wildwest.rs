use std::default::Default;
use std::fmt;
use std::path;

use liquid;
use serde_yaml;

use error::*;
use cobalt_model;
use cobalt_model::files;

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

pub type DocumentBuilder = cobalt_model::DocumentBuilder<FrontmatterBuilder>;

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
    #[serde(skip)]
    pub root: path::PathBuf,
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

impl GlobalConfig {
    pub fn from_file<P: Into<path::PathBuf>>(path: P) -> Result<GlobalConfig> {
        Self::from_file_internal(path.into())
    }

    fn from_file_internal(path: path::PathBuf) -> Result<GlobalConfig> {
        let content = files::read_file(&path)?;

        let mut config = if content.trim().is_empty() {
            GlobalConfig::default()
        } else {
            let config: GlobalConfig = serde_yaml::from_str(&content)?;
            config
        };

        let mut root = path;
        root.pop(); // Remove filename
        config.root = root;

        Ok(config)
    }

    pub fn from_cwd<P: Into<path::PathBuf>>(cwd: P) -> Result<GlobalConfig> {
        Self::from_cwd_internal(cwd.into())
    }

    fn from_cwd_internal(cwd: path::PathBuf) -> Result<GlobalConfig> {
        let file_path = files::find_project_file(&cwd, ".cobalt.yml");
        let config = file_path
            .map(|p| {
                     info!("Using config file {:?}", &p);
                     Self::from_file(&p).chain_err(|| format!("Error reading config file {:?}", p))
                 })
            .unwrap_or_else(|| {
                warn!("No .cobalt.yml file found in current directory, using default config.");
                let config = GlobalConfig {
                    root: cwd,
                    ..Default::default()
                };
                Ok(config)
            })?;
        Ok(config)
    }
}

impl Default for GlobalConfig {
    fn default() -> GlobalConfig {
        GlobalConfig {
            root: Default::default(),
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

impl From<GlobalConfig> for cobalt_model::ConfigBuilder {
    fn from(legacy: GlobalConfig) -> Self {
        let GlobalConfig {
            root,
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
            "asc" | "Asc" => cobalt_model::SortOrder::Asc,
            _ => cobalt_model::SortOrder::Desc,
        };

        let default = cobalt_model::FrontmatterBuilder::new()
            .set_excerpt_separator(excerpt_separator)
            .set_draft(false)
            .set_post(false);
        let posts = cobalt_model::PostBuilder {
            title: None,
            slug: None,
            description: None,
            dir: posts,
            drafts_dir: Some(drafts),
            order: post_order,
            rss: rss,
            jsonfeed: jsonfeed,
            default: cobalt_model::FrontmatterBuilder::new()
                .set_permalink(post_path.map(convert_permalink))
                .set_post(true),
        };

        let site = cobalt_model::SiteBuilder {
            title: name,
            description: description,
            base_url: link,
            ..Default::default()
        };

        let syntax_highlight = cobalt_model::SyntaxHighlight { theme: syntax_highlight.theme };
        let sass = cobalt_model::SassBuilder {
            style: match sass.style {
                SassOutputStyle::Nested => cobalt_model::SassOutputStyle::Nested,
                SassOutputStyle::Expanded => cobalt_model::SassOutputStyle::Expanded,
                SassOutputStyle::Compact => cobalt_model::SassOutputStyle::Compact,
                SassOutputStyle::Compressed => cobalt_model::SassOutputStyle::Compressed,
            },
            ..Default::default()
        };

        cobalt_model::ConfigBuilder {
            root: root,
            source: source,
            destination: dest,
            include_drafts: include_drafts,
            default,
            pages: cobalt_model::PageBuilder::default(),
            posts,
            site: site,
            template_extensions: template_extensions,
            ignore: ignore,
            syntax_highlight: syntax_highlight,
            assets: cobalt_model::AssetsBuilder { sass },
            dump: vec![],
            ..Default::default()
        }
    }
}

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
