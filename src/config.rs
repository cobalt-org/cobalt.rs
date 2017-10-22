use std::default::Default;
use std::path;
use std::fs::File;
use std::io::Read;
use error::*;
use serde_yaml;

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
const LAYOUTS_DIR: &'static str = "_layouts";

#[derive(Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ConfigBuilder {
    #[serde(skip)]
    pub root: path::PathBuf,
    pub source: String,
    pub dest: String,
    #[serde(skip)]
    pub layouts: &'static str,
    pub drafts: String,
    #[serde(skip)]
    pub data: &'static str,
    pub include_drafts: bool,
    pub posts: String,
    pub post_path: Option<String>,
    pub post_order: SortOrder,
    pub template_extensions: Vec<String>,
    pub rss: Option<String>,
    pub jsonfeed: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub ignore: Vec<String>,
    pub excerpt_separator: String,
    // This is a debug-only field and should be transient rather than persistently set.
    #[serde(skip)]
    pub dump: Vec<Dump>,
    pub syntax_highlight: SyntaxHighlight,
    pub sass: SassOptions,
}

impl Default for ConfigBuilder {
    fn default() -> ConfigBuilder {
        ConfigBuilder {
            root: path::PathBuf::new(),
            source: "./".to_owned(),
            dest: "./".to_owned(),
            layouts: LAYOUTS_DIR,
            drafts: "_drafts".to_owned(),
            data: DATA_DIR,
            include_drafts: false,
            posts: "posts".to_owned(),
            post_path: None,
            post_order: SortOrder::default(),
            template_extensions: vec!["md".to_owned(), "liquid".to_owned()],
            rss: None,
            jsonfeed: None,
            name: None,
            description: None,
            link: None,
            ignore: vec![],
            excerpt_separator: "\n\n".to_owned(),
            dump: vec![],
            syntax_highlight: SyntaxHighlight::default(),
            sass: SassOptions::default(),
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
            dest,
            layouts,
            drafts,
            data,
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
            dump,
            syntax_highlight,
            sass,
        } = self;

        let link = link.map(|mut l| {
                                if l.ends_with('/') {
                                    l.pop();
                                }
                                l
                            });

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

        let config = Config {
            root,
            source,
            dest,
            layouts,
            drafts,
            data,
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
            dump,
            syntax_highlight,
            sass,
        };

        Ok(config)
    }
}

#[derive(Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub root: path::PathBuf,
    pub source: String,
    pub dest: String,
    pub layouts: &'static str,
    pub drafts: String,
    pub data: &'static str,
    pub include_drafts: bool,
    pub posts: String,
    pub post_path: Option<String>,
    pub post_order: SortOrder,
    pub template_extensions: Vec<String>,
    pub rss: Option<String>,
    pub jsonfeed: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub ignore: Vec<String>,
    pub excerpt_separator: String,
    pub dump: Vec<Dump>,
    pub syntax_highlight: SyntaxHighlight,
    pub sass: SassOptions,
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
                   dest: "./dest".to_owned(),
                   posts: "_my_posts".to_owned(),
                   ..Default::default()
               });
}

#[test]
fn test_from_file_rss() {
    let result = ConfigBuilder::from_file("tests/fixtures/config/rss.yml").unwrap();
    assert_eq!(result,
               ConfigBuilder {
                   root: path::Path::new("tests/fixtures/config").to_path_buf(),
                   rss: Some("rss.xml".to_owned()),
                   name: Some("My blog!".to_owned()),
                   description: Some("Blog description".to_owned()),
                   link: Some("http://example.com".to_owned()),
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
                   dest: "./dest".to_owned(),
                   posts: "_my_posts".to_owned(),
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
