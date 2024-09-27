use std::io::Write;

use crate::error;
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

#[cfg(not(feature = "syntax-highlight"))]
pub use engarde::Raw as SyntaxHighlight;
#[cfg(feature = "syntax-highlight")]
pub use engarde::Syntax as SyntaxHighlight;

#[cfg(feature = "syntax-highlight")]
fn has_syntax_theme(syntax: &SyntaxHighlight, name: &str) -> error::Result<bool> {
    Ok(syntax.has_theme(name))
}

#[cfg(not(feature = "syntax-highlight"))]
fn has_syntax_theme(syntax: &SyntaxHighlight, name: &str) -> error::Result<bool> {
    anyhow::bail!("Themes are unsupported in this build.");
}

fn verify_theme(syntax: &SyntaxHighlight, theme: Option<&str>) -> error::Result<()> {
    if let Some(theme) = &theme {
        match has_syntax_theme(syntax, theme) {
            Ok(true) => {}
            Ok(false) => anyhow::bail!("Syntax theme '{}' is unsupported", theme),
            Err(err) => {
                log::warn!("Syntax theme named '{}' ignored. Reason: {}", theme, err);
            }
        };
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct CodeBlock {
    syntax: std::sync::Arc<SyntaxHighlight>,
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
            self.syntax
                .format(&self.code, self.lang.as_deref(), self.theme.as_deref())
        )
        .replace("Failed to render")?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CodeBlockParser {
    syntax: std::sync::Arc<SyntaxHighlight>,
    syntax_theme: Option<liquid::model::KString>,
}

impl CodeBlockParser {
    pub(crate) fn new(
        syntax: std::sync::Arc<SyntaxHighlight>,
        theme: Option<liquid::model::KString>,
    ) -> error::Result<Self> {
        verify_theme(&syntax, theme.as_deref())?;
        Ok(Self {
            syntax,
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
            syntax: self.syntax.clone(),
            code: content,
            lang,
            theme: self.syntax_theme.clone(),
        }))
    }
}

pub(crate) struct DecoratedParser<'a> {
    parser: cmark::Parser<'a>,
    syntax: std::sync::Arc<SyntaxHighlight>,
    theme: Option<&'a str>,
    lang: Option<String>,
    code: Option<Vec<pulldown_cmark::CowStr<'a>>>,
}

