use std::default::Default;
use std::path;

use serde_yaml;

use error::*;
use cobalt_model;
use cobalt_model::files;

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
    pub syntax_highlight: cobalt_model::SyntaxHighlight,
    pub sass: cobalt_model::SassBuilder,
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
            syntax_highlight: cobalt_model::SyntaxHighlight::default(),
            sass: cobalt_model::SassBuilder::default(),
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
            .set_draft(false);
        let posts = cobalt_model::PostBuilder {
            title: None,
            description: None,
            dir: Some(posts),
            drafts_dir: Some(drafts),
            order: post_order,
            rss: rss,
            jsonfeed: jsonfeed,
            default: cobalt_model::FrontmatterBuilder::new()
                .set_permalink(post_path.map(convert_permalink)),
        };

        let site = cobalt_model::SiteBuilder {
            title: name,
            description: description,
            base_url: link,
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
            assets: cobalt_model::AssetsBuilder {
                sass,
                ..Default::default()
            },
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
