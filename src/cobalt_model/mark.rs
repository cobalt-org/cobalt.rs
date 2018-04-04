use pulldown_cmark as cmark;

use syntax_highlight::decorate_markdown;
use error::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkdownBuilder {
    pub theme: String,
    pub syntax_highlight_enabled: bool,
}

impl MarkdownBuilder {
    pub fn build(self) -> Markdown {
        Markdown {
            theme: self.theme,
            syntax_highlight_enabled: self.syntax_highlight_enabled,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Markdown {
    theme: String,
    syntax_highlight_enabled: bool,
}

impl Markdown {
    pub fn parse(&self, content: &str) -> Result<String> {
        let mut buf = String::new();
        let options = cmark::OPTION_ENABLE_FOOTNOTES | cmark::OPTION_ENABLE_TABLES;
        let parser = cmark::Parser::new_ext(content, options);
        if self.syntax_highlight_enabled {
            cmark::html::push_html(&mut buf, decorate_markdown(parser, &self.theme));
        } else {
            cmark::html::push_html(&mut buf, parser);
        }
        Ok(buf)
    }
}
