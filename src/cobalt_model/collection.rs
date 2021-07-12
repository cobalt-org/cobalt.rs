use cobalt_config::Frontmatter;
use cobalt_config::SortOrder;
use liquid;

use crate::error::*;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Collection {
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub dir: String,
    pub drafts_dir: Option<String>,
    pub order: SortOrder,
    pub rss: Option<String>,
    pub jsonfeed: Option<String>,
    pub publish_date_in_filename: bool,
    pub default: Frontmatter,
}

impl Collection {
    pub fn from_page_config(
        config: cobalt_config::PageCollection,
        site: &cobalt_config::Site,
        common_default: &cobalt_config::Frontmatter,
    ) -> Result<Self> {
        let mut config: cobalt_config::Collection = config.into();
        // Use `site` because the pages are effectively the site
        config.title = Some(site.title.clone().unwrap_or_else(|| "".to_owned()));
        config.description = site.description.clone();
        Self::from_config(config, "pages", false, common_default)
    }

    pub fn from_post_config(
        config: cobalt_config::PostCollection,
        site: &cobalt_config::Site,
        include_drafts: bool,
        common_default: &cobalt_config::Frontmatter,
    ) -> Result<Self> {
        let mut config: cobalt_config::Collection = config.into();
        // Default with `site` for people quickly bootstrapping a blog, the blog and site are
        // effectively equivalent.
        if config.title.is_none() {
            config.title = Some(site.title.clone().unwrap_or_else(|| "".to_owned()));
        }
        if config.description.is_none() {
            config.description = site.description.clone();
        }
        Self::from_config(config, "posts", include_drafts, common_default)
    }

    fn from_config(
        config: cobalt_config::Collection,
        slug: &str,
        include_drafts: bool,
        common_default: &cobalt_config::Frontmatter,
    ) -> Result<Self> {
        let cobalt_config::Collection {
            title,
            description,
            dir,
            drafts_dir,
            order,
            rss,
            jsonfeed,
            default,
            publish_date_in_filename,
        } = config;

        let title = title.ok_or_else(|| failure::err_msg("Collection is missing a `title`"))?;
        let slug = slug.to_owned();

        let dir = dir.map(|p| p.to_string()).unwrap_or_else(|| slug.clone());
        let drafts_dir = if include_drafts {
            drafts_dir.map(|p| p.to_string())
        } else {
            None
        };

        let default = default.merge(common_default).merge(&Frontmatter {
            collection: Some(slug.clone()),
            ..Default::default()
        });

        let new = Collection {
            title,
            slug,
            description,
            dir,
            drafts_dir,
            order,
            rss,
            jsonfeed,
            publish_date_in_filename,
            default,
        };
        Ok(new)
    }

    pub fn attributes(&self) -> liquid::Object {
        let mut attributes: liquid::Object = vec![
            (
                "title".into(),
                liquid::model::Value::scalar(self.title.clone()),
            ),
            (
                "slug".into(),
                liquid::model::Value::scalar(self.slug.clone()),
            ),
            (
                "description".into(),
                liquid::model::Value::scalar(
                    self.description.clone().unwrap_or_else(|| "".to_owned()),
                ),
            ),
        ]
        .into_iter()
        .collect();
        if let Some(rss) = self.rss.as_ref() {
            attributes.insert("rss".into(), liquid::model::Value::scalar(rss.to_owned()));
        }
        if let Some(jsonfeed) = self.jsonfeed.as_ref() {
            attributes.insert(
                "jsonfeed".into(),
                liquid::model::Value::scalar(jsonfeed.to_owned()),
            );
        }
        attributes
    }
}
