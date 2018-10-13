use std::fmt;

use regex;

use super::frontmatter;
use error::*;

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
            write!(f, "---\n{}\n---\n{}", front, self.content)
        }
    }
}

fn split_document(content: &str) -> Result<(Option<&str>, &str)> {
    lazy_static! {
        static ref FRONT_MATTER_DIVIDE: regex::Regex = regex::Regex::new(r"---\s*\r?\n").unwrap();
        static ref FRONT_MATTER: regex::Regex = regex::Regex::new(r"\A---\s*\r?\n([\s\S]*\n)?---\s*\r?\n").unwrap();
    }

    if FRONT_MATTER.is_match(content) {
        // skip first empty string
        let mut splits = FRONT_MATTER_DIVIDE.splitn(content, 3).skip(1);

        // split between dividers
        let front_split = splits.next().unwrap_or("");

        // split after second divider
        let content_split = splits.next().unwrap_or("");

        if front_split.is_empty() {
            Ok((None, content_split))
        } else {
            Ok((Some(front_split), content_split))
        }
    } else {
        deprecated_split_front_matter(content)
    }
}

fn deprecated_split_front_matter(content: &str) -> Result<(Option<&str>, &str)> {
    lazy_static! {
        static ref FRONT_MATTER_DIVIDE: regex::Regex = regex::Regex::new(r"(\A|\n)---\s*\r?\n").unwrap();
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
    fn split_document_deprecated_empty_front_matter() {
        let input = "---\nBody";
        let (cobalt_model, content) = split_document(input).unwrap();
        assert!(cobalt_model.is_none());
        assert_eq!(content, "Body");
    }

    #[test]
    fn split_document_empty_front_matter() {
        let input = "---\n---\nBody";
        let (cobalt_model, content) = split_document(input).unwrap();
        assert!(cobalt_model.is_none());
        assert_eq!(content, "Body");
    }

    #[test]
    fn split_document_deprecated_empty_body() {
        let input = "cobalt_model\n---\n";
        let (cobalt_model, content) = split_document(input).unwrap();
        assert_eq!(cobalt_model.unwrap(), "cobalt_model");
        assert_eq!(content, "");
    }

    #[test]
    fn split_document_empty_body() {
        let input = "---\ncobalt_model\n---\n";
        let (cobalt_model, content) = split_document(input).unwrap();
        assert_eq!(cobalt_model.unwrap(), "cobalt_model\n");
        assert_eq!(content, "");
    }

    #[test]
    fn split_document_front_matter_and_body() {
        let input = "---\ncobalt_model\n---\nbody";
        let (cobalt_model, content) = split_document(input).unwrap();
        assert_eq!(cobalt_model.unwrap(), "cobalt_model\n");
        assert_eq!(content, "body");
    }

    #[test]
    fn split_document_no_new_line_after_front_matter() {
        let input = "invalid_front_matter---\nbody";
        let (cobalt_model, content) = split_document(input).unwrap();
        assert!(cobalt_model.is_none());
        assert_eq!(content, input);

    }

    #[test]
    fn document_format_empty_has_no_front() {
        let doc = DocumentBuilder::<frontmatter::FrontmatterBuilder>::default();
        let doc = doc.to_string();
        assert_eq!(doc, "");
    }
}
