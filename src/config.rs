use std::default::Default;
use std::path;

use serde_yaml;

use error::*;
use files;
use frontmatter;
use legacy::wildwest;
use sass;
use site;
use slug;
use syntax_highlight::has_syntax_theme;

arg_enum! {
    #[derive(Serialize, Deserialize)]
    #[derive(Debug, PartialEq, Copy, Clone)]
    pub enum Dump {
        DocObject,
        DocTemplate,
        DocLinkObject,
        Document
    }
}

impl Dump {
    pub fn is_doc(&self) -> bool {
        true
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl Default for SortOrder {
    fn default() -> SortOrder {
        SortOrder::Desc
    }
}

#[derive(Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct SyntaxHighlight {
    pub theme: String,
}

impl Default for SyntaxHighlight {
    fn default() -> SyntaxHighlight {
        SyntaxHighlight { theme: "base16-ocean.dark".to_owned() }
    }
}

#[derive(Debug, PartialEq, Default)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct PageBuilder {
    pub default: frontmatter::FrontmatterBuilder,
}

#[derive(Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct PostBuilder {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub dir: String,
    pub drafts_dir: Option<String>,
    pub order: SortOrder,
    pub rss: Option<String>,
    pub jsonfeed: Option<String>,
    pub default: frontmatter::FrontmatterBuilder,
}

impl Default for PostBuilder {
    fn default() -> PostBuilder {
        Self {
            title: None,
            slug: None,
            description: None,
            dir: "posts".to_owned(),
            drafts_dir: None,
            order: SortOrder::default(),
            rss: None,
            jsonfeed: None,
            default: frontmatter::FrontmatterBuilder::new().set_post(true),
        }
    }
}

#[derive(Debug, PartialEq, Default)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct AssetsBuilder {
    pub sass: sass::SassBuilder,
}

impl AssetsBuilder {
    pub fn build(self) -> Assets {
        Assets { sass: self.sass.build() }
    }
}

#[derive(Debug, PartialEq, Default)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Assets {
    pub sass: sass::SassCompiler,
}

const LAYOUTS_DIR: &'static str = "_layouts";
const INCLUDES_DIR: &'static str = "_includes";

#[derive(Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ConfigBuilder {
    #[serde(skip)]
    pub root: path::PathBuf,
    pub source: String,
    pub destination: String,
    #[serde(skip)]
    pub abs_dest: Option<String>,
    #[serde(skip)]
    pub include_drafts: bool,
    pub default: frontmatter::FrontmatterBuilder,
    pub pages: PageBuilder,
    pub posts: PostBuilder,
    pub site: site::SiteBuilder,
    pub template_extensions: Vec<String>,
    pub ignore: Vec<String>,
    pub syntax_highlight: SyntaxHighlight,
    #[serde(skip)]
    pub layouts_dir: &'static str,
    #[serde(skip)]
    pub includes_dir: &'static str,
    pub assets: AssetsBuilder,
    // This is a debug-only field and should be transient rather than persistently set.
    #[serde(skip)]
    pub dump: Vec<Dump>,
}

impl Default for ConfigBuilder {
    fn default() -> ConfigBuilder {
        ConfigBuilder {
            root: path::PathBuf::new(),
            source: "./".to_owned(),
            destination: "./_site".to_owned(),
            abs_dest: None,
            include_drafts: false,
            default: frontmatter::FrontmatterBuilder::new()
                .set_excerpt_separator("\n\n".to_owned())
                .set_draft(false)
                .set_post(false),
            pages: PageBuilder::default(),
            posts: PostBuilder::default(),
            site: site::SiteBuilder::default(),
            template_extensions: vec!["md".to_owned(), "liquid".to_owned()],
            ignore: vec![],
            syntax_highlight: SyntaxHighlight::default(),
            layouts_dir: LAYOUTS_DIR,
            includes_dir: INCLUDES_DIR,
            assets: AssetsBuilder::default(),
            dump: vec![],
        }
    }
}

impl ConfigBuilder {
    pub fn from_file<P: Into<path::PathBuf>>(path: P) -> Result<ConfigBuilder> {
        Self::from_file_internal(path.into())
    }

