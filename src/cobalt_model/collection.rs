use std::path;

use liquid;

use error::*;
use super::FrontmatterBuilder;
use super::files;
use super::slug;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SortOrder {
    None,
    Asc,
    Desc,
}

impl Default for SortOrder {
    fn default() -> SortOrder {
        SortOrder::Desc
    }
}

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
        } = self;

        let title = title.ok_or("Collection is missing a `title`")?;
        let slug = slug.unwrap_or_else(|| slug::slugify(&title));

        let source = source.ok_or_else(|| "No asset source provided")?;

        let dir = dir.unwrap_or_else(|| slug.clone());
        let pages = Self::build_files(&source, &dir, &template_extensions, &ignore)?;

        let drafts_dir = if include_drafts { drafts_dir } else { None };
        let drafts = drafts_dir
            .map(|dir| Self::build_files(&source, &dir, &template_extensions, &ignore))
            .map_or(Ok(None), |r| r.map(Some))?;

        let mut attributes: liquid::Object = vec![
            ("title".to_owned(), liquid::Value::scalar(&title)),
            ("slug".to_owned(), liquid::Value::scalar(&slug)),
            (
                "description".to_owned(),
                liquid::Value::scalar(description.clone().unwrap_or_else(|| "".to_owned())),
            ),
        ].into_iter()
            .collect();
        if let Some(ref rss) = rss {
            attributes.insert("rss".to_owned(), liquid::Value::scalar(rss));
        }
        if let Some(ref jsonfeed) = jsonfeed {
            attributes.insert("jsonfeed".to_owned(), liquid::Value::scalar(jsonfeed));
        }

        let default = default.set_collection(slug.clone());

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
            bail!("Collection dir {} must be a relative path", dir)
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
    pub default: FrontmatterBuilder,
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
