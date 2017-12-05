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

impl From<cobalt_model::DocumentBuilder<cobalt_model::FrontmatterBuilder>> for DocumentBuilder {
    fn from(modern: cobalt_model::DocumentBuilder<cobalt_model::FrontmatterBuilder>) -> Self {
        let (front, content) = modern.parts();
        let front: FrontmatterBuilder = front.into();
        Self::new(front, content)
    }
}
