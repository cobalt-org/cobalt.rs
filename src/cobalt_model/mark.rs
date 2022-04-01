use pulldown_cmark as cmark;
use serde::Serialize;

use crate::error::*;
use crate::syntax_highlight::decorate_markdown;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MarkdownBuilder {
    pub theme: Option<liquid::model::KString>,
}

impl MarkdownBuilder {
    pub fn build(self) -> Markdown {
        Markdown { theme: self.theme }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Markdown {
    theme: Option<liquid::model::KString>,
}

impl Markdown {
    pub fn parse(&self, content: &str) -> Result<String> {
        let mut buf = String::new();
        let options = cmark::Options::ENABLE_FOOTNOTES
            | cmark::Options::ENABLE_TABLES
            | cmark::Options::ENABLE_STRIKETHROUGH
            | cmark::Options::ENABLE_TASKLISTS;
        let parser = cmark::Parser::new_ext(content, options);
        cmark::html::push_html(&mut buf, decorate_markdown(parser, self.theme.as_deref()));
        Ok(buf)
    }
}
