use std::fmt;

pub struct Document {
    attributes: Vec<(String, String)>,
    content: String,
}

impl Document {
    pub fn new(attributes: Vec<(String, String)>, content: String) -> Document {
        Document {
            attributes: attributes,
            content: content,
        }
    }

    pub fn as_html(&self) -> &str {
        "test"
    }
}

impl fmt::Show for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Attributes: {}\nContent: {}", self.attributes, self.content)
    }
}
