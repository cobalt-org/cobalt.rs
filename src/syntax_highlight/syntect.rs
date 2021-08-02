use std::io::Write;

use itertools::Itertools;
use liquid_core::error::ResultLiquidReplaceExt;
use liquid_core::parser::TryMatchToken;
use liquid_core::Language;
use liquid_core::TagBlock;
use liquid_core::TagTokenIter;
use liquid_core::ValueView;
use liquid_core::{Renderable, Runtime};
use pulldown_cmark as cmark;
use pulldown_cmark::Event::{self, End, Html, Start, Text};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::{
    highlighted_html_for_string, start_highlighted_html_snippet, IncludeBackground,
};
use syntect::parsing::{SyntaxReference, SyntaxSet};

use crate::error;

struct Setup {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

unsafe impl Send for Setup {}
unsafe impl Sync for Setup {}

lazy_static! {
    static ref SETUP: Setup = Setup {
        syntax_set: SyntaxSet::load_defaults_newlines(),
        theme_set: ThemeSet::load_defaults()
    };
}

pub fn has_syntax_theme(name: &str) -> error::Result<bool> {
    Ok(SETUP.theme_set.themes.contains_key(name))
}

pub fn list_syntax_themes<'a>() -> Vec<&'a String> {
    SETUP.theme_set.themes.keys().collect::<Vec<_>>()
}

pub fn list_syntaxes() -> Vec<String> {
    fn reference_to_string(sd: &SyntaxReference) -> String {
        let extensions = sd.file_extensions.iter().join(&", ".to_owned());
        format!("{} [{}]", sd.name, extensions)
    }

    let mut syntaxes = SETUP
        .syntax_set
        .syntaxes()
        .iter()
        .map(reference_to_string)
        .collect::<Vec<_>>();

    // sort alphabetically with insensitive ascii case
    syntaxes.sort_by_key(|a| a.to_ascii_lowercase());

    syntaxes
}

#[derive(Clone, Debug)]
struct CodeBlock {
    lang: Option<String>,
    code: String,
    theme: Theme,
}

impl Renderable for CodeBlock {
    fn render_to(
        &self,
        writer: &mut dyn Write,
        _context: &dyn Runtime,
    ) -> Result<(), liquid_core::Error> {
        let syntax = match self.lang {
            Some(ref lang) => SETUP.syntax_set.find_syntax_by_token(lang),
            _ => None,
        }
        .unwrap_or_else(|| SETUP.syntax_set.find_syntax_plain_text());

        write!(
            writer,
            "{}",
            highlighted_html_for_string(&self.code, &SETUP.syntax_set, syntax, &self.theme,)
        )
        .replace("Failed to render")?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct CodeBlockParser {
    syntax_theme: kstring::KString,
}

impl CodeBlockParser {
    pub fn new(syntax_theme: kstring::KString) -> Self {
        Self { syntax_theme }
    }
}

impl liquid_core::BlockReflection for CodeBlockParser {
    fn start_tag(&self) -> &'static str {
        "highlight"
    }

    fn end_tag(&self) -> &'static str {
        "endhighlight"
    }

    fn description(&self) -> &'static str {
        "Syntax highlight code using HTML"
    }
}

impl liquid_core::ParseBlock for CodeBlockParser {
    fn reflection(&self) -> &dyn liquid_core::BlockReflection {
        self
    }

    fn parse(
        &self,
        mut arguments: TagTokenIter,
        mut tokens: TagBlock,
        _options: &Language,
    ) -> Result<Box<dyn Renderable>, liquid_core::Error> {
        let lang = arguments
            .expect_next("Identifier or literal expected.")
            .ok()
            .map(|lang| {
                // This may accept strange inputs such as `{% include 0 %}` or `{% include filterchain | filter:0 %}`.
                // Those inputs would fail anyway by there being not a path with those langs so they are not a big concern.
                match lang.expect_literal() {
                    // Using `to_str()` on literals ensures `Strings` will have their quotes trimmed.
                    TryMatchToken::Matches(lang) => lang.to_kstr().into_string(),
                    TryMatchToken::Fails(lang) => lang.as_str().to_string(),
                }
            });
        // no more arguments should be supplied, trying to supply them is an error
        arguments.expect_nothing()?;

        let mut content = String::new();
        while let Some(element) = tokens.next()? {
            content.push_str(element.as_str());
        }
        tokens.assert_empty();

        Ok(Box::new(CodeBlock {
            code: content,
            lang,
            theme: SETUP.theme_set.themes[self.syntax_theme.as_str()].clone(),
        }))
    }
}

