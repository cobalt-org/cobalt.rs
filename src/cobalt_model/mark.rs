use pulldown_cmark as cmark;

use syntax_highlight::decorate_markdown;
use error::*;

#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkdownBuilder {
    pub theme: String,
}

impl MarkdownBuilder {
    pub fn build(self) -> Markdown {
        Markdown { theme: self.theme }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Markdown {
    theme: String,
}

impl Markdown {
    pub fn parse(&self, content: &str) -> Result<String> {
        let mut buf = String::new();
        let options = cmark::OPTION_ENABLE_FOOTNOTES | cmark::OPTION_ENABLE_TABLES;
        let parser = cmark::Parser::new_ext(content, options);
        cmark::html::push_html(&mut buf, decorate_markdown(parser, &self.theme));
        Ok(buf)
    }
}
