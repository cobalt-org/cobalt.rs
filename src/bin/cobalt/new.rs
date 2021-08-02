use std::borrow;
use std::collections;
use std::env;
use std::fs;
use std::io::Write;
use std::path;

use cobalt::cobalt_model;
use failure::ResultExt;

use crate::args;
use crate::error::*;

pub fn init_command_args() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("init")
        .about("create a new cobalt project")
        .arg(
            clap::Arg::with_name("DIRECTORY")
                .help("Target directory")
                .default_value("./")
                .index(1),
        )
}

pub fn init_command(matches: &clap::ArgMatches) -> Result<()> {
    let directory = matches.value_of("DIRECTORY").unwrap();

    create_new_project(&directory.to_string())
        .with_context(|_| failure::err_msg("Could not create a new cobalt project"))?;
    info!("Created new project at {}", directory);

    Ok(())
}

pub fn new_command_args() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("new")
        .about("Create a document")
        .args(&args::get_config_args())
        .arg(
            clap::Arg::with_name("TITLE")
                .required(true)
                .help("Title of the post")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("DIR_OR_FILE")
                .help("New document's parent directory or file (default: `<CWD>/title.ext`)")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("with-ext")
                .long("with-ext")
                .value_name("EXT")
                .help("The default file's extension (e.g. `liquid`)")
                .takes_value(true),
        )
}

pub fn new_command(matches: &clap::ArgMatches) -> Result<()> {
    let mut config = args::get_config(matches)?;
    config.include_drafts = true;
    let config = cobalt::cobalt_model::Config::from_config(config)?;

    let title = matches.value_of("TITLE").unwrap();

    let mut file = env::current_dir().expect("How does this fail?");
    if let Some(rel_file) = matches.value_of("file") {
        file.push(path::Path::new(rel_file))
    }

    let ext = matches.value_of("with-ext");

    create_new_document(&config, title, file, ext)
        .with_context(|_| failure::format_err!("Could not create `{}`", title))?;

    Ok(())
}

pub fn rename_command_args() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("rename")
        .about("Rename a document")
        .args(&args::get_config_args())
        .arg(
            clap::Arg::with_name("SRC")
                .required(true)
                .help("File to rename")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("TITLE")
                .required(true)
                .help("Title of the post")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("DIR_OR_FILE")
                .help("New document's parent directory or file (default: `<CWD>/title.ext`)")
                .takes_value(true),
        )
}

pub fn rename_command(matches: &clap::ArgMatches) -> Result<()> {
    let mut config = args::get_config(matches)?;
    config.include_drafts = true;
    let config = cobalt::cobalt_model::Config::from_config(config)?;

    let source = path::PathBuf::from(matches.value_of("SRC").unwrap());

    let title = matches.value_of("TITLE").unwrap();

    let mut file = env::current_dir().expect("How does this fail?");
    if let Some(rel_file) = matches.value_of("file") {
        file.push(path::Path::new(rel_file))
    }
    let file = file;

    rename_document(&config, source, title, file)
        .with_context(|_| failure::format_err!("Could not rename `{}`", title))?;

    Ok(())
}

pub fn publish_command_args() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("publish")
        .about("Publish a document")
        .arg(
            clap::Arg::with_name("FILENAME")
                .required(true)
                .help("Document path to publish")
                .takes_value(true),
        )
}

