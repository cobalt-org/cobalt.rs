use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::html::{
    append_highlighted_html_for_styled_line, start_highlighted_html_snippet, IncludeBackground,
};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

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

    pub fn load_custom_syntaxes(&mut self, syntaxes_path: &Path) {
        let mut builder = self.syntax_set.clone().into_builder();
        builder.add_from_folder(syntaxes_path, true).unwrap();
        self.syntax_set = builder.build();
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

            // Essentially the same as `syntect::html::highlighted_html_for_string`,
            // but adding <code> tags between the <pre> tags
            // See: https://docs.rs/syntect/5.0.0/src/syntect/html.rs.html#269
            let mut highlighter = HighlightLines::new(syntax, theme);
            let (mut output, bg) = start_highlighted_html_snippet(theme);
            output.push_str("<code>");

            for line in LinesWithEndings::from(code) {
                let regions = highlighter.highlight_line(line, &self.syntax_set).unwrap();
                append_highlighted_html_for_styled_line(
                    &regions[..],
                    IncludeBackground::IfDifferent(bg),
                    &mut output,
                )
                .unwrap();
            }
            output.push_str("</code></pre>\n");
            output
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

    const CUSTOM_CODEBLOCK: &str = "[[[]]]]";

    const CUSTOM_CODEBLOCK_RENDERED: &str = "<pre style=\"background-color:#2b303b;\">\n\
          <span style=\"color:#c0c5ce;\">[[[]]]</span>\
          <span style=\"background-color:#bf616a;color:#2b303b;\">]</span>\
          </pre>\n";

    #[test]
    fn highlight_custom_syntax() {
        let mut syntax = Syntax::new();
        let path = Path::new("./tests/fixtures/custom_syntaxes/");
        syntax.load_custom_syntaxes(path);
        let output = syntax.format(
            CUSTOM_CODEBLOCK,
            Some("brackets"),
            Some("base16-ocean.dark"),
        );
        assert_eq!(output, CUSTOM_CODEBLOCK_RENDERED);
    }
}
