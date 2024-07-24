use std::fmt;
use std::path;

use log::debug;
use serde::Serialize;
use serde_yaml;

use crate::error::Result;

use super::assets;
use super::collection;
use super::mark;
use super::site;
use super::template;
use crate::SyntaxHighlight;

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub source: path::PathBuf,
    pub destination: path::PathBuf,
    pub ignore: Vec<liquid::model::KString>,
    pub page_extensions: Vec<liquid::model::KString>,
    pub include_drafts: bool,
    pub pages: collection::Collection,
    pub posts: collection::Collection,
    pub site: site::Site,
    pub layouts_path: path::PathBuf,
    pub liquid: template::LiquidBuilder,
    pub markdown: mark::MarkdownBuilder,
    #[serde(skip)]
    pub syntax: std::sync::Arc<SyntaxHighlight>,
    pub assets: assets::AssetsBuilder,
    pub minify: cobalt_config::Minify,
}

impl Config {
    pub fn from_config(source: cobalt_config::Config) -> Result<Self> {
        let cobalt_config::Config {
            root,
            source,
            destination,
            abs_dest,
            include_drafts,
            default,
            pages,
            posts,
            site,
            template_extensions,
            ignore: custom_ignore,
            syntax_highlight,
            layouts_dir,
            includes_dir,
            assets,
            minify,
        } = source;

        if include_drafts {
            debug!("Draft mode enabled");
        }

        if template_extensions.is_empty() {
            anyhow::bail!("`template_extensions` should not be empty.");
        }

        let source = source.to_path(&root);
        let destination = abs_dest.unwrap_or_else(|| destination.to_path(root));

        let pages = collection::Collection::from_page_config(pages, &site, &default)?;

        let posts =
            collection::Collection::from_post_config(posts, &site, include_drafts, &default)?;

        let site = site::Site::from_config(site);

        let mut ignore: Vec<liquid::model::KString> = vec![".*".into(), "_*".into()];
        if let Ok(rel_dest) = path::Path::new(&destination).strip_prefix(&source) {
            let rel_dest = rel_dest.to_str().expect("started as a utf-8 string");
            if !rel_dest.is_empty() {
                ignore.push(format!("/{}", rel_dest.to_owned()).into());
            }
        }
        ignore.push(format!("/{}", includes_dir).into());
        ignore.push(format!("/{}", layouts_dir).into());
        ignore.push("/_defaults".into());
        ignore.push(format!("/{}", assets.sass.import_dir).into());
        assert_eq!(pages.dir, "");
        assert_eq!(pages.drafts_dir, None);
        ignore.push(format!("!/{}", posts.dir).into());
        if let Some(dir) = posts.drafts_dir.as_deref() {
            ignore.push(format!("!/{}", dir).into());
        }
        ignore.extend(custom_ignore);

        let assets = assets::AssetsBuilder::from_config(assets, &source);

        let includes_path = source.join(includes_dir);
        let layouts_path = source.join(layouts_dir);

        let mut highlight = SyntaxHighlight::new();
        let syntaxes_path = source.join("_syntaxes");
        if syntaxes_path.exists() {
            highlight.load_custom_syntaxes(&syntaxes_path);
        }

        let syntax = std::sync::Arc::new(highlight);

        let liquid = template::LiquidBuilder {
            includes_path,
            syntax: syntax.clone(),
            theme: syntax_highlight
                .enabled
                .then(|| syntax_highlight.theme.clone()),
        };
        let markdown = mark::MarkdownBuilder {
            syntax: syntax.clone(),
            theme: syntax_highlight
                .enabled
                .then(|| syntax_highlight.theme.clone()),
        };

        let config = Config {
            source,
            destination,
            ignore,
            page_extensions: template_extensions,
            include_drafts,
            pages,
            posts,
            site,
            layouts_path,
            liquid,
            markdown,
            syntax,
            assets,
            minify,
        };

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Config {
        Config::from_config(cobalt_config::Config::default())
            .expect("default config should not fail")
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut converted = serde_yaml::to_string(self).map_err(|_| fmt::Error)?;
        converted.drain(..4);
        write!(f, "{}", converted)
    }
}

#[test]
fn test_build_default() {
    let config = cobalt_config::Config::default();
    Config::from_config(config).unwrap();
}

#[test]
fn test_build_dest() {
    let config = cobalt_config::Config::from_file("tests/fixtures/config/_cobalt.yml").unwrap();
    let result = Config::from_config(config).unwrap();
    assert_eq!(
        result.source,
        path::Path::new("tests/fixtures/config").to_path_buf()
    );
    assert_eq!(
        result.destination,
        path::Path::new("tests/fixtures/config/dest").to_path_buf()
    );
}

#[test]
fn test_build_abs_dest() {
    let mut config = cobalt_config::Config::from_file("tests/fixtures/config/_cobalt.yml").unwrap();
    config.abs_dest = Some(path::PathBuf::from("hello/world"));
    let result = Config::from_config(config).unwrap();
    assert_eq!(
        result.source,
        path::Path::new("tests/fixtures/config").to_path_buf()
    );
    assert_eq!(
        result.destination,
        path::Path::new("hello/world").to_path_buf()
    );
}