pub struct DecoratedParser<'a> {
    h: Option<HighlightLines<'a>>,
    parser: cmark::Parser<'a>,
    theme: &'a Theme,
}

impl<'a> DecoratedParser<'a> {
    pub fn new(parser: cmark::Parser<'a>, theme: &'a Theme) -> Self {
        DecoratedParser {
            h: None,
            parser,
            theme,
        }
    }
}

impl<'a> Iterator for DecoratedParser<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Event<'a>> {
        match self.parser.next() {
            Some(Text(text)) => {
                if let Some(ref mut h) = self.h {
                    let mut html = String::new();
                    for line in syntect::util::LinesWithEndings::from(&text) {
                        let regions = h.highlight(line, &SETUP.syntax_set);
                        syntect::html::append_highlighted_html_for_styled_line(
                            &regions[..],
                            IncludeBackground::No,
                            &mut html,
                        );
                    }
                    Some(Html(pulldown_cmark::CowStr::Boxed(html.into_boxed_str())))
                } else {
                    Some(Text(text))
                }
            }
            Some(Start(cmark::Tag::CodeBlock(info))) => {
                let tag = match info {
                    pulldown_cmark::CodeBlockKind::Indented => "",
                    pulldown_cmark::CodeBlockKind::Fenced(ref tag) => tag.as_ref(),
                };
                // set local highlighter, if found
                let cur_syntax = tag
                    .split(' ')
                    .next()
                    .and_then(|lang| SETUP.syntax_set.find_syntax_by_token(lang))
                    .unwrap_or_else(|| SETUP.syntax_set.find_syntax_plain_text());
                self.h = Some(HighlightLines::new(cur_syntax, self.theme));
                let snippet = start_highlighted_html_snippet(self.theme);
                Some(Html(pulldown_cmark::CowStr::Boxed(
                    snippet.0.into_boxed_str(),
                )))
            }
            Some(End(cmark::Tag::CodeBlock(_))) => {
                // reset highlighter
                self.h = None;
                // close the code block
                Some(Html(pulldown_cmark::CowStr::Boxed(
                    "</pre>".to_owned().into_boxed_str(),
                )))
            }
            item => item,
        }
    }
}

pub fn decorate_markdown<'a>(parser: cmark::Parser<'a>, theme_name: &str) -> DecoratedParser<'a> {
    DecoratedParser::new(parser, &SETUP.theme_set.themes[theme_name])
}

#[cfg(test)]
mod test {
    use super::*;

    const CODE_BLOCK: &str = "mod test {
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
        let highlight: Box<dyn liquid_core::ParseBlock> =
            Box::new(CodeBlockParser::new("base16-ocean.dark".into()));
        let parser = liquid::ParserBuilder::new()
            .block(highlight)
            .build()
            .unwrap();
        let template = parser
            .parse(&format!(
                "{{% highlight rust %}}{}{{% endhighlight %}}",
                CODE_BLOCK
            ))
            .unwrap();
        let output = template.render(&liquid::Object::new());
        difference::assert_diff!(CODEBLOCK_RENDERED, &output.unwrap(), "\n", 0);
    }

    const MARKDOWN_RENDERED: &str =
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
         </span><span style=\"color:#c0c5ce;\">    \n\
         </span></pre>";

    #[test]
    fn markdown_renders_rust() {
        let html = format!(
            "```rust
{}
```",
            CODE_BLOCK
        );

        let mut buf = String::new();
        let parser = cmark::Parser::new(&html);
        cmark::html::push_html(&mut buf, decorate_markdown(parser, "base16-ocean.dark"));
        difference::assert_diff!(MARKDOWN_RENDERED, &buf, "\n", 0);
    }
}
