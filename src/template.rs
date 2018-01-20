use std::collections::HashMap;
use std::path;
use std::result;

use liquid;

use error::*;
use cobalt_model;
use cobalt_model::files;
use syntax_highlight;

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
        let template_files = files::FilesBuilder::new(&root)?
            .ignore_hidden(false)?
            .build()?;
        for file_path in template_files.files() {
            let rel_path = file_path
                .strip_prefix(root.as_path())
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
                    .ok_or_else(|| liquid::Error::from(&*format!("{:?} does not exist", path)))?
                    .include(path)?;
                warn!("Loading `include`s relative to `source` is deprecated, see {}.",
                      path);
                Ok(content)
            })
    }
}

pub struct LiquidParser {
    parser: liquid::Parser,
}

impl LiquidParser {
    pub fn with_config(config: &cobalt_model::Config) -> Result<Self> {
        let repo = InMemoryInclude::new()
            .load_from_path(config.source.join(&config.includes_dir))?
            .set_legacy_path(Some(config.source.clone()));
        let highlight: Box<liquid::compiler::ParseBlock> = {
            let syntax_theme = config.syntax_highlight.theme.clone();
            Box::new(syntax_highlight::CodeBlockParser::new(syntax_theme))
        };
        let parser = liquid::ParserBuilder::with_liquid()
            .extra_filters()
            .include_source(Box::new(repo))
            .block("highlight", highlight)
            .build();
        Ok(Self { parser })
    }

    pub fn parse(&self, template: &str) -> Result<liquid::Template> {
        let template = self.parser.parse(template)?;
        Ok(template)
    }
}