pub fn publish_command(matches: &clap::ArgMatches) -> Result<()> {
    let filename = matches
        .value_of("FILENAME")
        .expect("required parameters are present");
    let mut file = env::current_dir().expect("How does this fail?");
    file.push(path::Path::new(filename));
    let file = file;
    let mut config = args::get_config(matches)?;
    config.include_drafts = true;
    let config = cobalt::cobalt_model::Config::from_config(config)?;

    publish_document(&config, &file)
        .with_context(|_| failure::format_err!("Could not publish `{:?}`", file))?;

    Ok(())
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
    title: &str,
    file: path::PathBuf,
    extension: Option<&str>,
) -> Result<()> {
    let (file, extension) = if file.extension().is_none() || file.is_dir() {
        let extension = extension.unwrap_or("md");
        let file_name = format!("{}.{}", cobalt_model::slug::slugify(title), extension);
        let mut file = file;
        file.push(path::Path::new(&file_name));
        (file, borrow::Cow::Borrowed(extension))
    } else {
        // The user-provided extension will be used for selecting a template
        let extension = extension.map(borrow::Cow::Borrowed).unwrap_or_else(|| {
            borrow::Cow::Owned(
                file.extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .to_string(),
            )
        });
        (file, extension)
    };

    let rel_file = file.strip_prefix(&config.source).map_err(|_| {
        failure::format_err!(
            "New file {} not project directory ({})",
            file.display(),
            config.source.display()
        )
    })?;

    let source_files =
        cobalt_core::Source::new(&config.source, config.ignore.iter().map(|s| s.as_str()))?;
    let collection_slug = if source_files.includes_file(&file) {
        match cobalt::classify_path(
            &file,
            &source_files,
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
            failure::bail!(
                "No builtin default for `{}` files, only `md`: {}",
                extension,
                file.display()
            );
        }
        // For custom collections, use a post default.
        let default = *DEFAULT.get(collection_slug).unwrap_or(&POST_MD);
        default.to_string()
    };

    let doc = cobalt_model::Document::parse(&source)?;
    let (mut front, content) = doc.into_parts();
    front.title = Some(kstring::KString::from_ref(title));
    let doc = cobalt_model::Document::new(front, content);
    let doc = doc.to_string();

    create_file(&file, &doc)?;
    info!("Created new {} {:?}", collection_slug, rel_file);

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

    let source_files =
        cobalt_core::Source::new(&config.source, config.ignore.iter().map(|s| s.as_str()))?;
    let collection = if source_files.includes_file(&target) {
        match cobalt::classify_path(
            &target,
            &source_files,
            &config.pages,
            &config.posts,
            &config.page_extensions,
        ) {
            Some((slug, _)) if config.pages.slug == slug => &config.pages,
            Some((slug, _)) if config.posts.slug == slug => &config.posts,
            Some((slug, _)) => unreachable!("Unknown collection: {}", slug),
            None => failure::bail!("Target file is an asset: {}", target.display()),
        }
    } else {
        failure::bail!("Target file is ignored: {}", target.display());
    };
    // Can't rely on this for drafts atm
    let rel_src = target
        .strip_prefix(&config.source)
        .ok()
        .and_then(|s| cobalt_config::RelPath::from_path(s))
        .expect("file was found under the root");
    let full_front = front
        .clone()
        .merge_path(&rel_src)
        .merge(&collection.default);

    let full_front = cobalt_model::Frontmatter::from_config(full_front)?;

    front.title = Some(kstring::KString::from_ref(title));
    let doc = cobalt_model::Document::new(front, content);
    let doc = doc.to_string();
    cobalt_model::files::write_document_file(doc, target)?;

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
            .unwrap_or_else(|| &config
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

    let file = move_from_drafts_to_posts(&config, &file)?;

    let source_files =
        cobalt_core::Source::new(&config.source, config.ignore.iter().map(|s| s.as_str()))?;
    let collection = if source_files.includes_file(&file) {
        match cobalt::classify_path(
            &file,
            &source_files,
            &config.pages,
            &config.posts,
            &config.page_extensions,
        ) {
            Some((slug, _)) if config.pages.slug == slug => &config.pages,
            Some((slug, _)) if config.posts.slug == slug => &config.posts,
            Some((slug, _)) => unreachable!("Unknown collection: {}", slug),
            None => failure::bail!("Target file is an asset: {}", file.display()),
        }
    } else {
        failure::bail!("Target file is ignored: {}", file.display());
    };

    if collection.publish_date_in_filename {
        prepend_date_to_filename(&config, &file, &date)?;
    }
    Ok(())
}
