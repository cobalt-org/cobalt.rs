use std::collections;
use std::env;
use std::fs;
use std::io::Write;
use std::path;

use cobalt::cobalt_model;
use failure::ResultExt;

use crate::args;
use crate::error::*;

/// Create a document
#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
pub struct InitArgs {
    /// Target directory
    #[clap(default_value = "./", parse(from_os_str))]
    pub directory: path::PathBuf,
}

impl InitArgs {
    pub fn run(&self) -> Result<()> {
        create_new_project(&self.directory)
            .with_context(|_| failure::err_msg("Could not create a new cobalt project"))?;
        info!("Created new project at {}", self.directory.display());

        Ok(())
    }
}

/// Create a document
#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
pub struct NewArgs {
    /// Title of the post
    pub title: Option<String>,

    /// New document's parent directory or file (default: `<CWD>/title.ext`)
    #[clap(short, long, value_name = "DIR_OR_FILE", parse(from_os_str))]
    pub file: Option<path::PathBuf>,

    /// The default file's extension (e.g. `liquid`)
    #[clap(long, value_name = "EXT")]
    pub with_ext: Option<String>,

    /// Open the new document in your configured EDITOR
    #[clap(long)]
    pub edit: bool,

    #[clap(flatten, next_help_heading = "CONFIG")]
    pub config: args::ConfigArgs,
}

impl NewArgs {
    pub fn run(&self) -> Result<()> {
        let mut config = self.config.load_config()?;
        config.include_drafts = true;
        let config = cobalt::cobalt_model::Config::from_config(config)?;

        let title = self.title.as_deref();

        let mut file = env::current_dir().expect("How does this fail?");
        if let Some(rel_file) = self.file.as_deref() {
            file.push(rel_file)
        }

        let ext = self.with_ext.as_deref();

        create_new_document(&config, title, file, ext, self.edit)
            .with_context(|_| failure::format_err!("Could not create document"))?;

        Ok(())
    }
}

/// Rename a document
#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
pub struct RenameArgs {
    /// File to rename
    #[clap(value_name = "FILE", parse(from_os_str))]
    pub src: path::PathBuf,

    /// Title of the post
    pub title: String,

    /// New document's parent directory or file (default: `<CWD>/title.ext`)
    #[clap(short, long, value_name = "DIR_OR_FILE", parse(from_os_str))]
    pub file: Option<path::PathBuf>,

    #[clap(flatten, next_help_heading = "CONFIG")]
    pub config: args::ConfigArgs,
}

impl RenameArgs {
    pub fn run(&self) -> Result<()> {
        let mut config = self.config.load_config()?;
        config.include_drafts = true;
        let config = cobalt::cobalt_model::Config::from_config(config)?;

        let source = self.src.clone();

        let title = self.title.as_ref();

        let mut file = env::current_dir().expect("How does this fail?");
        if let Some(rel_file) = self.file.as_deref() {
            file.push(rel_file)
        }

        rename_document(&config, source, title, file)
            .with_context(|_| failure::format_err!("Could not rename `{}`", title))?;

        Ok(())
    }
}

/// Publish a document
#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
pub struct PublishArgs {
    /// Document to publish
    #[clap(value_name = "FILE", parse(from_os_str))]
    pub filename: path::PathBuf,

    #[clap(flatten, next_help_heading = "CONFIG")]
    pub config: args::ConfigArgs,
}

impl PublishArgs {
    pub fn run(&self) -> Result<()> {
        let mut config = self.config.load_config()?;
        config.include_drafts = true;
        let config = cobalt::cobalt_model::Config::from_config(config)?;

        let filename = self.filename.as_path();
        let mut file = env::current_dir().expect("How does this fail?");
        file.push(path::Path::new(filename));

        publish_document(&config, &file)
            .with_context(|_| failure::format_err!("Could not publish `{:?}`", file))?;

        Ok(())
    }
}

const COBALT_YML: &str = "
site:
  title: cobalt blog
  description: Blog Posts Go Here
  base_url: http://example.com
posts:
  rss: rss.xml
";

const DEFAULT_LAYOUT: &str = "<!DOCTYPE html>
<html>
    <head>
        <meta charset=\"utf-8\">
        <title>{{ page.title }}</title>
    </head>
    <body>
    <div>
      <h2>{{ page.title }}</h2>
      {{ page.content }}
    </div>
  </body>
</html>
";

const POST_MD: &str = "---
layout: default.liquid

title: First Post
is_draft: true
---

# This is our first Post!

Welcome to the first post ever on cobalt.rs!
";

const INDEX_MD: &str = "---
layout: default.liquid
---
## Blog!

{% for post in collections.posts.pages %}
#### {{post.title}}

[{{ post.title }}]({{ post.permalink }})
{% endfor %}
";

lazy_static! {
    static ref DEFAULT: collections::HashMap<&'static str, &'static str> =
        [("pages", INDEX_MD), ("posts", POST_MD)]
            .iter()
            .cloned()
            .collect();
}

pub fn create_new_project<P: AsRef<path::Path>>(dest: P) -> Result<()> {
    create_new_project_for_path(dest.as_ref())
}

