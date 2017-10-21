use std::default::Default;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use error::Result;
use serde_yaml;

use legacy::wildwest;
use syntax_highlight::has_syntax_theme;

arg_enum! {
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
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub source: String,
    pub dest: String,
    pub layouts: String,
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
    // This is a debug-only field and should be transient rather than persistently set.
    #[serde(skip)]
    pub dump: Vec<Dump>,
    pub syntax_highlight: SyntaxHighlight,
    pub sass: SassOptions,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            source: "./".to_owned(),
            dest: "./".to_owned(),
            layouts: "_layouts".to_owned(),
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

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let content = {
            let mut buffer = String::new();
            let mut f = File::open(path)?;
            f.read_to_string(&mut buffer)?;
            buffer
        };

        if content.trim().is_empty() {
            return Ok(Config::default());
        }

        let config: wildwest::GlobalConfig = serde_yaml::from_str(&content)?;
        let mut config: Config = config.into();

        config.link = if let Some(ref link) = config.link {
            let mut link = link.to_owned();
            if !link.ends_with('/') {
                link += "/";
            }
            Some(link)
        } else {
            None
        };

        let result: Result<()> = match has_syntax_theme(&config.syntax_highlight.theme) {
            Ok(true) => Ok(()),
            Ok(false) => {
                Err(format!("Syntax theme '{}' is unsupported",
                            config.syntax_highlight.theme)
                        .into())
            }
            Err(err) => {
                warn!("Syntax theme named '{}' ignored. Reason: {}",
                      config.syntax_highlight.theme,
                      err);
                Ok(())
            }
        };
        result?;

        Ok(config)
    }
}

#[test]
fn test_from_file_ok() {
    let result = Config::from_file("tests/fixtures/config/.cobalt.yml").unwrap();
    assert_eq!(result,
               Config {
                   dest: "./dest".to_owned(),
                   layouts: "_my_layouts".to_owned(),
                   posts: "_my_posts".to_owned(),
                   ..Default::default()
               });
}

#[test]
fn test_from_file_rss() {
    let result = Config::from_file("tests/fixtures/config/rss.yml").unwrap();
    assert_eq!(result,
               Config {
                   rss: Some("rss.xml".to_owned()),
                   name: Some("My blog!".to_owned()),
                   description: Some("Blog description".to_owned()),
                   link: Some("http://example.com/".to_owned()),
                   ..Default::default()
               });
}

#[test]
fn test_from_file_empty() {
    let result = Config::from_file("tests/fixtures/config/empty.yml").unwrap();
    assert_eq!((result), Config { ..Default::default() });
}

#[test]
fn test_from_file_invalid_syntax() {
    let result = Config::from_file("tests/fixtures/config/invalid_syntax.yml");
    assert!(result.is_err());
}

#[test]
fn test_from_file_not_found() {
    let result = Config::from_file("tests/fixtures/config/config_does_not_exist.yml");
    assert!(result.is_err());
}
