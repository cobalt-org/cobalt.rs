use std::fmt;
use std::path;

use super::files;
use crate::error::*;
use crate::syntax_highlight;
use liquid;

fn load_partials_from_path(root: path::PathBuf) -> Result<liquid::Partials> {
    let mut source = liquid::Partials::empty();

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
        source.add(rel_path, content);
    }
    Ok(source)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiquidBuilder {
    pub includes_dir: path::PathBuf,
    pub theme: String,
}

impl LiquidBuilder {
    pub fn build(self) -> Result<Liquid> {
        let highlight = Self::highlight(self.theme)?;
        let parser = liquid::ParserBuilder::with_liquid()
            .extra_filters()
            .jekyll_filters()
            .partials(load_partials_from_path(self.includes_dir)?)
            .block("highlight", highlight)
            .build()?;
        Ok(Liquid { parser })
    }

    fn highlight(theme: String) -> Result<Box<dyn liquid::compiler::ParseBlock>> {
        let result: Result<()> = match syntax_highlight::has_syntax_theme(&theme) {
            Ok(true) => Ok(()),
            Ok(false) => Err(failure::format_err!(
                "Syntax theme '{}' is unsupported",
                theme
            )),
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
