use liquid;

use error::*;
use super::FrontmatterBuilder;
use super::slug;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl Default for SortOrder {
    fn default() -> SortOrder {
        SortOrder::Desc
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct CollectionBuilder {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub dir: Option<String>,
    pub drafts_dir: Option<String>,
    pub order: SortOrder,
    pub rss: Option<String>,
    pub jsonfeed: Option<String>,
    pub default: FrontmatterBuilder,
}

impl CollectionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge_frontmatter(mut self, secondary: FrontmatterBuilder) -> Self {
        self.default = self.default.merge(secondary);
        self
    }

    pub fn build(self) -> Result<Collection> {
        let CollectionBuilder {
            title,
            slug,
            description,
            dir,
            drafts_dir,
            order,
            rss,
            jsonfeed,
            default,
        } = self;

        let title = title.ok_or("Collection is missing a `title`")?;
        let slug = slug.unwrap_or_else(|| slug::slugify(&title));
        let dir = dir.unwrap_or_else(|| slug.clone());

        if dir.starts_with('/') {
            bail!("Collection {}: dir {} must be a relative path", title, dir)
        }
        if let Some(ref drafts_dir) = drafts_dir {
            if drafts_dir.starts_with('/') {
                bail!("Collection {}: dir {} must be a relative path",
                      title,
                      drafts_dir)
            }
        }

        let mut attributes: liquid::Object =
            vec![("title".to_owned(), liquid::Value::str(&title)),
                 ("slug".to_owned(), liquid::Value::str(&slug)),
                 ("description".to_owned(),
                  liquid::Value::Str(description.clone().unwrap_or_else(|| "".to_owned())))]
                .into_iter()
                .collect();
        if let Some(ref rss) = rss {
            attributes.insert("rss".to_owned(), liquid::Value::str(rss));
        }
        if let Some(ref jsonfeed) = jsonfeed {
            attributes.insert("jsonfeed".to_owned(), liquid::Value::str(jsonfeed));
        }

        let default = default.set_collection(slug.clone());

        let new = Collection {
            title,
            slug,
            description,
            dir,
            drafts_dir,
            order,
            rss,
            jsonfeed,
            default,
            attributes,
        };
        Ok(new)
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Collection {
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub dir: String,
    pub drafts_dir: Option<String>,
    pub order: SortOrder,
    pub rss: Option<String>,
    pub jsonfeed: Option<String>,
    pub default: FrontmatterBuilder,
    pub attributes: liquid::Object,
}