pub fn create_new_project_for_path(dest: &path::Path) -> Result<()> {
    fs::create_dir_all(dest)?;

    create_file(&dest.join("_cobalt.yml"), COBALT_YML)?;
    create_file(&dest.join("index.md"), INDEX_MD)?;

    fs::create_dir_all(&dest.join("_layouts"))?;
    create_file(&dest.join("_layouts/default.liquid"), DEFAULT_LAYOUT)?;

    fs::create_dir_all(&dest.join("posts"))?;
    create_file(&dest.join("posts/post-1.md"), POST_MD)?;

    fs::create_dir_all(&dest.join("_defaults"))?;
    create_file(&dest.join("_defaults/pages.md"), INDEX_MD)?;
    create_file(&dest.join("_defaults/posts.md"), POST_MD)?;

    Ok(())
}

pub fn create_new_document(
    config: &cobalt_model::Config,
    title: Option<&str>,
    mut file: path::PathBuf,
    extension: Option<&str>,
    edit: bool,
) -> Result<()> {
    let (parent_dir, filename, extension) = if file.is_file() {
        let filename = file.file_name().unwrap().to_string_lossy().into_owned();
        let ext = extension
            .map(|e| e.to_owned())
            .or_else(|| file.extension().map(|s| s.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "md".to_owned());
        file.pop();
        let parent_dir = file.clone();
        (parent_dir, Some(filename), ext)
    } else {
        let parent_dir = file.clone();
        let filename = None;
        let ext = extension
            .map(|e| e.to_owned())
            .unwrap_or_else(|| "md".to_owned());
        (parent_dir, filename, ext)
    };

    let interim_path = parent_dir.join(format!("NON_EXISTENT.{}", extension));
    let interim_path = cobalt_core::SourcePath::from_root(&config.source, &interim_path)
        .ok_or_else(|| {
            failure::format_err!(
                "New file {} not project directory ({})",
                file.display(),
                config.source.display()
            )
        })?;

    let source_files =
        cobalt_core::Source::new(&config.source, config.ignore.iter().map(|s| s.as_str()))?;
    let collection_slug = if source_files.includes_file(&interim_path.abs_path) {
        match cobalt::classify_path(
            &interim_path.rel_path,
            &config.pages,
            &config.posts,
            &config.page_extensions,
        ) {
            Some((slug, _)) => slug,
            None => failure::bail!("Target file is an asset: {}", file.display()),
        }
    } else {
        failure::bail!("Target file is ignored: {}", file.display());
    };

    let source_path = config
        .source
        .join(format!("_defaults/{}.{}", collection_slug, extension));
    let source = if source_path.is_file() {
        cobalt_model::files::read_file(&source_path)
            .with_context(|_| failure::format_err!("Failed to read default: {:?}", source_path))?
    } else {
        debug!(
            "No custom default provided ({:?}), falling back to built-in",
            source_path
        );
        if extension != "md" {
            failure::bail!("No builtin default for `{}` files, only `md`", extension,);
        }
        // For custom collections, use a post default.
        let default = *DEFAULT.get(collection_slug).unwrap_or(&POST_MD);
        default.to_string()
    };

    let parsed = cobalt_model::Document::parse(&source)?;
    let (mut front, content) = parsed.into_parts();
    if let Some(title) = title {
        front.title = Some(kstring::KString::from_ref(title));
    } else {
        front.title = Some(kstring::KString::from_ref("Untitled"));
    }

    let doc = cobalt_model::Document::new(front.clone(), content);
    let mut doc = doc.to_string();
    if edit || title.is_none() {
        doc = scrawl::editor::new()
            .extension(extension.as_str())
            .contents(doc.as_str())
            .open()?;
        let parsed = cobalt_model::Document::parse(&doc)?;
        front = parsed.into_parts().0;
    }

    let title = title
        .map(|t| t.to_owned())
        .or_else(|| front.title.map(|s| s.into_string()))
        .ok_or_else(|| failure::format_err!("Title is missing"))?;
    let filename = filename
        .unwrap_or_else(|| format!("{}.{}", cobalt_model::slug::slugify(&title), extension));
    let mut file = interim_path;
    file.pop();
    file.push(&filename);

    if let Some(parent) = file.abs_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    create_file(&file.abs_path, &doc)?;
    info!("Created new {} {}", collection_slug, file.rel_path);

    Ok(())
}

fn create_file<P: AsRef<path::Path>>(path: P, content: &str) -> Result<()> {
    create_file_for_path(path.as_ref(), content)
}

fn create_file_for_path(path: &path::Path, content: &str) -> Result<()> {
    trace!("Creating file {:?}", path);

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|_| failure::format_err!("Failed to create file {}", path.display()))?;

    file.write_all(content.as_bytes())?;

    Ok(())
}

