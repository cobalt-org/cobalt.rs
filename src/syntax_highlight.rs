use std::io::Write;

use crate::error;
use itertools::Itertools;
use lazy_static::lazy_static;
use liquid_core::error::ResultLiquidReplaceExt;
use liquid_core::parser::TryMatchToken;
use liquid_core::Language;
use liquid_core::TagBlock;
use liquid_core::TagTokenIter;
use liquid_core::ValueView;
use liquid_core::{Renderable, Runtime};
use pulldown_cmark as cmark;
use pulldown_cmark::Event::{self, End, Html, Start, Text};

#[cfg(not(feature = "syntax-highlight"))]
use engarde::Raw as Highlight;
#[cfg(feature = "syntax-highlight")]
use engarde::Syntax as Highlight;

lazy_static! {
    static ref HIGHLIGHT: Highlight = Highlight::new();
}

#[cfg(feature = "syntax-highlight")]
fn has_syntax_theme(name: &str) -> error::Result<bool> {
    Ok(HIGHLIGHT.has_theme(name))
}

#[cfg(not(feature = "syntax-highlight"))]
fn has_syntax_theme(name: &str) -> error::Result<bool> {
    failure::bail!("Themes are unsupported in this build.");
}

fn verify_theme(theme: Option<&str>) -> error::Result<()> {
    if let Some(theme) = &theme {
        match has_syntax_theme(theme) {
            Ok(true) => {}
            Ok(false) => failure::bail!("Syntax theme '{}' is unsupported", theme),
            Err(err) => {
                log::warn!("Syntax theme named '{}' ignored. Reason: {}", theme, err);
            }
        };
    }
    Ok(())
}

pub fn list_syntax_themes() -> Vec<String> {
    HIGHLIGHT.themes().collect()
}

pub fn list_syntaxes() -> Vec<String> {
    HIGHLIGHT.syntaxes().collect()
}

#[derive(Clone, Debug)]
struct CodeBlock {
    lang: Option<liquid::model::KString>,
    code: String,
    theme: Option<liquid::model::KString>,
}

impl Renderable for CodeBlock {
    fn render_to(
        &self,
        writer: &mut dyn Write,
        _context: &dyn Runtime,
    ) -> Result<(), liquid_core::Error> {
        write!(
            writer,
            "{}",
            HIGHLIGHT.format(&self.code, self.lang.as_deref(), self.theme.as_deref())
        )
        .replace("Failed to render")?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct CodeBlockParser {
    syntax_theme: Option<liquid::model::KString>,
}

impl CodeBlockParser {
    pub fn new(theme: Option<liquid::model::KString>) -> error::Result<Self> {
        verify_theme(theme.as_deref())?;
        Ok(Self {
            syntax_theme: theme,
        })
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
        mut arguments: TagTokenIter<'_>,
        mut tokens: TagBlock<'_, '_>,
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
                    TryMatchToken::Matches(lang) => lang.to_kstr().into_owned(),
                    TryMatchToken::Fails(lang) => liquid::model::KString::from_ref(lang.as_str()),
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
            theme: self.syntax_theme.clone(),
        }))
    }
}

pub struct DecoratedParser<'a> {
    parser: cmark::Parser<'a, 'a>,
    theme: Option<&'a str>,
    lang: Option<String>,
    code: Option<Vec<pulldown_cmark::CowStr<'a>>>,
}

impl<'a> DecoratedParser<'a> {
    pub fn new(parser: cmark::Parser<'a, 'a>, theme: Option<&'a str>) -> error::Result<Self> {
        verify_theme(theme)?;
        Ok(DecoratedParser {
            parser,
            theme,
            lang: None,
            code: None,
        })
    }
}

impl<'a> Iterator for DecoratedParser<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Event<'a>> {
        match self.parser.next() {
            Some(Text(text)) => {
                if let Some(ref mut code) = self.code {
                    code.push(text);
                    Some(Text(pulldown_cmark::CowStr::Borrowed("")))
                } else {
                    Some(Text(text))
                }
            }
            Some(Start(cmark::Tag::CodeBlock(info))) => {
                let tag = match info {
                    pulldown_cmark::CodeBlockKind::Indented => "",
                    pulldown_cmark::CodeBlockKind::Fenced(ref tag) => tag.as_ref(),
                };
                self.lang = tag.split(' ').map(|s| s.to_owned()).next();
                self.code = Some(vec![]);
                Some(Text(pulldown_cmark::CowStr::Borrowed("")))
            }
            Some(End(cmark::Tag::CodeBlock(_))) => {
                let html = if let Some(code) = self.code.as_deref() {
                    let code = code.iter().join("\n");
                    HIGHLIGHT.format(&code, self.lang.as_deref(), self.theme)
                } else {
                    HIGHLIGHT.format("", self.lang.as_deref(), self.theme)
                };
                // reset highlighter
                self.lang = None;
                self.code = None;
                // close the code block
                Some(Html(pulldown_cmark::CowStr::Boxed(html.into_boxed_str())))
            }
            item => item,
        }
    }
}

pub fn decorate_markdown<'a>(
    parser: cmark::Parser<'a, 'a>,
    theme_name: Option<&'a str>,
) -> error::Result<DecoratedParser<'a>> {
    DecoratedParser::new(parser, theme_name)
}

#[cfg(test)]
#[cfg(feature = "syntax-highlight")]
mod test_syntsx {
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
            Box::new(CodeBlockParser::new(Some("base16-ocean.dark".into())).unwrap());
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
        snapbox::assert_eq(CODEBLOCK_RENDERED, &output.unwrap());
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
         </span></pre>\n";

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
        cmark::html::push_html(
            &mut buf,
            decorate_markdown(parser, Some("base16-ocean.dark")).unwrap(),
        );
        snapbox::assert_eq(MARKDOWN_RENDERED, &buf);
    }
}

#[cfg(test)]
#[cfg(not(feature = "syntax-highlight"))]
mod test_raw {
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
</code></pre>
"#;

    #[test]
    fn codeblock_renders_rust() {
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
        cmark::html::push_html(
            &mut buf,
            decorate_markdown(parser, Some("base16-ocean.dark")),
        );
        assert_eq!(buf, MARKDOWN_RENDERED);
    }
}
