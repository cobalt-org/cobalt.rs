use std::fmt;

use regex;

use error::*;
use super::frontmatter;
use cobalt_model;

#[derive(Debug, Eq, PartialEq, Default, Clone)]
pub struct DocumentBuilder {
    front: frontmatter::FrontmatterBuilder,
    content: String,
}

impl DocumentBuilder {
    pub fn new(front: frontmatter::FrontmatterBuilder, content: String) -> Self {
        Self { front, content }
    }

    pub fn parts(self) -> (frontmatter::FrontmatterBuilder, String) {
        let Self { front, content } = self;
        (front, content)
    }

    pub fn parse(content: &str) -> Result<Self> {
        let (front, content) = split_document(content)?;
        let front = front
            .map(|s| frontmatter::FrontmatterBuilder::parse(s))
            .map_or(Ok(None), |r| r.map(Some))?
            .unwrap_or_else(Default::default);
        let content = content.to_owned();
        Ok(Self { front, content })
    }
}

impl fmt::Display for DocumentBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "---\n{}\n---\n{}", self.front, self.content)
    }
}

impl From<DocumentBuilder> for cobalt_model::DocumentBuilder<cobalt_model::FrontmatterBuilder> {
    fn from(doc: DocumentBuilder) -> Self {
        let (front, content) = doc.parts();
        cobalt_model::DocumentBuilder::new(front.into(), content)
    }
}

fn split_document(content: &str) -> Result<(Option<&str>, &str)> {
    lazy_static! {
        static ref FRONT_MATTER_DIVIDE: regex::Regex = regex::Regex::new(r"---\s*\r?\n").unwrap();
    }

    if FRONT_MATTER_DIVIDE.is_match(content) {
        let mut splits = FRONT_MATTER_DIVIDE.splitn(content, 3);
        let first = splits.next().unwrap_or("");
        let second = splits.next().unwrap_or("");
        let third = splits.next().unwrap_or("");

        if !first.is_empty() {
            bail!("Invalid leading text in frontmatter: {:?}", first);
        }
        if second.is_empty() {
            Ok((None, third))
        } else {
            Ok((Some(second), third))
        }
    } else {
        Ok((None, content))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn split_document_empty_document() {
        let fixture = "";

        let (front, content) = split_document(fixture).unwrap();
        assert!(front.is_none());
        assert_eq!(content, "");
    }

    #[test]
    fn split_document_empty_sections() {
        let fixture = "---\n---\n";

        let (front, content) = split_document(fixture).unwrap();
        assert!(front.is_none());
        assert_eq!(content, "");
    }

    #[test]
    fn split_document_content_only() {
        let fixture = "Content\n";

        let (front, content) = split_document(fixture).unwrap();
        assert!(front.is_none());
        assert_eq!(content, "Content\n");
    }

    #[test]
    fn split_document_empty_front() {
        let fixture = "---\n---\nContent\n";

        let (front, content) = split_document(fixture).unwrap();
        assert!(front.is_none());
        assert_eq!(content, "Content\n");
    }

    #[test]
    fn split_document_empty_content() {
        let fixture = "---\ntitle: test_post\n---\n";

        let (front, content) = split_document(fixture).unwrap();
        assert_eq!(front.unwrap(), "title: test_post\n");
        assert_eq!(content, "");
    }

    #[test]
    fn split_document_all_sections() {
        let fixture = "---\ntitle: test_post\n---\nContent\n";

        let (front, content) = split_document(fixture).unwrap();
        assert_eq!(front.unwrap(), "title: test_post\n");
        assert_eq!(content, "Content\n");
    }

    #[test]
    fn document_parse() {
        let fixture = format!("---\n{}---\nthe content\n", FIXTURE_MINIMAL);

        let _doc = DocumentBuilder::parse(&fixture).unwrap();
        // TODO(epage): verify content
    }

    static FIXTURE_MINIMAL: &str = r#"title: test_post"#;

    #[test]
    fn document_into() {
        let fixture = format!("---\n{}---\nthe content\n", FIXTURE_MINIMAL);

        let doc = DocumentBuilder::parse(&fixture).unwrap();
        let doc: cobalt_model::DocumentBuilder<cobalt_model::FrontmatterBuilder> = doc.into();
        let (_front, content) = doc.parts();

        assert_eq!(content, "the content\n");
    }
}