pub fn rename_document(
    config: &cobalt_model::Config,
    source: path::PathBuf,
    title: &str,
    file: path::PathBuf,
) -> Result<()> {
    let target = if file.extension().is_none() || file.is_dir() {
        let extension = source.extension().and_then(|s| s.to_str()).unwrap_or("md");
        let file_name = format!("{}.{}", cobalt_model::slug::slugify(title), extension);
        let mut file = file;
        file.push(path::Path::new(&file_name));
        file
    } else {
        file
    };

    let doc = cobalt_model::files::read_file(&source)?;
    let doc = cobalt_model::Document::parse(&doc)?;
    let (mut front, content) = doc.into_parts();

    let target = cobalt_core::SourcePath::from_root(&config.source, &target).ok_or_else(|| {
        failure::format_err!(
            "New file {} not project directory ({})",
            target.display(),
            config.source.display()
        )
    })?;

    let source_files =
        cobalt_core::Source::new(&config.source, config.ignore.iter().map(|s| s.as_str()))?;
    let collection = if source_files.includes_file(&target.abs_path) {
        match cobalt::classify_path(
            &target.rel_path,
            &config.pages,
            &config.posts,
            &config.page_extensions,
        ) {
            Some((slug, _)) if config.pages.slug == slug => &config.pages,
            Some((slug, _)) if config.posts.slug == slug => &config.posts,
            Some((slug, _)) => unreachable!("Unknown collection: {}", slug),
            None => failure::bail!("Target file is an asset: {}", target.rel_path),
        }
    } else {
        failure::bail!("Target file is ignored: {}", target.rel_path);
    };
    // Can't rely on this for drafts atm
    let full_front = front
        .clone()
        .merge_path(&target.rel_path)
        .merge(&collection.default);

    let full_front = cobalt_model::Frontmatter::from_config(full_front)?;

    front.title = Some(kstring::KString::from_ref(title));
    let doc = cobalt_model::Document::new(front, content);
    let doc = doc.to_string();
    cobalt_model::files::write_document_file(doc, &target.abs_path)?;

    if !full_front.is_draft {
        warn!("Renaming a published page might invalidate links");
    }
    fs::remove_file(source)?;

    Ok(())
}

fn prepend_date_to_filename(
    config: &cobalt_model::Config,
    file: &path::Path,
    date: &cobalt_model::DateTime,
) -> Result<()> {
    // avoid prepend to existing date prefix

    let file_stem = file
        .file_stem()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();
    let (_, file_stem) = cobalt_config::path::parse_file_stem(file_stem);
    let file_name = format!(
        "{}{}.{}",
        (**date).format("%Y-%m-%d-"),
        file_stem,
        file.extension()
            .and_then(|os| os.to_str())
            .unwrap_or_else(|| config
                .page_extensions
                .get(0)
                .expect("at least one element is enforced by config validator"))
    );
    trace!("`publish_date_in_filename` setting is activated, prefix filename with date, new filename: {}", file_name);
    fs::rename(file, file.with_file_name(file_name))?;
    Ok(())
}

fn move_from_drafts_to_posts(
    config: &cobalt_model::Config,
    file: &path::Path,
) -> Result<path::PathBuf> {
    if let Some(drafts_dir) = config.posts.drafts_dir.as_ref() {
        let drafts_root = drafts_dir.to_path(&config.source);
        if let Ok(relpath) = file.strip_prefix(drafts_root) {
            let target = config.posts.dir.to_path(&config.source).join(relpath);
            log::trace!(
                "post is in `drafts_dir`; moving it to `posts` directory: {}",
                target.display()
            );
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).with_context(|_| {
                    failure::format_err!("Could not create {}", parent.display())
                })?;
            }
            fs::rename(file, &target)?;
            return Ok(target);
        }
    }

    Ok(file.to_owned())
}

pub fn publish_document(config: &cobalt_model::Config, file: &path::Path) -> Result<()> {
    let doc = cobalt_model::files::read_file(file)?;
    let doc = cobalt_model::Document::parse(&doc)?;
    let (mut front, content) = doc.into_parts();

    let date = cobalt_model::DateTime::now();
    front.is_draft = Some(false);
    front.published_date = Some(date);

    let doc = cobalt_model::Document::new(front, content);
    let doc = doc.to_string();
    cobalt_model::files::write_document_file(doc, file)?;

    let file = move_from_drafts_to_posts(config, file)?;
    let file = cobalt_core::SourcePath::from_root(&config.source, &file).ok_or_else(|| {
        failure::format_err!(
            "New file {} not project directory ({})",
            file.display(),
            config.source.display()
        )
    })?;

    let source_files =
        cobalt_core::Source::new(&config.source, config.ignore.iter().map(|s| s.as_str()))?;
    let collection = if source_files.includes_file(&file.abs_path) {
        match cobalt::classify_path(
            &file.rel_path,
            &config.pages,
            &config.posts,
            &config.page_extensions,
        ) {
            Some((slug, _)) if config.pages.slug == slug => &config.pages,
            Some((slug, _)) if config.posts.slug == slug => &config.posts,
            Some((slug, _)) => unreachable!("Unknown collection: {}", slug),
            None => failure::bail!("Target file is an asset: {}", file.rel_path),
        }
    } else {
        failure::bail!("Target file is ignored: {}", file.rel_path);
    };

    if collection.publish_date_in_filename {
        prepend_date_to_filename(config, &file.abs_path, &date)?;
    }
    Ok(())
}
