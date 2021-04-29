use std::io::Write;

use liquid_core::error::ResultLiquidReplaceExt;
use liquid_core::parser::TryMatchToken;
use liquid_core::Language;
use liquid_core::TagBlock;
use liquid_core::TagTokenIter;
use liquid_core::ValueView;
use liquid_core::{Renderable, Runtime};
use pulldown_cmark as cmark;

use crate::error;

pub fn has_syntax_theme(_name: &str) -> error::Result<bool> {
    failure::bail!("Themes are unsupported in this build.");
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
    fn render_to(
        &self,
        writer: &mut dyn Write,
        _context: &dyn Runtime,
    ) -> Result<(), liquid_core::Error> {
        if let Some(ref lang) = self.lang {
            write!(
                writer,
                "<pre><code class=\"language-{}\">{}</code></pre>",
                lang, self.code
            )
            .replace("Failed to render")?;
        } else {
            write!(writer, "<pre><code>{}</code></pre>", self.code).replace("Failed to render")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct CodeBlockParser {}

impl CodeBlockParser {
    pub fn new(_syntax_theme: String) -> Self {
        Self {}
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
        let content = html_escape(&content);
        tokens.assert_empty();

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
        let highlight: Box<dyn liquid_core::ParseBlock> =
            Box::new(CodeBlockParser::new("base16-ocean.dark".to_owned()));
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
