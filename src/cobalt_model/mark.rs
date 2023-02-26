use pulldown_cmark as cmark;
use serde::Serialize;

use crate::error::*;
use crate::syntax_highlight::decorate_markdown;

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MarkdownBuilder {
    pub custom_executable: Option<String>,
    pub custom_executable_args: Vec<String>,
    pub theme: Option<liquid::model::KString>,
    #[serde(skip)]
    pub syntax: std::sync::Arc<crate::SyntaxHighlight>,
}

impl MarkdownBuilder {
    pub fn build(self) -> Box<dyn AbstractMarkdown> {
        if let Some(cmd) = self.custom_executable {
            Box::new(CustomMarkdown {
                cmd,
                args: self.custom_executable_args,
            })
        } else {
            Box::new(Markdown {
                theme: self.theme,
                syntax: self.syntax,
            })
        }
    }
}

pub trait AbstractMarkdown {
    fn parse(&self, content: &str) -> Result<String>;
}

#[derive(Debug, Clone)]
pub struct Markdown {
    theme: Option<liquid::model::KString>,
    syntax: std::sync::Arc<crate::SyntaxHighlight>,
}

impl AbstractMarkdown for Markdown {
    fn parse(&self, content: &str) -> Result<String> {
        let mut buf = String::new();
        let options = cmark::Options::ENABLE_FOOTNOTES
            | cmark::Options::ENABLE_TABLES
            | cmark::Options::ENABLE_STRIKETHROUGH
            | cmark::Options::ENABLE_TASKLISTS;
        let parser = cmark::Parser::new_ext(content, options);
        cmark::html::push_html(
            &mut buf,
            decorate_markdown(parser, self.syntax.clone(), self.theme.as_deref())?,
        );
        Ok(buf)
    }
}

#[derive(Debug, Clone)]
pub struct CustomMarkdown {
    cmd: String,
    args: Vec<String>,
}

impl AbstractMarkdown for CustomMarkdown {
    fn parse(&self, content: &str) -> Result<String> {
        use std::io::{Read, Write};
        use std::process::{Command, Stdio};
        let mut child = Command::new(&self.cmd)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(content.as_bytes()).unwrap();
        std::mem::drop(stdin);

        let stdout = child.stdout.as_mut().unwrap();
        let mut html = String::new();
        stdout.read_to_string(&mut html).unwrap();

        let stderr = child.stderr.as_mut().unwrap();
        let mut errlog = String::new();
        stderr.read_to_string(&mut errlog).unwrap();

        if !child.wait().unwrap().success() {
            failure::bail!("Custom markdown processor exited with error:\n{}", &errlog);
        }

        Ok(html)
    }
}
