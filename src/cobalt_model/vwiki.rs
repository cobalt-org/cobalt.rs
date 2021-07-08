use crate::error::*;
use vimwiki::{HtmlCodeConfig, HtmlConfig, Language, Page, ParseError, ToHtmlString};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VimwikiBuilder {
    pub theme: String,
    pub syntax_highlight_enabled: bool,
}

impl VimwikiBuilder {
    pub fn build(self) -> Vimwiki {
        Vimwiki {
            theme: self.theme,
            syntax_highlight_enabled: self.syntax_highlight_enabled,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vimwiki {
    theme: String,
    syntax_highlight_enabled: bool,
}

impl Vimwiki {
    pub fn parse(&self, content: &str) -> Result<String> {
        let lang = Language::from_vimwiki_str(content);

        // TODO: vimwiki crate needs to support converting ParseError<'a>
        //       into owned version of ParseError<'static> similar to elements
        //
        //       Until then, we just convert to error display message and
        //       wrap that in a general failure error type
        let page: Page = lang
            .parse()
            .map_err(|x: ParseError| failure::err_msg(format!("vimwiki parsing failed: {}", x)))?;

        let config = HtmlConfig {
            code: HtmlCodeConfig {
                theme: self.theme.to_string(),
                server_side: self.syntax_highlight_enabled,
                ..Default::default()
            },
            ..Default::default()
        };

        let s = page.to_html_string(config)?;
        Ok(s)
    }
}
