use cobalt_model;

use super::FrontmatterBuilder;

pub type DocumentBuilder = cobalt_model::DocumentBuilder<FrontmatterBuilder>;

impl From<DocumentBuilder> for cobalt_model::DocumentBuilder<cobalt_model::FrontmatterBuilder> {
    fn from(legacy: DocumentBuilder) -> Self {
        let (front, content) = legacy.parts();
        let front: cobalt_model::FrontmatterBuilder = front.into();
        Self::new(front, content)
    }
}
