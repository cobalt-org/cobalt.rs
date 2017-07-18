extern crate syntect;
extern crate liquid;
extern crate pulldown_cmark as cmark;

use error;
use itertools::Itertools;

use liquid::Renderable;
use liquid::Context;
use liquid::Token::{self, Identifier};
use liquid::lexer::Element::{self, Expression, Tag, Raw};
use liquid::Error;

use syntect::parsing::{SyntaxDefinition, SyntaxSet};
use syntect::highlighting::{ThemeSet, Theme};
use syntect::html::{IncludeBackground, highlighted_snippet_for_string, styles_to_coloured_html,
                    start_coloured_html_snippet};
use syntect::easy::HighlightLines;

use std::ascii::AsciiExt;
use std::borrow::Cow::Owned;

use self::cmark::Parser;
use self::cmark::Tag as cmarkTag;
use self::cmark::Event::{self, Start, End, Text, Html};

struct Setup {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

unsafe impl Send for Setup {}
unsafe impl Sync for Setup {}

lazy_static!{
    static ref SETUP: Setup = Setup {
        syntax_set: SyntaxSet::load_defaults_newlines(),
        theme_set: ThemeSet::load_defaults()
    };
}

struct CodeBlock {
    lang: Option<String>,
    code: String,
    theme: Theme,
}

impl Renderable for CodeBlock {
    fn render(&self, _: &mut Context) -> Result<Option<String>, Error> {
        let syntax = match self.lang {
                Some(ref lang) => SETUP.syntax_set.find_syntax_by_token(lang),
                _ => None,
            }
            .unwrap_or_else(|| SETUP.syntax_set.find_syntax_plain_text());

        Ok(Some(highlighted_snippet_for_string(&self.code, syntax, &self.theme)))
    }
}

pub struct DecoratedParser<'a> {
    h: Option<HighlightLines<'a>>,
    parser: Parser<'a>,
    theme: &'a Theme,
}

impl<'a> DecoratedParser<'a> {
    pub fn new(parser: Parser<'a>, theme: &'a Theme) -> Self {
        DecoratedParser {
            h: None,
            parser: parser,
            theme: &theme,
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
                        let highlighted = &h.highlight(&text);
                        let html = styles_to_coloured_html(highlighted, IncludeBackground::Yes);
                        Some(Html(Owned(html)))
                    } else {
                        Some(Text(text))
                    }
                } else {
                    if let Start(cmarkTag::CodeBlock(ref info)) = item {
                        // set local highlighter, if found
                        let cur_syntax =
                            info.clone()
                                .split(' ')
                                .next()
                                .and_then(|lang| SETUP.syntax_set.find_syntax_by_token(lang))
                                .unwrap_or_else(|| SETUP.syntax_set.find_syntax_plain_text());
                        self.h = Some(HighlightLines::new(&cur_syntax, self.theme));
                        let snippet = start_coloured_html_snippet(self.theme);
                        return Some(Html(Owned(snippet)));
                    }
                    if let End(cmarkTag::CodeBlock(_)) = item {
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

pub fn initialize_codeblock(arguments: &[Token],
                            tokens: &[Element],
                            theme_name: &str)
                            -> Result<Box<Renderable>, Error> {

    let content = tokens
        .iter()
        .fold("".to_owned(), |a, b| {
            match *b {
                    Expression(_, ref text) |
                    Tag(_, ref text) |
                    Raw(ref text) => text,
                }
                .to_owned() + &a
        });

    let lang = match arguments.iter().next() {
        Some(&Identifier(ref x)) => Some(x.clone()),
        _ => None,
    };

    Ok(Box::new(CodeBlock {
                    code: content,
                    lang: lang,
                    theme: SETUP.theme_set.themes[theme_name].clone(),
                }))
}

pub fn decorate_markdown<'a>(parser: Parser<'a>, theme_name: &str) -> DecoratedParser<'a> {
    DecoratedParser::new(parser, &SETUP.theme_set.themes[theme_name])
}

pub fn has_syntax_theme(name: &str) -> error::Result<bool> {
    return Ok(SETUP.theme_set.themes.contains_key(name));
}

pub fn list_syntax_themes<'a>() -> Vec<&'a String> {
    return SETUP.theme_set.themes.keys().collect::<Vec<_>>();
}

pub fn list_syntaxes<'a>() -> Vec<String> {
    fn definition_to_string(sd: &SyntaxDefinition) -> String {
        let extensions = sd.file_extensions.iter().join(&", ".to_owned());
        format!("{} [{}]", sd.name, extensions)
    }

    let mut syntaxes = SETUP
        .syntax_set
        .syntaxes()
        .iter()
        .map(definition_to_string)
        .collect::<Vec<_>>();

    // sort alphabetically with insensitive ascii case
    syntaxes.sort_by_key(|a| (a as &str).to_ascii_lowercase());

    return syntaxes;
}

#[cfg(test)]
mod test {

    use std::default::Default;
    use syntax_highlight::initialize_codeblock;
    use liquid::{self, Renderable, LiquidOptions, Context};

    const CODE_BLOCK: &'static str = "mod test {
        fn hello(arg: int) -> bool {
            \
                                      true
        }
    }
    ";

    const RENDERED: &'static str = "<pre style=\"background-color:#2b303b;\">\n<span \
         style=\"color:#b48ead;\">mod</span><span style=\"color:#c0c5ce;\"> </span><span \
         style=\"color:#c0c5ce;\">test</span><span style=\"color:#c0c5ce;\"> </span><span \
         style=\"color:#c0c5ce;\">{</span>\n<span style=\"color:#c0c5ce;\">        </span><span \
         style=\"color:#b48ead;\">fn</span><span style=\"color:#c0c5ce;\"> </span><span \
         style=\"color:#8fa1b3;\">hello</span><span style=\"color:#c0c5ce;\">(</span><span \
         style=\"color:#bf616a;\">arg</span><span style=\"color:#c0c5ce;\">:</span><span \
         style=\"color:#c0c5ce;\"> int</span><span style=\"color:#c0c5ce;\">)</span><span \
         style=\"color:#c0c5ce;\"> </span><span style=\"color:#c0c5ce;\">-&gt;</span><span \
         style=\"color:#c0c5ce;\"> </span><span style=\"color:#b48ead;\">bool</span><span \
         style=\"color:#c0c5ce;\"> </span><span style=\"color:#c0c5ce;\">{</span>\n<span \
         style=\"color:#c0c5ce;\">            </span><span \
         style=\"color:#d08770;\">true</span>\n<span style=\"color:#c0c5ce;\">        \
         </span><span style=\"color:#c0c5ce;\">}</span>\n<span style=\"color:#c0c5ce;\">    \
         </span><span style=\"color:#c0c5ce;\">}</span>\n<span style=\"color:#c0c5ce;\">    \
         </span>\n</pre>\n";
    #[test]
    fn test_codeblock_renders_rust() {
        let mut options: LiquidOptions = Default::default();
        options
            .blocks
            .insert("codeblock".to_string(),
                    Box::new(|_, args, tokens, _| {
                                 initialize_codeblock(args, tokens, "base16-ocean.dark")
                             }));
        let template = liquid::parse(&format!("{{% codeblock rust %}}{}{{% endcodeblock %}}",
                                              CODE_BLOCK),
                                     options)
                .unwrap();
        let mut data = Context::new();
        let output = template.render(&mut data);
        assert_eq!(output.unwrap(), Some(RENDERED.to_string()));
    }

}