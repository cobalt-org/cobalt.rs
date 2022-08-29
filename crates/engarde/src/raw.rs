use std::path::Path;

/// No highlighting, defer to frontend
#[derive(Default, Debug, Clone)]
#[non_exhaustive]
pub struct Raw {}

impl Raw {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load_custom_syntaxes(&mut self, _syntaxes_path: &Path) {}

    pub fn has_theme(&self, _name: &str) -> bool {
        false
    }

    pub fn themes(&self) -> impl Iterator<Item = String> + '_ {
        vec![].into_iter()
    }

    pub fn syntaxes(&self) -> impl Iterator<Item = String> + '_ {
        vec![].into_iter()
    }

    pub fn format(&self, code: &str, lang: Option<&str>, _theme: Option<&str>) -> String {
        let code = html_escape(code);
        if let Some(ref lang) = lang {
            format!(
                "<pre><code class=\"language-{}\">{}</code></pre>\n",
                lang, code
            )
        } else {
            format!("<pre><code>{}</code></pre>\n", code)
        }
    }
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

#[cfg(test)]
mod test {
    use super::*;

    const CODEBLOCK: &str = "mod test {
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
        let syntax = Raw::new();
        let output = syntax.format(CODEBLOCK, Some("rust"), Some(""));
        assert_eq!(output, CODEBLOCK_RENDERED.to_string());
    }
}
