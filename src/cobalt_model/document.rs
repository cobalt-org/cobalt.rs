use std::fmt;

use regex;

use error::*;
use super::frontmatter;

#[derive(Debug, Eq, PartialEq, Default, Clone)]
pub struct DocumentBuilder<T: frontmatter::Front> {
    front: T,
    content: String,
}

impl<T: frontmatter::Front> DocumentBuilder<T> {
    pub fn new(front: T, content: String) -> Self {
        Self { front, content }
    }

    pub fn parts(self) -> (T, String) {
        let Self { front, content } = self;
        (front, content)
    }

    pub fn parse(content: &str) -> Result<Self> {
        let (front, content) = split_document(content)?;
        let front = front
            .map(|s| T::parse(s))
            .map_or(Ok(None), |r| r.map(Some))?
            .unwrap_or_else(T::default);
        let content = content.to_owned();
        Ok(Self { front, content })
    }
}

impl<T: frontmatter::Front> fmt::Display for DocumentBuilder<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let front = self.front.to_string().map_err(|_| fmt::Error)?;
        if front.trim().is_empty() {
            write!(f, "{}", self.content)
        } else {
            write!(f, "{}\n---\n{}", front, self.content)
        }
    }
}

fn split_document(content: &str) -> Result<(Option<&str>, &str)> {
    lazy_static!{
        static ref FRONT_MATTER_DIVIDE: regex::Regex = regex::Regex::new(r"---\s*\r?\n").unwrap();
    }

    if FRONT_MATTER_DIVIDE.is_match(content) {
        let mut splits = FRONT_MATTER_DIVIDE.splitn(content, 2);

        // above the split are the attributes
        let front_split = splits.next().unwrap_or("");

        // everything below the split becomes the new content
        let content_split = splits.next().unwrap_or("");

        if front_split.is_empty() {
            Ok((None, content_split))
        } else {
            Ok((Some(front_split), content_split))
        }
    } else {
        Ok((None, content))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn split_document_empty() {
        let input = "";
        let (cobalt_model, content) = split_document(input).unwrap();
        assert!(cobalt_model.is_none());
        assert_eq!(content, "");
    }

    #[test]
    fn split_document_no_front_matter() {
        let input = "Body";
        let (cobalt_model, content) = split_document(input).unwrap();
        assert!(cobalt_model.is_none());
        assert_eq!(content, "Body");
    }

    #[test]
    fn split_document_empty_front_matter() {
        let input = "---\nBody";
        let (cobalt_model, content) = split_document(input).unwrap();
        assert!(cobalt_model.is_none());
        assert_eq!(content, "Body");
    }

    #[test]
    fn split_document_empty_body() {
        let input = "cobalt_model---\n";
        let (cobalt_model, content) = split_document(input).unwrap();
        assert_eq!(cobalt_model.unwrap(), "cobalt_model");
        assert_eq!(content, "");
    }

    #[test]
    fn document_format_empty_has_no_front() {
        let doc = DocumentBuilder::<frontmatter::FrontmatterBuilder>::default();
        let doc = doc.to_string();
        assert_eq!(doc, "");
    }
}
