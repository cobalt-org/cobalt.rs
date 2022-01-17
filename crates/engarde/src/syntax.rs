use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::{SyntaxReference, SyntaxSet};

#[derive(Debug)]
#[non_exhaustive]
pub struct Syntax {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    default_theme: Option<String>,
}

impl Syntax {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            default_theme: None,
        }
    }

    pub fn has_theme(&self, name: &str) -> bool {
        self.theme_set.themes.contains_key(name)
    }

    pub fn themes(&self) -> impl Iterator<Item = String> + '_ {
        self.theme_set.themes.keys().cloned()
    }

    pub fn syntaxes(&self) -> impl Iterator<Item = String> + '_ {
        fn reference_to_string(sd: &SyntaxReference) -> String {
            let extensions = sd.file_extensions.join(", ");
            format!("{} [{}]", sd.name, extensions)
        }

        let mut syntaxes = self
            .syntax_set
            .syntaxes()
            .iter()
            .map(reference_to_string)
            .collect::<Vec<_>>();

        // sort alphabetically with insensitive ascii case
        syntaxes.sort_by_key(|a| a.to_ascii_lowercase());

        syntaxes.into_iter()
    }

    pub fn default_theme(&self) -> Option<&str> {
        self.default_theme.as_deref()
    }

    pub fn set_default_theme(&mut self, theme: impl Into<String>) {
        self.default_theme = Some(theme.into());
    }

    pub fn format(&self, code: &str, lang: Option<&str>, theme: Option<&str>) -> String {
        if let Some(theme) = theme.or_else(|| self.default_theme()) {
            let theme = &self.theme_set.themes[theme];

            let syntax = lang
                .and_then(|l| self.syntax_set.find_syntax_by_token(l))
                .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
            highlighted_html_for_string(code, &self.syntax_set, syntax, theme)
        } else {
            crate::Raw::new().format(code, lang, theme)
        }
    }
}

impl Default for Syntax {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const CODEBLOCK: &str = "mod test {
        fn hello(arg: int) -> bool {
            \
                                      true
        }
    }
    ";

    const CODEBLOCK_RENDERED: &str =
        "<pre style=\"background-color:#2b303b;\">\n\
         <span style=\"color:#b48ead;\">mod </span>\
         <span style=\"color:#c0c5ce;\">test {\n\
         </span><span style=\"color:#c0c5ce;\">        </span>\
         <span style=\"color:#b48ead;\">fn \
         </span><span style=\"color:#8fa1b3;\">hello</span><span style=\"color:#c0c5ce;\">(\
         </span><span style=\"color:#bf616a;\">arg</span><span style=\"color:#c0c5ce;\">: int) -&gt; \
         </span><span style=\"color:#b48ead;\">bool </span><span style=\"color:#c0c5ce;\">{\n\
         </span><span style=\"color:#c0c5ce;\">            \
         </span><span style=\"color:#d08770;\">true\n\
         </span><span style=\"color:#c0c5ce;\">        }\n\
         </span><span style=\"color:#c0c5ce;\">    }\n\
         </span><span style=\"color:#c0c5ce;\">    </span></pre>\n";

    #[test]
    fn highlight_block_renders_rust() {
        let syntax = Syntax::new();
        let output = syntax.format(CODEBLOCK, Some("rust"), Some("base16-ocean.dark"));
        assert_eq!(output, CODEBLOCK_RENDERED.to_string());
    }
}
