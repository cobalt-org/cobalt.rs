use liquid;

use config;
use error::*;
use syntax_highlight;

#[derive(Clone)]
pub struct LiquidParser {
    parser: liquid::LiquidOptions,
}

impl LiquidParser {
    pub fn with_config(config: &config::Config) -> Result<Self> {
        let mut parser = liquid::LiquidOptions::default();
        parser.template_repository =
            Box::new(liquid::LocalTemplateRepository::new(config.source.clone()));
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
