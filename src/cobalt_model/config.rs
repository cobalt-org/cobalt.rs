use std::fmt;
use std::path;

use liquid;
use serde_yaml;

use error::*;

use super::assets;
use super::collection;
use super::files;
use super::frontmatter;
use super::mark;
use super::sass;
use super::site;
use super::template;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct SyntaxHighlight {
    pub theme: String,
    pub enabled: bool,
}

impl Default for SyntaxHighlight {
    fn default() -> Self {
        Self {
            theme: "base16-ocean.dark".to_owned(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct PageConfig {
    pub default: frontmatter::FrontmatterBuilder,
}

impl PageConfig {
    fn builder(
        self,
        source: &path::Path,
        site: &SiteConfig,
        posts: &PostConfig,
        common_default: &frontmatter::FrontmatterBuilder,
        ignore: &[String],
        template_extensions: &[String],
    ) -> collection::CollectionBuilder {
        let mut ignore = ignore.to_vec();
        ignore.push(format!("/{}", posts.dir));
        if let Some(ref drafts_dir) = posts.drafts_dir {
            ignore.push(format!("/{}", drafts_dir));
        }
        // Use `site` because the pages are effectively the site
        collection::CollectionBuilder {
            title: Some(site.title.clone().unwrap_or_else(|| "".to_owned())),
            slug: Some("pages".to_owned()),
            description: site.description.clone(),
            source: Some(source.to_owned()),
            dir: Some(".".to_owned()),
            drafts_dir: None,
            include_drafts: false,
            template_extensions: template_extensions.to_vec(),
            ignore: ignore,
            order: collection::SortOrder::None,
            rss: None,
            jsonfeed: None,
            base_url: None,
            default: self.default
                .merge_excerpt_separator("".to_owned())
                .merge(common_default.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct PostConfig {
    pub title: Option<String>,
    pub description: Option<String>,
    pub dir: String,
    pub drafts_dir: Option<String>,
    pub order: collection::SortOrder,
    pub rss: Option<String>,
    pub jsonfeed: Option<String>,
    pub default: frontmatter::FrontmatterBuilder,
}

impl PostConfig {
    fn builder(
        self,
        source: &path::Path,
        site: &SiteConfig,
        include_drafts: bool,
        common_default: &frontmatter::FrontmatterBuilder,
        ignore: &[String],
        template_extensions: &[String],
    ) -> collection::CollectionBuilder {
        let PostConfig {
            title,
            description,
            dir,
            drafts_dir,
            order,
            rss,
            jsonfeed,
            default,
        } = self;
        // Default with `site` for people quickly bootstrapping a blog, the blog and site are
        // effectively equivalent.
        collection::CollectionBuilder {
            title: Some(
                title
                    .or_else(|| site.title.clone())
                    .unwrap_or_else(|| "".to_owned()),
            ),
            slug: Some("posts".to_owned()),
            description: description.or_else(|| site.description.clone()),
            source: Some(source.to_owned()),
            dir: Some(dir),
            drafts_dir,
            include_drafts: include_drafts,
            template_extensions: template_extensions.to_vec(),
            ignore: ignore.to_vec(),
            order,
            rss,
            jsonfeed,
            base_url: site.base_url.clone(),
            default: default.merge(common_default.clone()),
        }
    }
}

impl Default for PostConfig {
    fn default() -> Self {
        Self {
            title: Default::default(),
            description: Default::default(),
            dir: "posts".to_owned(),
            drafts_dir: Default::default(),
            order: Default::default(),
            rss: Default::default(),
            jsonfeed: Default::default(),
            default: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct SiteConfig {
    pub title: Option<String>,
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub data: Option<liquid::Object>,
    #[serde(skip)] pub data_dir: &'static str,
}

impl SiteConfig {
    fn builder(self, source: &path::Path) -> site::SiteBuilder {
        site::SiteBuilder {
            title: self.title,
            description: self.description,
            base_url: self.base_url,
            data: self.data,
            data_dir: Some(source.join(self.data_dir)),
        }
    }
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            title: Default::default(),
            description: Default::default(),
            base_url: Default::default(),
            data: Default::default(),
            data_dir: "_data",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct SassConfig {
    #[serde(skip)] pub import_dir: &'static str,
    pub style: sass::SassOutputStyle,
}

impl SassConfig {
    fn builder(self, source: &path::Path) -> sass::SassBuilder {
        let mut sass = sass::SassBuilder::new();
        sass.style = self.style;
        sass.import_dir = source
            .join(self.import_dir)
            .into_os_string()
            .into_string()
            .ok();
        sass
    }
}

impl Default for SassConfig {
    fn default() -> Self {
        Self {
            import_dir: "_sass",
            style: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct AssetsConfig {
    pub sass: SassConfig,
}

impl AssetsConfig {
    fn builder(
        self,
        source: &path::Path,
        ignore: &[String],
        template_extensions: &[String],
    ) -> assets::AssetsBuilder {
        assets::AssetsBuilder {
            sass: self.sass.builder(source),
            source: Some(source.to_owned()),
            ignore: ignore.to_vec(),
            template_extensions: template_extensions.to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ConfigBuilder {
    #[serde(skip)] pub root: path::PathBuf,
    pub source: String,
    pub destination: String,
    #[serde(skip)] pub abs_dest: Option<path::PathBuf>,
    pub include_drafts: bool,
    pub default: frontmatter::FrontmatterBuilder,
    pub pages: PageConfig,
    pub posts: PostConfig,
    pub site: SiteConfig,
    pub template_extensions: Vec<String>,
    pub ignore: Vec<String>,
    pub syntax_highlight: SyntaxHighlight,
    #[serde(skip)] pub layouts_dir: &'static str,
    #[serde(skip)] pub includes_dir: &'static str,
    pub assets: AssetsConfig,
}

impl Default for ConfigBuilder {
    fn default() -> ConfigBuilder {
        ConfigBuilder {
            root: Default::default(),
            source: "./".to_owned(),
            destination: "./_site".to_owned(),
            abs_dest: Default::default(),
            include_drafts: false,
            default: Default::default(),
            pages: Default::default(),
            posts: Default::default(),
            site: Default::default(),
            template_extensions: vec!["md".to_owned(), "liquid".to_owned()],
            ignore: Default::default(),
            syntax_highlight: SyntaxHighlight::default(),
            layouts_dir: "_layouts",
            includes_dir: "_includes",
            assets: AssetsConfig::default(),
        }
    }
}

impl ConfigBuilder {
    pub fn from_file<P: Into<path::PathBuf>>(path: P) -> Result<ConfigBuilder> {
        Self::from_file_internal(path.into())
    }

    fn from_file_internal(path: path::PathBuf) -> Result<ConfigBuilder> {
        let content = files::read_file(&path)?;

        let mut config = if content.trim().is_empty() {
            ConfigBuilder::default()
        } else {
            serde_yaml::from_str(&content)?
        };

        let mut root = path;
        root.pop(); // Remove filename
        config.root = root;

        Ok(config)
    }

    pub fn from_cwd<P: Into<path::PathBuf>>(cwd: P) -> Result<ConfigBuilder> {
        Self::from_cwd_internal(cwd.into())
    }

    fn from_cwd_internal(cwd: path::PathBuf) -> Result<ConfigBuilder> {
        let file_path = files::find_project_file(&cwd, "_cobalt.yml");
        let config = file_path
            .map(|p| {
                debug!("Using config file {:?}", &p);
                Self::from_file(&p).chain_err(|| format!("Error reading config file {:?}", p))
            })
            .unwrap_or_else(|| {
                warn!("No _cobalt.yml file found in current directory, using default config.");
                let config = ConfigBuilder {
                    root: cwd,
                    ..Default::default()
                };
                Ok(config)
            })?;
        Ok(config)
    }

    pub fn build(self) -> Result<Config> {
        let ConfigBuilder {
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
        } = self;

        if include_drafts {
            debug!("Draft mode enabled");
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

        let pages = pages.builder(
            &source,
            &site,
            &posts,
            &default,
            &ignore,
            &template_extensions,
        );

        let posts = posts.builder(
            &source,
            &site,
            include_drafts,
            &default,
            &ignore,
            &template_extensions,
        );

        let site = site.builder(&source);

        let assets = assets.builder(&source, &ignore, &template_extensions);

        let includes_dir = source.join(includes_dir);
        let layouts_dir = source.join(layouts_dir);

        let liquid = template::LiquidBuilder {
            includes_dir: includes_dir.clone(),
            legacy_path: source.clone(),
            theme: syntax_highlight.theme.clone(),
        };
        let markdown = mark::MarkdownBuilder {
            theme: syntax_highlight.theme,
            syntax_highlight_enabled: syntax_highlight.enabled,
        };

        let config = Config {
            source,
            destination,
            pages,
            posts,
            site,
            layouts_dir,
            liquid,
            markdown,
            assets,
        };

        Ok(config)
    }
}

impl fmt::Display for ConfigBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut converted = serde_yaml::to_string(self).map_err(|_| fmt::Error)?;
        converted.drain(..4);
        write!(f, "{}", converted)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub source: path::PathBuf,
    pub destination: path::PathBuf,
    pub pages: collection::CollectionBuilder,
    pub posts: collection::CollectionBuilder,
    pub site: site::SiteBuilder,
    pub layouts_dir: path::PathBuf,
    pub liquid: template::LiquidBuilder,
    pub markdown: mark::MarkdownBuilder,
    pub assets: assets::AssetsBuilder,
}

impl Default for Config {
    fn default() -> Config {
        ConfigBuilder::default()
            .build()
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
fn test_from_file_ok() {
    let result = ConfigBuilder::from_file("tests/fixtures/config/_cobalt.yml").unwrap();
    assert_eq!(
        result.root,
        path::Path::new("tests/fixtures/config").to_path_buf()
    );
}

#[test]
fn test_from_file_alternate_name() {
    let result = ConfigBuilder::from_file("tests/fixtures/config/rss.yml").unwrap();
    assert_eq!(
        result.root,
        path::Path::new("tests/fixtures/config").to_path_buf()
    );
}

#[test]
fn test_from_file_empty() {
    let result = ConfigBuilder::from_file("tests/fixtures/config/empty.yml").unwrap();
    assert_eq!(
        result.root,
        path::Path::new("tests/fixtures/config").to_path_buf()
    );
}

#[test]
fn test_from_file_invalid_syntax() {
    let result = ConfigBuilder::from_file("tests/fixtures/config/invalid_syntax.yml");
    assert!(result.is_err());
}

#[test]
fn test_from_file_not_found() {
    let result = ConfigBuilder::from_file("tests/fixtures/config/config_does_not_exist.yml");
    assert!(result.is_err());
}

#[test]
fn test_from_cwd_ok() {
    let result = ConfigBuilder::from_cwd("tests/fixtures/config/child").unwrap();
    assert_eq!(
        result.root,
        path::Path::new("tests/fixtures/config").to_path_buf()
    );
}

#[test]
fn test_from_cwd_not_found() {
    let result = ConfigBuilder::from_cwd("tests/fixtures").unwrap();
    assert_eq!(result.root, path::Path::new("tests/fixtures").to_path_buf());
}

#[test]
fn test_build_default() {
    let config = ConfigBuilder::default();
    config.build().unwrap();
}

#[test]
fn test_build_dest() {
    let result = ConfigBuilder::from_file("tests/fixtures/config/_cobalt.yml").unwrap();
    let result = result.build().unwrap();
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
    let mut result = ConfigBuilder::from_file("tests/fixtures/config/_cobalt.yml").unwrap();
    result.abs_dest = Some(path::PathBuf::from("hello/world"));
    let result = result.build().unwrap();
    assert_eq!(
        result.source,
        path::Path::new("tests/fixtures/config").to_path_buf()
    );
    assert_eq!(
        result.destination,
        path::Path::new("hello/world").to_path_buf()
    );
}