    fn from_file_internal(path: path::PathBuf) -> Result<ConfigBuilder> {
        let content = files::read_file(&path)?;

        if content.trim().is_empty() {
            return Ok(ConfigBuilder::default());
        }

        let config: wildwest::GlobalConfig = serde_yaml::from_str(&content)?;
        let mut config: ConfigBuilder = config.into();

        let mut root = path;
        root.pop(); // Remove filename
        config.root = root;

        Ok(config)
    }

    pub fn from_cwd<P: Into<path::PathBuf>>(cwd: P) -> Result<ConfigBuilder> {
        Self::from_cwd_internal(cwd.into())
    }

    fn from_cwd_internal(cwd: path::PathBuf) -> Result<ConfigBuilder> {
        let file_path = files::find_project_file(&cwd, ".cobalt.yml");
        let config = file_path
            .map(|p| {
                     info!("Using config file {:?}", &p);
                     Self::from_file(&p).chain_err(|| format!("Error reading config file {:?}", p))
                 })
            .unwrap_or_else(|| {
                warn!("No .cobalt.yml file found in current directory, using default config.");
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
            layouts_dir: _layouts_dir,
            includes_dir: _includes_dir,
            assets,
            dump,
        } = self;

        let result: Result<()> = match has_syntax_theme(&syntax_highlight.theme) {
            Ok(true) => Ok(()),
            Ok(false) => {
                Err(format!("Syntax theme '{}' is unsupported", syntax_highlight.theme).into())
            }
            Err(err) => {
                warn!("Syntax theme named '{}' ignored. Reason: {}",
                      syntax_highlight.theme,
                      err);
                Ok(())
            }
        };
        result?;

        let mut pages = pages;
        pages.default = pages.default.merge(default.clone());

        let mut posts = posts;
        posts.default = posts.default.merge(default);
        if posts.dir.starts_with('/') {
            bail!("posts dir {} must be a relative path", posts.dir)
        }
        if let Some(ref drafts_dir) = posts.drafts_dir {
            if drafts_dir.starts_with('/') {
                bail!("posts dir {} must be a relative path", drafts_dir)
            }
        }
        if posts.slug.is_none() {
            if let Some(ref title) = posts.title {
                posts.slug = Some(slug::slugify(title));
            } else {
                posts.slug = Some(posts.dir.clone());
            }
        }

        let source = files::cleanup_path(source);
        let destination = files::cleanup_path(destination);

        let mut ignore = ignore;
        if let Ok(rel_dest) = path::Path::new(&destination).strip_prefix(&source) {
            let rel_dest = rel_dest.to_str().expect("started as a utf-8 string");
            if !rel_dest.is_empty() {
                ignore.push(format!("/{}", rel_dest.to_owned()));
            }
        }

        let source = root.join(source);
        let destination = abs_dest
            .map(|s| s.into())
            .unwrap_or_else(|| root.join(destination));

        // HACK for serde #1105
        let layouts_dir = LAYOUTS_DIR;
        let includes_dir = INCLUDES_DIR;

        let site = site.build(&source)?;

        let assets = assets.build();

        let config = Config {
            source,
            destination,
            include_drafts,
            pages,
            posts,
            site,
            ignore,
            template_extensions,
            syntax_highlight,
            layouts_dir,
            includes_dir,
            assets,
            dump,
        };

        Ok(config)
    }
}

#[derive(Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub source: path::PathBuf,
    pub destination: path::PathBuf,
    pub include_drafts: bool,
    pub pages: PageBuilder,
    pub posts: PostBuilder,
    pub site: site::Site,
    pub template_extensions: Vec<String>,
    pub ignore: Vec<String>,
    pub syntax_highlight: SyntaxHighlight,
    pub layouts_dir: &'static str,
    pub includes_dir: &'static str,
    pub assets: Assets,
    pub dump: Vec<Dump>,
}

impl Default for Config {
    fn default() -> Config {
        ConfigBuilder::default()
            .build()
            .expect("default config should not fail")
    }
}

#[test]
fn test_from_file_ok() {
    let result = ConfigBuilder::from_file("tests/fixtures/config/.cobalt.yml").unwrap();
    assert_eq!(result,
               ConfigBuilder {
                   root: path::Path::new("tests/fixtures/config").to_path_buf(),
                   destination: "./dest".to_owned(),
                   posts: PostBuilder {
                       dir: "_my_posts".to_owned(),
                       drafts_dir: Some("_drafts".to_owned()),
                       ..Default::default()
                   },
                   ..Default::default()
               });
}

