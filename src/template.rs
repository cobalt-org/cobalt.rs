use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path;
use std::result;

use liquid;

use config;
use error::*;
use files;
use syntax_highlight;

#[derive(Clone, Debug, Default)]
struct InMemoryTemplateRepository {
    templates: HashMap<String, String>,
    legacy_path: Option<path::PathBuf>,
}

impl InMemoryTemplateRepository {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn load_from_path<R: Into<path::PathBuf>>(self, root: R) -> Result<Self> {
        self.load_from_pathbuf(root.into())
    }

    /// Overwrites previous, conflicting snippets
    fn load_from_pathbuf(mut self, root: path::PathBuf) -> Result<Self> {
        let template_files = files::FilesBuilder::new(&root)?;
        let template_files = template_files.build()?;
        for file_path in template_files.files() {
            let rel_path = file_path
                .strip_prefix(root.as_path())
                .expect("file was found under the root")
                .to_str()
                .expect("only UTF-8 characters supported in paths")
                .to_owned();
            let content = files::read_file(file_path)?;
            self.templates.insert(rel_path, content);
        }
        Ok(self)
    }

    pub fn set_legacy_path(mut self, legacy_path: Option<path::PathBuf>) -> Self {
        self.legacy_path = legacy_path;
        self
    }
}

impl liquid::TemplateRepository for InMemoryTemplateRepository {
    fn read_template(&self, path: &str) -> result::Result<String, liquid::Error> {
        self.templates
            .get(path)
            .map(|s| Ok(s.to_owned()))
            .unwrap_or_else(|| {
                let legacy_path = self.legacy_path
                    .clone()
                    .ok_or_else(|| liquid::Error::from(&*format!("{:?} does not exist", path)))?;
                let abs_path = legacy_path.join(path);
                if !abs_path.exists() {
                    return Err(liquid::Error::from(&*format!("{:?} does not exist", path)));
                }

                warn!("Loading `include`s relative to `source` is deprecated, see {}.",
                      path);
                let mut file = fs::File::open(abs_path)?;
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                Ok(content)
            })
    }
}

#[derive(Clone)]
pub struct LiquidParser {
    parser: liquid::LiquidOptions,
}

impl LiquidParser {
    pub fn with_config(config: &config::Config) -> Result<Self> {
        let mut parser = liquid::LiquidOptions::default();
        let repo = InMemoryTemplateRepository::new()
            .load_from_path(config.source.join(&config.includes_dir))?
            .set_legacy_path(Some(config.source.clone()));
        parser.template_repository = Box::new(repo);
        let highlight: Box<liquid::ParseBlock> = {
            let syntax_theme = config.syntax_highlight.theme.clone();
            Box::new(syntax_highlight::CodeBlockParser::new(syntax_theme))
        };
        parser.blocks.insert("highlight".to_string(), highlight);
        Ok(Self { parser })
    }

    pub fn parse(&self, template: &str) -> Result<liquid::Template> {
        let template = liquid::parse(template, self.parser.clone())?;
        Ok(template)
    }
}
