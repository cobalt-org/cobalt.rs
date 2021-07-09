use std::path;

use cobalt_config::Frontmatter;
use cobalt_config::SortOrder;
use liquid;

use super::files;
use super::slug;
use crate::error::*;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct CollectionBuilder {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub source: Option<path::PathBuf>,
    pub dir: Option<String>,
    pub drafts_dir: Option<String>,
    pub include_drafts: bool,
    pub template_extensions: Vec<String>,
    pub ignore: Vec<String>,
    pub order: SortOrder,
    pub rss: Option<String>,
    pub jsonfeed: Option<String>,
    pub base_url: Option<String>,
    pub publish_date_in_filename: bool,
    pub default: Frontmatter,
}

impl CollectionBuilder {
    pub fn from_page_config(
        config: cobalt_config::PageCollection,
        source: &path::Path,
        site: &cobalt_config::Site,
        posts: &cobalt_config::PostCollection,
        common_default: &cobalt_config::Frontmatter,
        ignore: &[String],
        template_extensions: &[String],
    ) -> Self {
        let mut ignore = ignore.to_vec();
        ignore.push(format!("/{}", posts.dir));
        if let Some(ref drafts_dir) = posts.drafts_dir {
            ignore.push(format!("/{}", drafts_dir));
        }
        let mut config: cobalt_config::Collection = config.into();
        // Use `site` because the pages are effectively the site
        config.title = Some(site.title.clone().unwrap_or_else(|| "".to_owned()));
        config.description = site.description.clone();
        Self::from_config(
            config,
            "pages",
            false,
            source,
            site,
            common_default,
            ignore,
            template_extensions,
        )
    }

    pub fn from_post_config(
        config: cobalt_config::PostCollection,
        source: &path::Path,
        site: &cobalt_config::Site,
        include_drafts: bool,
        common_default: &cobalt_config::Frontmatter,
        ignore: &[String],
        template_extensions: &[String],
    ) -> Self {
        let mut config: cobalt_config::Collection = config.into();
        // Default with `site` for people quickly bootstrapping a blog, the blog and site are
        // effectively equivalent.
        if config.title.is_none() {
            config.title = Some(site.title.clone().unwrap_or_else(|| "".to_owned()));
        }
        if config.description.is_none() {
            config.description = site.description.clone();
        }
        Self::from_config(
            config,
            "posts",
            include_drafts,
            source,
            site,
            common_default,
            ignore.to_vec(),
            template_extensions,
        )
    }

    fn from_config(
        config: cobalt_config::Collection,
        slug: &str,
        include_drafts: bool,
        source: &path::Path,
        site: &cobalt_config::Site,
        common_default: &cobalt_config::Frontmatter,
        ignore: Vec<String>,
        template_extensions: &[String],
    ) -> Self {
        let cobalt_config::Collection {
            title,
            description,
            dir,
            drafts_dir,
            order,
            rss,
            jsonfeed,
            publish_date_in_filename,
            default,
        } = config;
        Self {
            title: title,
            slug: Some(slug.to_owned()),
            description: description,
            source: Some(source.to_owned()),
            dir: dir.map(|p| p.to_string()),
            drafts_dir: drafts_dir.map(|p| p.to_string()),
            include_drafts,
            template_extensions: template_extensions.to_vec(),
            ignore,
            order,
            rss,
            jsonfeed,
            base_url: site.base_url.clone(),
            publish_date_in_filename,
            default: default.merge(&common_default),
        }
    }

    pub fn merge_frontmatter(mut self, secondary: &Frontmatter) -> Self {
        self.default = self.default.merge(secondary);
        self
    }

