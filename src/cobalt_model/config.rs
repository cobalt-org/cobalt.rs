use std::fmt;
use std::path;

use serde_yaml;

use crate::error::*;

use super::assets;
use super::collection;
use super::files;
use super::mark;
use super::site;
use super::template;
use super::vwiki;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub source: path::PathBuf,
    pub destination: path::PathBuf,
    pub pages: collection::CollectionBuilder,
    pub posts: collection::CollectionBuilder,
    pub site: site::Site,
    pub layouts_path: path::PathBuf,
    pub liquid: template::LiquidBuilder,
    pub markdown: mark::MarkdownBuilder,
    pub vimwiki: vwiki::VimwikiBuilder,
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
            ignore,
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
            failure::bail!("`template_extensions` should not be empty.");
        }

        let source = files::cleanup_path(&source);
        let destination = files::cleanup_path(&destination);

        let mut ignore = ignore;
        if let Ok(rel_dest) = path::Path::new(&destination).strip_prefix(&source) {
            let rel_dest = rel_dest.to_str().expect("started as a utf-8 string");
            if !rel_dest.is_empty() {
                ignore.push(format!("/{}", rel_dest.to_owned()));
            }
        }

        let source = root.join(source);
        let destination = abs_dest.unwrap_or_else(|| root.join(destination));

        let pages = collection::CollectionBuilder::from_page_config(
            pages,
            &source,
            &site,
            &posts,
            &default,
            &ignore,
            &template_extensions,
        );

        let posts = collection::CollectionBuilder::from_post_config(
            posts,
            &source,
            &site,
            include_drafts,
            &default,
            &ignore,
            &template_extensions,
        );

        let site = site::Site::from_config(site);

        let assets =
            assets::AssetsBuilder::from_config(assets, &source, &ignore, &template_extensions);

        let includes_path = source.join(includes_dir);
        let layouts_path = source.join(layouts_dir);

        let liquid = template::LiquidBuilder {
            includes_path,
            theme: syntax_highlight.theme.clone(),
        };
        let markdown = mark::MarkdownBuilder {
            theme: syntax_highlight.theme.clone(),
            syntax_highlight_enabled: syntax_highlight.enabled,
        };
        let vimwiki = vwiki::VimwikiBuilder {
            theme: syntax_highlight.theme,
            syntax_highlight_enabled: syntax_highlight.enabled,
        };

        let config = Config {
            source,
            destination,
            pages,
            posts,
            site,
            layouts_path,
            liquid,
            markdown,
            vimwiki,
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
