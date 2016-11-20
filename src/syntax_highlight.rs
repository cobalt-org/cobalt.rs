
extern crate syntect;
extern crate liquid;

use liquid::Renderable;
use liquid::Context;
use liquid::LiquidOptions;
use liquid::Token;
use liquid::lexer::Element::{self, Expression, Tag, Raw};
use liquid::Error;

use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_snippet_for_string;


struct CodeBlock {
    lang: Option<String>,
    code: String
}


impl Renderable for CodeBlock {
    fn render(&self, context: &mut Context) -> Result<Option<String>, Error> {
        // FIXME: do this setup only once.
        let syn = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();

        let syntax = match self.lang {
            Some(ref lang) => syn.find_syntax_by_name(lang).unwrap_or(syn.find_syntax_plain_text()),
            None => syn.find_syntax_plain_text()
            // None => syn.find_syntax_by_firstline().unwrap_or(syn.find_syntax_plain_text())
        };

        // FIXME: allow for theming options?
        Ok(Some(highlighted_snippet_for_string(&self.code, syntax, &ts.themes["base16-ocean.dark"])))
    }
}


pub fn initialize_codeblock(_tag_name: &str,
                            _arguments: &[Token],
                            tokens: Vec<Element>,
                            options: &LiquidOptions)
                        -> Result<Box<Renderable>, Error> {

    let content = tokens.iter().fold("".to_owned(), |a, b| {
        match *b {
                Expression(_, ref text) |
                Tag(_, ref text) |
                Raw(ref text) => text,
            }
            .to_owned() + &a
    });
       
    // FIXME: add language declarion support
    Ok(Box::new(CodeBlock { code: content, lang: None }))
}


#[cfg(test)]
fn test_codeblock_renders_rust() {

    use std::default::Default;
    use syntax_highlight::initialize_codeblock;
    use liquid::{self, Renderable, LiquidOptions, Context};

    const CODE_BLOCK : &'static str = "mod test {
        fn hello(arg: int) -> bool {
            true
        }
    }
    ";


    let mut options: LiquidOptions = Default::default();
    options.blocks.insert("codeblock".to_string(), Box::new(initialize_codeblock));
    let template = liquid::parse(&format!("{{% codeblock rust %}}{}{{% endcodeblock %}}", CODE_BLOCK), options).unwrap();
    let mut data = Context::new();
    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("LIQUID!".to_string()));
}