impl<'a> DecoratedParser<'a> {
    pub(crate) fn new(
        parser: cmark::Parser<'a>,
        syntax: std::sync::Arc<SyntaxHighlight>,
        theme: Option<&'a str>,
    ) -> error::Result<Self> {
        verify_theme(&syntax, theme)?;
        Ok(DecoratedParser {
            parser,
            syntax,
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
            Some(End(cmark::TagEnd::CodeBlock)) => {
                let html = if let Some(code) = self.code.as_deref() {
                    let code = code.iter().join("\n");
                    self.syntax.format(&code, self.lang.as_deref(), self.theme)
                } else {
                    self.syntax.format("", self.lang.as_deref(), self.theme)
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

pub(crate) fn decorate_markdown<'a>(
    parser: cmark::Parser<'a>,
    syntax: std::sync::Arc<SyntaxHighlight>,
    theme_name: Option<&'a str>,
) -> error::Result<DecoratedParser<'a>> {
    DecoratedParser::new(parser, syntax, theme_name)
}

#[cfg(test)]
#[cfg(feature = "syntax-highlight")]
mod test_syntsx {
    use super::*;

    use snapbox::assert_data_eq;
    use snapbox::prelude::*;
    use snapbox::str;

    const CODE_BLOCK: &str = "mod test {
        fn hello(arg: int) -> bool {
            \
                                      true
        }
    }
    ";

    #[test]
    fn highlight_block_renders_rust() {
        let syntax = std::sync::Arc::new(SyntaxHighlight::new());
        let highlight: Box<dyn liquid_core::ParseBlock> =
            Box::new(CodeBlockParser::new(syntax, Some("base16-ocean.dark".into())).unwrap());
        let parser = liquid::ParserBuilder::new()
            .block(highlight)
            .build()
            .unwrap();
        let template = parser
            .parse(&format!(
                "{{% highlight rust %}}{CODE_BLOCK}{{% endhighlight %}}"
            ))
            .unwrap();
        let output = template.render(&liquid::Object::new());
        let expected = str![[r#"
<pre style="background-color:#2b303b;">
<code><span style="color:#b48ead;">mod </span><span style="color:#c0c5ce;">test {
</span><span style="color:#c0c5ce;">        </span><span style="color:#b48ead;">fn </span><span style="color:#8fa1b3;">hello</span><span style="color:#c0c5ce;">(</span><span style="color:#bf616a;">arg</span><span style="color:#c0c5ce;">: int) -&gt; </span><span style="color:#b48ead;">bool </span><span style="color:#c0c5ce;">{
</span><span style="color:#c0c5ce;">            </span><span style="color:#d08770;">true
</span><span style="color:#c0c5ce;">        }
</span><span style="color:#c0c5ce;">    }
</span><span style="color:#c0c5ce;">    </span></code></pre>

"#]];

        assert_data_eq!(output.unwrap(), expected.raw());
    }

    #[test]
    fn markdown_renders_rust() {
        let html = format!(
            "```rust
{CODE_BLOCK}
```"
        );

        let mut buf = String::new();
        let parser = cmark::Parser::new(&html);
        let syntax = std::sync::Arc::new(SyntaxHighlight::new());
        cmark::html::push_html(
            &mut buf,
            decorate_markdown(parser, syntax, Some("base16-ocean.dark")).unwrap(),
        );
        let expected = str![[r#"
<pre style="background-color:#2b303b;">
<code><span style="color:#b48ead;">mod </span><span style="color:#c0c5ce;">test {
</span><span style="color:#c0c5ce;">        </span><span style="color:#b48ead;">fn </span><span style="color:#8fa1b3;">hello</span><span style="color:#c0c5ce;">(</span><span style="color:#bf616a;">arg</span><span style="color:#c0c5ce;">: int) -&gt; </span><span style="color:#b48ead;">bool </span><span style="color:#c0c5ce;">{
</span><span style="color:#c0c5ce;">            </span><span style="color:#d08770;">true
</span><span style="color:#c0c5ce;">        }
</span><span style="color:#c0c5ce;">    }
</span><span style="color:#c0c5ce;">    
</span></code></pre>

"#]];

        assert_data_eq!(&buf, expected.raw());
    }
}

#[cfg(test)]
#[cfg(not(feature = "syntax-highlight"))]
mod test_raw {
    use super::*;

    use snapbox::assert_data_eq;
    use snapbox::prelude::*;
    use snapbox::str;

    const CODE_BLOCK: &str = "mod test {
        fn hello(arg: int) -> bool {
            \
                                      true
        }
    }
";

    #[test]
    fn codeblock_renders_rust() {
        let syntax = std::sync::Arc::new(SyntaxHighlight::new());
        let highlight: Box<dyn liquid_core::ParseBlock> =
            Box::new(CodeBlockParser::new(syntax, Some("base16-ocean.dark".into())).unwrap());
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
        let expected = str![[r#"
<pre><code class="language-rust">mod test {
        fn hello(arg: int) -&gt; bool {
            true
        }
    }
</code></pre>

"#]];

        assert_data_eq!(output.unwrap(), expected.raw());
    }

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
        let syntax = std::sync::Arc::new(SyntaxHighlight::new());
        cmark::html::push_html(
            &mut buf,
            decorate_markdown(parser, syntax, Some("base16-ocean.dark")).unwrap(),
        );
        let expected = str![[r#"
<pre><code class="language-rust">mod test {
        fn hello(arg: int) -&gt; bool {
            true
        }
    }

</code></pre>

"#]];

        assert_data_eq!(&buf, expected.raw());
    }
}
