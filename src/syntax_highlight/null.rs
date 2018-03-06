use liquid;
use liquid::interpreter::{Context, Renderable};
use liquid::compiler::LiquidOptions;
use liquid::compiler::Token::{self, Identifier};
use liquid::compiler::Element::{self, Expression, Raw, Tag};
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

#[derive(Clone, Debug)]
struct CodeBlock {
    lang: Option<String>,
    code: String,
}

impl Renderable for CodeBlock {
    fn render(&self, _: &mut Context) -> Result<Option<String>, liquid::Error> {
        if let Some(ref lang) = self.lang {
            Ok(Some(format!(
                "<pre><code class=\"language-{}\">{}</code></pre>",
                lang, self.code
            )))
        } else {
            Ok(Some(format!("<pre><code>{}</code></pre>", self.code)))
        }
    }
}

#[derive(Clone, Debug)]
pub struct CodeBlockParser {}

impl CodeBlockParser {
    pub fn new(_syntax_theme: String) -> Self {
        Self {}
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
            }.to_owned() + &a
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
}

pub type DecoratedParser<'a> = cmark::Parser<'a>;

pub fn decorate_markdown<'a>(parser: cmark::Parser<'a>, _theme_name: &str) -> DecoratedParser<'a> {
    parser
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

    const CODEBLOCK_RENDERED: &str = r#"<pre><code class="language-rust">mod test {
        fn hello(arg: int) -&gt; bool {
            true
        }
    }
</code></pre>"#;

    #[test]
    fn codeblock_renders_rust() {
        let highlight: Box<liquid::compiler::ParseBlock> =
            Box::new(CodeBlockParser::new("base16-ocean.dark".to_owned()));
        let parser = liquid::ParserBuilder::new()
            .block("highlight", highlight)
            .build();
        let template = parser
            .parse(&format!(
                "{{% highlight rust %}}{}{{% endhighlight %}}",
                CODE_BLOCK
            ))
            .unwrap();
        let output = template.render(&liquid::Object::new());
        assert_eq!(output.unwrap(), CODEBLOCK_RENDERED.to_string());
    }

    const MARKDOWN_RENDERED: &str = r#"<pre><code class="language-rust">mod test {
        fn hello(arg: int) -&gt; bool {
            true
        }
    }

</code></pre>
"#;

    #[test]
    fn decorate_markdown_renders_rust() {
        let html = format!(
            "```rust
{}
```",
            CODE_BLOCK
        );

        let mut buf = String::new();
        let parser = cmark::Parser::new(&html);
        cmark::html::push_html(&mut buf, decorate_markdown(parser, "base16-ocean.dark"));
        assert_eq!(buf, MARKDOWN_RENDERED);
    }
}
