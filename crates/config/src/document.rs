use std::fmt;

use regex;

use crate::Frontmatter;
use crate::Result;
use crate::Status;

#[derive(Debug, Eq, PartialEq, Default, Clone)]
pub struct Document {
    front: crate::Frontmatter,
    content: String,
}

impl Document {
    pub fn new(front: Frontmatter, content: String) -> Self {
        Self { front, content }
    }

    pub fn parse(content: &str) -> Result<Self> {
        let (front, content) = split_document(content);
        let front = front
            .map(parse_frontmatter)
            .map_or(Ok(None), |r| r.map(Some))?
            .unwrap_or_else(Frontmatter::default);
        let content = content.to_owned();
        Ok(Self { front, content })
    }

    pub fn into_parts(self) -> (Frontmatter, String) {
        let Self { front, content } = self;
        (front, content)
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let front = self.front.to_string();
        if front.is_empty() {
            write!(f, "{}", self.content)
        } else {
            write!(f, "---\n{}\n---\n{}", front, self.content)
        }
    }
}

fn parse_frontmatter(front: &str) -> Result<Frontmatter> {
    let front: Frontmatter = serde_yaml::from_str(front)
        .map_err(|e| Status::new("Failed to parse frontmatter").with_source(e))?;
    Ok(front)
}

static FRONT_MATTER: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| {
    regex::RegexBuilder::new(r"\A---\s*\r?\n([\s\S]*\n)?---\s*\r?\n(.*)")
        .dot_matches_new_line(true)
        .build()
        .unwrap()
});

fn split_document(content: &str) -> (Option<&str>, &str) {
    if let Some(captures) = FRONT_MATTER.captures(content) {
        let front_split = captures.get(1).map(|m| m.as_str()).unwrap_or_default();
        let content_split = captures.get(2).unwrap().as_str();

        if front_split.is_empty() {
            (None, content_split)
        } else {
            (Some(front_split), content_split)
        }
    } else {
        (None, content)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn split_document_empty() {
        let input = "";
        let (cobalt_model, content) = split_document(input);
        assert!(cobalt_model.is_none());
        assert_eq!(content, "");
    }

    #[test]
    fn split_document_no_front_matter() {
        let input = "Body";
        let (cobalt_model, content) = split_document(input);
        assert!(cobalt_model.is_none());
        assert_eq!(content, "Body");
    }

    #[test]
    fn split_document_empty_front_matter() {
        let input = "---\n---\nBody";
        let (cobalt_model, content) = split_document(input);
        assert!(cobalt_model.is_none());
        assert_eq!(content, "Body");
    }

    #[test]
    fn split_document_empty_body() {
        let input = "---\ncobalt_model\n---\n";
        let (cobalt_model, content) = split_document(input);
        assert_eq!(cobalt_model.unwrap(), "cobalt_model\n");
        assert_eq!(content, "");
    }

    #[test]
    fn split_document_front_matter_and_body() {
        let input = "---\ncobalt_model\n---\nbody";
        let (cobalt_model, content) = split_document(input);
        assert_eq!(cobalt_model.unwrap(), "cobalt_model\n");
        assert_eq!(content, "body");
    }

    #[test]
    fn split_document_no_new_line_after_front_matter() {
        let input = "invalid_front_matter---\nbody";
        let (cobalt_model, content) = split_document(input);
        println!("{:?}", cobalt_model);
        assert!(cobalt_model.is_none());
        assert_eq!(content, input);
    }

    #[test]
    fn split_document_multiline_body() {
        let input = "---\ncobalt_model\n---\nfirst\nsecond";
        let (cobalt_model, content) = split_document(input);
        println!("{:?}", cobalt_model);
        assert_eq!(cobalt_model.unwrap(), "cobalt_model\n");
        assert_eq!(content, "first\nsecond");
    }

    #[test]
    fn display_empty() {
        let front = Frontmatter::empty();
        let doc = Document::new(front, String::new());
        assert_eq!(&doc.to_string(), "");
    }

    #[test]
    fn display_empty_front() {
        let front = Frontmatter::empty();
        let doc = Document::new(front, "body".to_owned());
        assert_eq!(&doc.to_string(), "body");
    }

    #[test]
    fn display_empty_body() {
        let front = Frontmatter {
            slug: Some("foo".to_owned()),
            ..Default::default()
        };
        let doc = Document::new(front, String::new());
        assert_eq!(&doc.to_string(), "---\nslug: foo\n---\n");
    }

    #[test]
    fn display_both() {
        let front = Frontmatter {
            slug: Some("foo".to_owned()),
            ..Default::default()
        };
        let doc = Document::new(front, "body".to_owned());
        assert_eq!(&doc.to_string(), "---\nslug: foo\n---\nbody");
    }
}
