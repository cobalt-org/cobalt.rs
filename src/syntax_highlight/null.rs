use liquid;
use liquid::Token::{self, Identifier};
use liquid::lexer::Element::{self, Expression, Tag, Raw};

use pulldown_cmark as cmark;

use error;

pub fn has_syntax_theme(_name: &str) -> error::Result<bool> {
    bail!("Themes are unsupported in this build.");
}

pub fn list_syntax_themes<'a>() -> Vec<&'a String> {
    vec![]
}

pub fn list_syntaxes() -> Vec<String> {
    vec![]
}

// The code is taken from Liquid which was adapted from
// https://github.com/rust-lang/rust/blob/master/src/librustdoc/html/escape.rs
// Retrieved 2016-11-19.
fn html_escape(input: &str) -> String {
    let mut result = String::new();
    let mut last = 0;
    let mut skip = 0;
    for (i, c) in input.chars().enumerate() {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        let c: char = c;
        match c {
            '<' | '>' | '\'' | '"' | '&' => {
                result.push_str(&input[last..i]);
                last = i + 1;
                let escaped = match c {
                    '<' => "&lt;",
                    '>' => "&gt;",
                    '\'' => "&#39;",
                    '"' => "&quot;",
                    '&' => "&amp;",
                    _ => unreachable!(),
                };
                result.push_str(escaped);
            }
            _ => {}
        }
    }
    if last < input.len() {
        result.push_str(&input[last..]);
    }
    result
}

struct CodeBlock {
    lang: Option<String>,
    code: String,
}

impl liquid::Renderable for CodeBlock {
    fn render(&self, _: &mut liquid::Context) -> Result<Option<String>, liquid::Error> {
        if let Some(ref lang) = self.lang {
            Ok(Some(format!("<pre><code class=\"language-{}\">{}</code></pre>",
                            lang,
                            self.code)))
        } else {
            Ok(Some(format!("<pre><code>{}</code></pre>", self.code)))
        }
    }
}

pub fn initialize_codeblock(arguments: &[Token],
                            tokens: &[Element],
                            _theme_name: &str)
                            -> Result<Box<liquid::Renderable>, liquid::Error> {

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

    let content = html_escape(&content);

    Ok(Box::new(CodeBlock {
                    lang: lang,
                    code: content,
                }))
}

pub type DecoratedParser<'a> = cmark::Parser<'a>;

pub fn decorate_markdown<'a>(parser: cmark::Parser<'a>, _theme_name: &str) -> DecoratedParser<'a> {
    parser
}

#[cfg(test)]
mod test {

    use std::default::Default;
    use liquid::{self, Renderable, LiquidOptions, Context};

    use super::*;

    const CODE_BLOCK: &'static str = "mod test {
        fn hello(arg: int) -> bool {
            \
                                      true
        }
    }
";

    const CODEBLOCK_RENDERED: &'static str = r#"<pre><code class="language-rust">mod test {
        fn hello(arg: int) -&gt; bool {
            true
        }
    }
</code></pre>"#;

    #[test]
    fn codeblock_renders_rust() {
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
        assert_eq!(output.unwrap(), Some(CODEBLOCK_RENDERED.to_string()));
    }

    const MARKDOWN_RENDERED: &'static str = r#"<pre><code class="language-rust">mod test {
        fn hello(arg: int) -&gt; bool {
            true
        }
    }

</code></pre>
"#;

    #[test]
    fn decorate_markdown_renders_rust() {
        let html = format!("```rust
{}
```",
                           CODE_BLOCK);

        let mut buf = String::new();
        let parser = cmark::Parser::new(&html);
        cmark::html::push_html(&mut buf, decorate_markdown(parser, "base16-ocean.dark"));
        assert_eq!(buf, MARKDOWN_RENDERED);
    }
}
