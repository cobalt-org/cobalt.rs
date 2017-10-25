use std::default::Default;
use std::path;
use std::fs::File;
use std::io::Read;
use error::*;
use serde_yaml;

use frontmatter;
use legacy::wildwest;
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

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SassOutputStyle {
    Nested,
    Expanded,
    Compact,
    Compressed,
}

const SASS_IMPORT_DIR: &'static str = "_sass";

#[derive(Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SassOptions {
    #[serde(skip)]
    pub import_dir: &'static str,
    pub style: SassOutputStyle,
}

impl Default for SassOptions {
    fn default() -> SassOptions {
        SassOptions {
            import_dir: SASS_IMPORT_DIR,
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

const DATA_DIR: &'static str = "_data";

#[derive(Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteBuilder {
    pub name: Option<String>,
    pub description: Option<String>,
    pub base_url: Option<String>,
    #[serde(skip)]
    pub data_dir: &'static str,
}

impl Default for SiteBuilder {
    fn default() -> SiteBuilder {
        SiteBuilder {
            name: None,
            description: None,
            base_url: None,
            data_dir: DATA_DIR,
        }
    }
}

impl SiteBuilder {
    pub fn build(self) -> Result<SiteBuilder> {
        let SiteBuilder {
            name,
            description,
            base_url,
            data_dir,
        } = self;
        let base_url = base_url.map(|mut l| {
                                        if l.ends_with('/') {
                                            l.pop();
                                        }
                                        l
                                    });
        Ok(SiteBuilder {
               name,
               description,
               base_url,
               data_dir,
           })
    }
}

#[derive(Debug, PartialEq, Default)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageBuilder {
    pub default: frontmatter::FrontmatterBuilder,
}

#[derive(Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostBuilder {
    pub name: Option<String>,
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
            name: None,
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

const LAYOUTS_DIR: &'static str = "_layouts";

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
    pub site: SiteBuilder,
    pub template_extensions: Vec<String>,
    pub ignore: Vec<String>,
    pub syntax_highlight: SyntaxHighlight,
    pub layouts_dir: &'static str,
    pub sass: SassOptions,
    // This is a debug-only field and should be transient rather than persistently set.
    #[serde(skip)]
    pub dump: Vec<Dump>,
}

impl Default for ConfigBuilder {
    fn default() -> ConfigBuilder {
        ConfigBuilder {
            root: path::PathBuf::new(),
            source: "./".to_owned(),
            destination: "./".to_owned(),
            abs_dest: None,
            include_drafts: false,
            default: frontmatter::FrontmatterBuilder::new()
                .set_excerpt_separator("\n\n".to_owned())
                .set_draft(false)
                .set_post(false),
            pages: PageBuilder::default(),
            posts: PostBuilder::default(),
            site: SiteBuilder::default(),
            template_extensions: vec!["md".to_owned(), "liquid".to_owned()],
            ignore: vec![],
            syntax_highlight: SyntaxHighlight::default(),
            layouts_dir: LAYOUTS_DIR,
            sass: SassOptions::default(),
            dump: vec![],
        }
    }
}

impl ConfigBuilder {
    pub fn from_file<P: Into<path::PathBuf>>(path: P) -> Result<ConfigBuilder> {
        Self::from_file_internal(path.into())
    }

    fn from_file_internal(path: path::PathBuf) -> Result<ConfigBuilder> {
        let content = {
            let mut buffer = String::new();
            let mut f = File::open(&path)?;
            f.read_to_string(&mut buffer)?;
            buffer
        };

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
        let file_path = find_project_file(&cwd, ".cobalt.yml");
        let mut config = file_path
            .map(|p| {
                     info!("Using config file {:?}", &p);
                     Self::from_file(&p).chain_err(|| format!("Error reading config file {:?}", p))
                 })
            .unwrap_or_else(|| {
                warn!("No .cobalt.yml file found in current directory, using default config.");
                Ok(ConfigBuilder::default())
            })?;
        config.root = cwd;
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
            sass,
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

        let config = Config {
            source: root.join(source),
            destination: abs_dest
                .map(|s| s.into())
                .unwrap_or_else(|| root.join(destination)),
            include_drafts,
            pages,
            posts,
            site: site.build()?,
            ignore,
            template_extensions,
            syntax_highlight,
            layouts_dir,
            sass,
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
    pub site: SiteBuilder,
    pub template_extensions: Vec<String>,
    pub ignore: Vec<String>,
    pub syntax_highlight: SyntaxHighlight,
    pub layouts_dir: &'static str,
    pub sass: SassOptions,
    pub dump: Vec<Dump>,
}

impl Default for Config {
    fn default() -> Config {
        ConfigBuilder::default()
            .build()
            .expect("default config should not fail")
    }
}
fn find_project_file<P: Into<path::PathBuf>>(dir: P, name: &str) -> Option<path::PathBuf> {
    find_project_file_internal(dir.into(), name)
}

fn find_project_file_internal(dir: path::PathBuf, name: &str) -> Option<path::PathBuf> {
    let mut file_path = dir;
    file_path.push(name);
    while !file_path.exists() {
        file_path.pop(); // filename
        let hit_bottom = !file_path.pop();
        if hit_bottom {
            return None;
        }
        file_path.push(name);
    }
    Some(file_path)
}

#[test]
fn find_project_file_same_dir() {
    let actual = find_project_file("tests/fixtures/config", ".cobalt.yml").unwrap();
    let expected = path::Path::new("tests/fixtures/config/.cobalt.yml");
    assert_eq!(actual, expected);
}

#[test]
fn find_project_file_parent_dir() {
    let actual = find_project_file("tests/fixtures/config/child", ".cobalt.yml").unwrap();
    let expected = path::Path::new("tests/fixtures/config/.cobalt.yml");
    assert_eq!(actual, expected);
}

#[test]
fn find_project_file_doesnt_exist() {
    let expected = path::Path::new("<NOT FOUND>");
    let actual = find_project_file("tests/fixtures/", ".cobalt.yml")
        .unwrap_or_else(|| expected.into());
    assert_eq!(actual, expected);
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
                   posts: PostBuilder {
                       drafts_dir: Some("_drafts".to_owned()),
                       rss: Some("rss.xml".to_owned()),
                       ..Default::default()
                   },
                   site: SiteBuilder {
                       name: Some("My blog!".to_owned()),
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
    let result = ConfigBuilder::from_cwd("tests/fixtures/config").unwrap();
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
fn test_build_dest() {
    let result = ConfigBuilder::from_file("tests/fixtures/config/.cobalt.yml").unwrap();
    let result = result.build().unwrap();
    assert_eq!(result,
               Config {
                   source: path::Path::new("tests/fixtures/config").to_path_buf(),
                   destination: path::Path::new("tests/fixtures/config/./dest").to_path_buf(),
                   posts: PostBuilder {
                       dir: "_my_posts".to_owned(),
                       drafts_dir: Some("_drafts".to_owned()),
                       default: frontmatter::FrontmatterBuilder::new()
                           .set_excerpt_separator("\n\n".to_owned())
                           .set_draft(false)
                           .set_post(true),
                       ..Default::default()
                   },
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
                       dir: "_my_posts".to_owned(),
                       drafts_dir: Some("_drafts".to_owned()),
                       default: frontmatter::FrontmatterBuilder::new()
                           .set_excerpt_separator("\n\n".to_owned())
                           .set_draft(false)
                           .set_post(true),
                       ..Default::default()
                   },
                   ..Default::default()
               });
}