    pub fn build(self) -> Result<Collection> {
        let CollectionBuilder {
            title,
            slug,
            description,
            source,
            dir,
            drafts_dir,
            include_drafts,
            template_extensions,
            ignore,
            order,
            rss,
            jsonfeed,
            base_url,
            default,
            ..
        } = self;

        let title = title.ok_or_else(|| failure::err_msg("Collection is missing a `title`"))?;
        let slug = slug.unwrap_or_else(|| slug::slugify(&title));

        let source = source.ok_or_else(|| failure::err_msg("No asset source provided"))?;

        let dir = dir.unwrap_or_else(|| slug.clone());
        let pages = Self::build_files(&source, &dir, &template_extensions, &ignore)?;

        let drafts_dir = if include_drafts { drafts_dir } else { None };
        let drafts = drafts_dir
            .map(|dir| Self::build_files(&source, &dir, &template_extensions, &ignore))
            .map_or(Ok(None), |r| r.map(Some))?;

        let mut attributes: liquid::Object = vec![
            ("title".into(), liquid::model::Value::scalar(title.clone())),
            ("slug".into(), liquid::model::Value::scalar(slug.clone())),
            (
                "description".into(),
                liquid::model::Value::scalar(description.clone().unwrap_or_else(|| "".to_owned())),
            ),
        ]
        .into_iter()
        .collect();
        if let Some(ref rss) = rss {
            attributes.insert("rss".into(), liquid::model::Value::scalar(rss.to_owned()));
        }
        if let Some(ref jsonfeed) = jsonfeed {
            attributes.insert(
                "jsonfeed".into(),
                liquid::model::Value::scalar(jsonfeed.to_owned()),
            );
        }

        let default = default.merge(&Frontmatter {
            collection: Some(slug.clone()),
            ..Default::default()
        });

        let new = Collection {
            title,
            slug,
            description,
            pages,
            drafts,
            include_drafts,
            order,
            rss,
            jsonfeed,
            base_url,
            default,
            attributes,
        };
        Ok(new)
    }

    fn build_files(
        source: &path::Path,
        dir: &str,
        template_extensions: &[String],
        ignore: &[String],
    ) -> Result<files::Files> {
        if dir.starts_with('/') {
            failure::bail!("Collection dir {} must be a relative path", dir)
        }
        let dir = files::cleanup_path(dir);
        let mut pages = files::FilesBuilder::new(source)?;
        if !dir.is_empty() {
            // In-case `dir` starts with `_`
            pages
                .add_ignore(&format!("!/{}", dir))?
                .add_ignore(&format!("!/{}/**", dir))?
                .add_ignore(&format!("/{}/**/_*", dir))?
                .add_ignore(&format!("/{}/**/_*/**", dir))?;
            pages.limit(path::PathBuf::from(dir))?;
        }
        for line in ignore {
            pages.add_ignore(line.as_str())?;
        }
        for ext in template_extensions {
            pages.add_extension(ext)?;
        }
        pages.build()
    }
}

#[derive(Clone, Debug)]
pub struct Collection {
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub pages: files::Files,
    pub drafts: Option<files::Files>,
    pub include_drafts: bool,
    pub order: SortOrder,
    pub rss: Option<String>,
    pub jsonfeed: Option<String>,
    pub base_url: Option<String>,
    pub default: Frontmatter,
    pub attributes: liquid::Object,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_build_dir_rel() {
        let mut collection = CollectionBuilder::default();
        collection.source = Some(path::PathBuf::from("/"));
        collection.title = Some("title".to_owned());
        collection.dir = Some("rel".to_owned());
        let collection = collection.build().unwrap();
        assert_eq!(collection.pages.subtree(), path::Path::new("/rel"));
    }

    #[test]
    fn test_build_dir_abs() {
        let mut collection = CollectionBuilder::default();
        collection.source = Some(path::PathBuf::from("/"));
        collection.title = Some("title".to_owned());
        collection.dir = Some("/root".to_owned());
        let collection = collection.build();
        assert!(collection.is_err());
    }

    #[test]
    fn test_build_drafts_rel() {
        let mut collection = CollectionBuilder::default();
        collection.source = Some(path::PathBuf::from("/"));
        collection.title = Some("title".to_owned());
        collection.drafts_dir = Some("rel".to_owned());
        collection.include_drafts = true;
        let collection = collection.build().unwrap();
        assert_eq!(
            collection.drafts.unwrap().subtree(),
            path::Path::new("/rel")
        );
    }

    #[test]
    fn test_build_drafts_abs() {
        let mut collection = CollectionBuilder::default();
        collection.source = Some(path::PathBuf::from("/"));
        collection.title = Some("title".to_owned());
        collection.drafts_dir = Some("/root".to_owned());
        collection.include_drafts = true;
        let collection = collection.build();
        assert!(collection.is_err());
    }
}
