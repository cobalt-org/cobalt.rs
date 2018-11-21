use std::borrow::Cow::Owned;
use std::io::Write;

use itertools::Itertools;
use liquid;
use liquid::compiler::Element::{self, Expression, Raw, Tag};
use liquid::compiler::LiquidOptions;
use liquid::compiler::Token::{self, Identifier};
use liquid::interpreter::{Context, Renderable};
use liquid_error::ResultLiquidChainExt;
use pulldown_cmark as cmark;
use pulldown_cmark::Event::{self, End, Html, Start, Text};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::{
    highlighted_html_for_string, start_highlighted_html_snippet, styled_line_to_highlighted_html,
    IncludeBackground,
};
use syntect::parsing::{SyntaxReference, SyntaxSet};

use error;

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
    fn render_to(&self, writer: &mut Write, _context: &mut Context) -> Result<(), liquid::Error> {
        let syntax = match self.lang {
            Some(ref lang) => SETUP.syntax_set.find_syntax_by_token(lang),
            _ => None,
        }.unwrap_or_else(|| SETUP.syntax_set.find_syntax_plain_text());

        write!(
            writer,
            "{}",
            highlighted_html_for_string(&self.code, &SETUP.syntax_set, syntax, &self.theme,)
        ).chain("Failed to render")?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct CodeBlockParser {
    syntax_theme: String,
}

impl CodeBlockParser {
    pub fn new(syntax_theme: String) -> Self {
        Self { syntax_theme }
    }
}

impl liquid::compiler::ParseBlock for CodeBlockParser {
    fn parse(
        &self,
        _tag_name: &str,
        arguments: &[Token],
        tokens: &[Element],
        _options: &LiquidOptions,
    ) -> Result<Box<Renderable>, liquid::Error> {
        let content = tokens.iter().fold("".to_owned(), |a, b| {
            match *b {
                Expression(_, ref text) | Tag(_, ref text) | Raw(ref text) => text,
            }.to_owned()
                + &a
        });

        let lang = match arguments.iter().next() {
            Some(&Identifier(ref x)) => Some(x.clone()),
            _ => None,
        };

        Ok(Box::new(CodeBlock {
            code: content,
            lang,
            theme: SETUP.theme_set.themes[&self.syntax_theme].clone(),
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
            Some(item) => {
                if let Text(text) = item {
                    if let Some(ref mut h) = self.h {
                        let highlighted = &h.highlight(&text, &SETUP.syntax_set);
                        let html =
                            styled_line_to_highlighted_html(highlighted, IncludeBackground::Yes);
                        Some(Html(Owned(html)))
                    } else {
                        Some(Text(text))
                    }
                } else {
                    if let Start(cmark::Tag::CodeBlock(ref info)) = item {
                        // set local highlighter, if found
                        let cur_syntax = info
                            .clone()
                            .split(' ')
                            .next()
                            .and_then(|lang| SETUP.syntax_set.find_syntax_by_token(lang))
                            .unwrap_or_else(|| SETUP.syntax_set.find_syntax_plain_text());
                        self.h = Some(HighlightLines::new(cur_syntax, self.theme));
                        let snippet = start_highlighted_html_snippet(self.theme);
                        return Some(Html(Owned(snippet.0)));
                    }
                    if let End(cmark::Tag::CodeBlock(_)) = item {
                        // reset highlighter
                        self.h = None;
                        // close the code block
                        return Some(Html(Owned("</pre>".to_owned())));
                    }

                    Some(item)
                }
            }
            None => None,
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
        let highlight: Box<liquid::compiler::ParseBlock> =
            Box::new(CodeBlockParser::new("base16-ocean.dark".to_owned()));
        let parser = liquid::ParserBuilder::new()
            .block("highlight", highlight)
            .build();
        let template = parser
            .parse(&format!(
                "{{% highlight rust %}}{}{{% endhighlight %}}",
                CODE_BLOCK
            )).unwrap();
        let output = template.render(&liquid::value::Object::new());
        assert_diff!(CODEBLOCK_RENDERED, &output.unwrap(), "\n", 0);
    }

    const MARKDOWN_RENDERED: &str =
        "<pre style=\"background-color:#2b303b;\">\n\
         <span style=\"background-color:#2b303b;color:#b48ead;\">mod </span>\
         <span style=\"background-color:#2b303b;color:#c0c5ce;\">test {\n\
         </span><span style=\"background-color:#2b303b;color:#c0c5ce;\">        </span>\
         <span style=\"background-color:#2b303b;color:#b48ead;\">fn </span>\
         <span style=\"background-color:#2b303b;color:#8fa1b3;\">hello</span>\
         <span style=\"background-color:#2b303b;color:#c0c5ce;\">(</span>\
         <span style=\"background-color:#2b303b;color:#bf616a;\">arg</span>\
         <span style=\"background-color:#2b303b;color:#c0c5ce;\">: int) -&gt; </span>\
         <span style=\"background-color:#2b303b;color:#b48ead;\">bool </span>\
         <span style=\"background-color:#2b303b;color:#c0c5ce;\">{\n\
         </span><span style=\"background-color:#2b303b;color:#c0c5ce;\">            </span>\
         <span style=\"background-color:#2b303b;color:#d08770;\">true\n\
         </span><span style=\"background-color:#2b303b;color:#c0c5ce;\">        </span>\
         <span style=\"background-color:#2b303b;color:#c0c5ce;\">}\n\
         </span><span style=\"background-color:#2b303b;color:#c0c5ce;\">    </span>\
         <span style=\"background-color:#2b303b;color:#c0c5ce;\">}\n\
         </span><span style=\"background-color:#2b303b;color:#c0c5ce;\">    </span>\
         <span style=\"background-color:#2b303b;color:#c0c5ce;\">\n\
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
        assert_diff!(MARKDOWN_RENDERED, &buf, "\n", 0);
    }
}