#[test]
fn test_from_file_rss() {
    let result = ConfigBuilder::from_file("tests/fixtures/config/rss.yml").unwrap();
    assert_eq!(result,
               ConfigBuilder {
                   root: path::Path::new("tests/fixtures/config").to_path_buf(),
                   destination: "./".to_owned(),
                   posts: PostBuilder {
                       drafts_dir: Some("_drafts".to_owned()),
                       rss: Some("rss.xml".to_owned()),
                       ..Default::default()
                   },
                   site: site::SiteBuilder {
                       title: Some("My blog!".to_owned()),
                       description: Some("Blog description".to_owned()),
                       base_url: Some("http://example.com".to_owned()),
                       ..Default::default()
                   },
                   ..Default::default()
               });
}

#[test]
fn test_from_file_empty() {
    let result = ConfigBuilder::from_file("tests/fixtures/config/empty.yml").unwrap();
    assert_eq!((result), ConfigBuilder { ..Default::default() });
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
    assert_eq!(result,
               ConfigBuilder {
                   root: path::Path::new("tests/fixtures/config").to_path_buf(),
                   destination: "./dest".to_owned(),
                   posts: PostBuilder {
                       dir: "_my_posts".to_owned(),
                       drafts_dir: Some("_drafts".to_owned()),
                       ..Default::default()
                   },
                   ..Default::default()
               });
}

#[test]
fn test_from_cwd_not_found() {
    let result = ConfigBuilder::from_cwd("tests/fixtures").unwrap();
    assert_eq!(result,
               ConfigBuilder {
                   root: path::Path::new("tests/fixtures").to_path_buf(),
                   ..Default::default()
               });
}

#[test]
fn test_build_default() {
    let config = ConfigBuilder::default();
    config.build().unwrap();
}
#[test]
fn test_build_dest() {
    let result = ConfigBuilder::from_file("tests/fixtures/config/.cobalt.yml").unwrap();
    let result = result.build().unwrap();
    assert_eq!(result,
               Config {
                   source: path::Path::new("tests/fixtures/config").to_path_buf(),
                   destination: path::Path::new("tests/fixtures/config/dest").to_path_buf(),
                   posts: PostBuilder {
                       slug: Some("_my_posts".to_owned()),
                       dir: "_my_posts".to_owned(),
                       drafts_dir: Some("_drafts".to_owned()),
                       default: frontmatter::FrontmatterBuilder::new()
                           .set_excerpt_separator("\n\n".to_owned())
                           .set_draft(false)
                           .set_post(true),
                       ..Default::default()
                   },
                   ignore: ["/dest".to_owned()].to_vec(),
                   ..Default::default()
               });
}

#[test]
fn test_build_abs_dest() {
    let mut result = ConfigBuilder::from_file("tests/fixtures/config/.cobalt.yml").unwrap();
    result.abs_dest = Some("hello/world".to_owned());
    let result = result.build().unwrap();
    assert_eq!(result,
               Config {
                   source: path::Path::new("tests/fixtures/config").to_path_buf(),
                   destination: path::Path::new("hello/world").to_path_buf(),
                   posts: PostBuilder {
                       slug: Some("_my_posts".to_owned()),
                       dir: "_my_posts".to_owned(),
                       drafts_dir: Some("_drafts".to_owned()),
                       default: frontmatter::FrontmatterBuilder::new()
                           .set_excerpt_separator("\n\n".to_owned())
                           .set_draft(false)
                           .set_post(true),
                       ..Default::default()
                   },
                   ignore: ["/dest".to_owned()].to_vec(),
                   ..Default::default()
               });
}

#[test]
fn test_build_posts_rel() {
    let mut config = ConfigBuilder::default();
    config.posts.dir = "rel".into();
    let config = config.build().unwrap();
    assert_eq!(config.posts.dir, "rel");
}

#[test]
fn test_build_posts_abs() {
    let mut config = ConfigBuilder::default();
    config.posts.dir = "/root".into();
    assert!(config.build().is_err());
}

#[test]
fn test_build_drafts_rel() {
    let mut config = ConfigBuilder::default();
    config.posts.drafts_dir = Some("rel".into());
    let config = config.build().unwrap();
    assert_eq!(config.posts.drafts_dir, Some("rel".into()));
}

#[test]
fn test_build_drafts_abs() {
    let mut config = ConfigBuilder::default();
    config.posts.drafts_dir = Some("/root".into());
    assert!(config.build().is_err());
}
