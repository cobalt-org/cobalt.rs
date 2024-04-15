use std::convert::TryInto;
use std::fmt;
use std::path;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "unstable", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "unstable"), non_exhaustive)]
pub struct Config {
    #[serde(skip)]
    pub root: path::PathBuf,
    pub source: crate::RelPath,
    pub destination: crate::RelPath,
    #[serde(skip)]
    pub abs_dest: Option<path::PathBuf>,
    pub include_drafts: bool,
    pub default: frontmatter::Frontmatter,
    pub pages: PageCollection,
    pub posts: PostCollection,
    pub site: Site,
    pub template_extensions: Vec<liquid_core::model::KString>,
    pub ignore: Vec<liquid_core::model::KString>,
    pub syntax_highlight: SyntaxHighlight,
    #[serde(skip)]
    pub layouts_dir: &'static str,
    #[serde(skip)]
    pub includes_dir: &'static str,
    pub assets: Assets,
    pub minify: Minify,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            root: Default::default(),
            source: "./".try_into().unwrap(),
            destination: "./_site".try_into().unwrap(),
            abs_dest: Default::default(),
            include_drafts: false,
            default: Default::default(),
            pages: Default::default(),
            posts: Default::default(),
            site: Default::default(),
            template_extensions: vec!["md".into(), "wiki".into(), "liquid".into()],
            ignore: Default::default(),
            syntax_highlight: SyntaxHighlight::default(),
            layouts_dir: "_layouts",
            includes_dir: "_includes",
            assets: Assets::default(),
            minify: Minify::default(),
        }
    }
}

impl Config {
    pub fn from_file<P: Into<path::PathBuf>>(path: P) -> Result<Config> {
        Self::from_file_internal(path.into())
    }

    fn from_file_internal(path: path::PathBuf) -> Result<Config> {
        let content = std::fs::read_to_string(&path).map_err(|e| {
            Status::new("Failed to read config")
                .with_source(e)
                .context_with(|c| c.insert("Path", path.display().to_string()))
        })?;

        let mut config = if content.trim().is_empty() {
            Config::default()
        } else {
            serde_yaml::from_str(&content).map_err(|e| {
                Status::new("Failed to parse config")
                    .with_source(e)
                    .context_with(|c| c.insert("Path", path.display().to_string()))
            })?
        };

        let mut root = path;
        root.pop(); // Remove filename
        if root == std::path::Path::new("") {
            root = std::path::Path::new(".").to_owned();
        }
        config.root = root;

        Ok(config)
    }

    pub fn from_cwd<P: Into<path::PathBuf>>(cwd: P) -> Result<Config> {
        Self::from_cwd_internal(cwd.into())
    }

    fn from_cwd_internal(cwd: path::PathBuf) -> Result<Config> {
        let file_path = find_project_file(&cwd, "_cobalt.yml");
        let config = file_path
            .map(|p| {
                log::debug!("Using config file `{}`", p.display());
                Self::from_file(&p)
            })
            .unwrap_or_else(|| {
                log::warn!("No _cobalt.yml file found in current directory, using default config.");
                let config = Config {
                    root: cwd,
                    ..Default::default()
                };
                Ok(config)
            })?;
        Ok(config)
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut converted = serde_yaml::to_string(self).map_err(|_| fmt::Error)?;
        converted.drain(..4);
        write!(f, "{}", converted)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "unstable", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "unstable"), non_exhaustive)]
pub struct SyntaxHighlight {
    pub theme: liquid_core::model::KString,
    pub enabled: bool,
}

impl Default for SyntaxHighlight {
    fn default() -> Self {
        Self {
            theme: "base16-ocean.dark".into(),
            enabled: true,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "unstable", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "unstable"), non_exhaustive)]
pub struct Minify {
    pub html: bool,
    pub css: bool,
    pub js: bool,
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_file_ok() {
        let result = Config::from_file("tests/fixtures/config/_cobalt.yml").unwrap();
        assert_eq!(
            result.root,
            path::Path::new("tests/fixtures/config").to_path_buf()
        );
    }

    #[test]
    fn test_from_file_alternate_name() {
        let result = Config::from_file("tests/fixtures/config/rss.yml").unwrap();
        assert_eq!(
            result.root,
            path::Path::new("tests/fixtures/config").to_path_buf()
        );
    }

    #[test]
    fn test_from_file_empty() {
        let result = Config::from_file("tests/fixtures/config/empty.yml").unwrap();
        assert_eq!(
            result.root,
            path::Path::new("tests/fixtures/config").to_path_buf()
        );
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

    #[test]
    fn test_from_cwd_ok() {
        let result = Config::from_cwd("tests/fixtures/config/child").unwrap();
        assert_eq!(
            result.root,
            path::Path::new("tests/fixtures/config").to_path_buf()
        );
    }

    #[test]
    fn test_from_cwd_not_found() {
        let result = Config::from_cwd("tests/fixtures").unwrap();
        assert_eq!(result.root, path::Path::new("tests/fixtures").to_path_buf());
    }

    #[test]
    fn find_project_file_same_dir() {
        let actual = find_project_file("tests/fixtures/config", "_cobalt.yml").unwrap();
        let expected = path::Path::new("tests/fixtures/config/_cobalt.yml");
        assert_eq!(actual, expected);
    }

    #[test]
    fn find_project_file_parent_dir() {
        let actual = find_project_file("tests/fixtures/config/child", "_cobalt.yml").unwrap();
        let expected = path::Path::new("tests/fixtures/config/_cobalt.yml");
        assert_eq!(actual, expected);
    }

    #[test]
    fn find_project_file_doesnt_exist() {
        let expected = path::Path::new("<NOT FOUND>");
        let actual =
            find_project_file("tests/fixtures/", "_cobalt.yml").unwrap_or_else(|| expected.into());
        assert_eq!(actual, expected);
    }
}
