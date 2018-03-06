use std::collections::HashMap;
use std::fmt;
use std::path;
use std::result;

use liquid;
use syntax_highlight;
use error::*;
use super::files;

#[derive(Clone, Debug, Default)]
struct InMemoryInclude {
    templates: HashMap<String, String>,
    legacy: Option<liquid::compiler::FilesystemInclude>,
}

impl InMemoryInclude {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn load_from_path<R: Into<path::PathBuf>>(self, root: R) -> Result<Self> {
        self.load_from_pathbuf(root.into())
    }

    /// Overwrites previous, conflicting snippets
    fn load_from_pathbuf(mut self, root: path::PathBuf) -> Result<Self> {
        debug!("Loading snippets from {:?}", root);
        let template_files = files::FilesBuilder::new(root)?
            .ignore_hidden(false)?
            .build()?;
        for file_path in template_files.files() {
            let rel_path = file_path
                .strip_prefix(template_files.root())
                .expect("file was found under the root")
                .to_str()
                .expect("only UTF-8 characters supported in paths")
                .to_owned();
            trace!("Loading snippet {:?}", rel_path);
            let content = files::read_file(file_path)?;
            self.templates.insert(rel_path, content);
        }
        Ok(self)
    }

    pub fn set_legacy_path(mut self, legacy_path: Option<path::PathBuf>) -> Self {
        self.legacy = legacy_path.map(liquid::compiler::FilesystemInclude::new);
        self
    }
}

impl liquid::compiler::Include for InMemoryInclude {
    fn include(&self, path: &str) -> result::Result<String, liquid::Error> {
        self.templates
            .get(path)
            .map(|s| Ok(s.to_owned()))
            .unwrap_or_else(|| {
                let content = self.legacy
                    .as_ref()
                    .ok_or_else(|| liquid::Error::with_msg("No legacy path specified"))?
                    .include(path)?;
                warn!(
                    "Loading `include`s relative to `source` is deprecated, see {}.",
                    path
                );
                Ok(content)
            })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiquidBuilder {
    pub includes_dir: path::PathBuf,
    pub legacy_path: path::PathBuf,
    pub theme: String,
}

impl LiquidBuilder {
    pub fn build(self) -> Result<Liquid> {
        let repo = InMemoryInclude::new()
            .load_from_path(self.includes_dir)?
            .set_legacy_path(Some(self.legacy_path));
        let highlight = Self::highlight(self.theme)?;
        let parser = liquid::ParserBuilder::with_liquid()
            .extra_filters()
            .include_source(Box::new(repo))
            .block("highlight", highlight)
            .build();
        Ok(Liquid { parser })
    }

    fn highlight(theme: String) -> Result<Box<liquid::compiler::ParseBlock>> {
        let result: Result<()> = match syntax_highlight::has_syntax_theme(&theme) {
            Ok(true) => Ok(()),
            Ok(false) => Err(format!("Syntax theme '{}' is unsupported", theme).into()),
            Err(err) => {
                warn!("Syntax theme named '{}' ignored. Reason: {}", theme, err);
                Ok(())
            }
        };
        result?;
        let block = syntax_highlight::CodeBlockParser::new(theme);
        Ok(Box::new(block))
    }
}

pub struct Liquid {
    parser: liquid::Parser,
}

impl Liquid {
    pub fn parse(&self, template: &str) -> Result<liquid::Template> {
        let template = self.parser.parse(template)?;
        Ok(template)
    }
}

impl fmt::Debug for Liquid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Liquid{{}}")
    }
}
