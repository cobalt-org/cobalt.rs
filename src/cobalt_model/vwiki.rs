use crate::error::*;
use serde::{Deserialize, Serialize};
use vimwiki::{HtmlCodeConfig, HtmlConfig, Language, Page, ParseError, ToHtmlString};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VimwikiBuilder {
    pub theme: kstring::KString,
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
    theme: kstring::KString,
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
        let page: Page<'_> = lang.parse().map_err(|x: ParseError<'_>| {
            failure::err_msg(format!("vimwiki parsing failed: {}", x))
        })?;

        let config = HtmlConfig {
            code: HtmlCodeConfig {
                theme: self.theme.as_str().to_owned(),
                server_side: self.syntax_highlight_enabled,
                ..Default::default()
            },
            ..Default::default()
        };

        let s = page.to_html_string(config)?;
        Ok(s)
    }
}
