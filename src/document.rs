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
}
