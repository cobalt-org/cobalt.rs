
extern crate syntect;
extern crate liquid;
extern crate pulldown_cmark as cmark;

use liquid::Renderable;
use liquid::Context;
use liquid::LiquidOptions;
use liquid::Token::{self, Identifier};
use liquid::lexer::Element::{self, Expression, Tag, Raw};
use liquid::Error;

use syntect::parsing::{SyntaxSet, SyntaxDefinition};
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_snippet_for_string;

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
}

impl Renderable for CodeBlock {
    fn render(&self, _: &mut Context) -> Result<Option<String>, Error> {
        // FIXME: do this setup only once.

        let syntax = match self.lang {
                Some(ref lang) => SETUP.syntax_set.find_syntax_by_token(lang),
                _ => None,
            }
            .unwrap_or_else(|| SETUP.syntax_set.find_syntax_plain_text());

        // FIXME: allow for theming options?
        Ok(Some(highlighted_snippet_for_string(&self.code,
                                               syntax,
                                               &SETUP.theme_set.themes["base16-ocean.dark"])))
    }
}


pub struct DecoratedParser<'a> {
    parser: Parser<'a>,
    cur_syntax: Option<SyntaxDefinition>,
}

impl<'a> DecoratedParser<'a> {
    pub fn new(parser: Parser<'a>) -> Self {
        DecoratedParser {
            parser: parser,
            cur_syntax: None,
        }
    }
}


impl<'a> Iterator for DecoratedParser<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Event<'a>> {
        match self.parser.next() {
            Some(item) => {
                if let Text(text) = item {
                    if let Some(ref syntax) = self.cur_syntax {
                        Some(Html(Owned(
                            highlighted_snippet_for_string(&text,
        syntax,
        &SETUP.theme_set.themes["base16-ocean.dark"]))))
                    } else {
                        Some(Text(text))
                    }
                } else {
                    if let Start(cmarkTag::CodeBlock(ref info)) = item {
                        // set local highlighter, if found
                        self.cur_syntax = Some(info.clone()
                            .split(' ')
                            .next()
                            .and_then(|lang| SETUP.syntax_set.find_syntax_by_token(lang))
                            .unwrap_or_else(|| SETUP.syntax_set.find_syntax_plain_text())
                            .clone());
                    }
                    if let End(cmarkTag::CodeBlock(_)) = item {
                        // reset
                        self.cur_syntax = None
                    }

                    Some(item)
                }
            }
            None => None,
        }
    }
}

pub fn initialize_codeblock(_: &str,
                            arguments: &[Token],
                            tokens: Vec<Element>,
                            _: &LiquidOptions)
                            -> Result<Box<Renderable>, Error> {

    let content = tokens.iter().fold("".to_owned(), |a, b| {
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

    // FIXME: add language declarion support
    Ok(Box::new(CodeBlock {
        code: content,
        lang: lang,
    }))
}


pub fn decorate_markdown<'a>(parser: Parser<'a>) -> DecoratedParser {
    DecoratedParser::new(parser)
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

    const RENDERED: &'static str =
        "<pre style=\"background-color:#2b303b;\">\n<span \
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
        options.blocks.insert("codeblock".to_string(), Box::new(initialize_codeblock));
        let template = liquid::parse(&format!("{{% codeblock rust %}}{}{{% endcodeblock %}}",
                                              CODE_BLOCK),
                                     options)
            .unwrap();
        let mut data = Context::new();
        let output = template.render(&mut data);
        assert_eq!(output.unwrap(), Some(RENDERED.to_string()));
    }

}
