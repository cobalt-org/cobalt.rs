use std::borrow;
use std::collections;
use std::env;
use std::fs;
use std::io::Write;
use std::path;

use clap;
use cobalt::cobalt_model;

use args;
use error::*;

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
        .chain_err(|| "Could not create a new cobalt project")?;
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
    let config = args::get_config(matches)?;
    let config = config.build()?;

    let title = matches.value_of("TITLE").unwrap();

    let mut file = env::current_dir().expect("How does this fail?");
    if let Some(rel_file) = matches.value_of("file") {
        file.push(path::Path::new(rel_file))
    }

    let ext = matches.value_of("with-ext");

    create_new_document(&config, title, file, ext)
        .chain_err(|| format!("Could not create `{}`", title))?;

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
    let config = args::get_config(matches)?;
    let config = config.build()?;

    let source = path::PathBuf::from(matches.value_of("SRC").unwrap());

    let title = matches.value_of("TITLE").unwrap();

    let mut file = env::current_dir().expect("How does this fail?");
    if let Some(rel_file) = matches.value_of("file") {
        file.push(path::Path::new(rel_file))
    }
    let file = file;

    rename_document(&config, source, title, file)
        .chain_err(|| format!("Could not rename `{}`", title))?;

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
    let file = matches
        .value_of("FILENAME")
        .expect("required parameters are present");
    let file = path::Path::new(file);

    publish_document(file).chain_err(|| format!("Could not publish `{:?}`", file))?;

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

const POST_MD: &str = "layout: default.liquid

title: First Post
is_draft: true
---

# This is our first Post!

Welcome to the first post ever on cobalt.rs!
";

const INDEX_MD: &str = "layout: default.liquid
---
## Blog!

{% for post in collections.posts.pages %}
#### {{post.title}}

[{{ post.title }}]({{ post.permalink }})
{% endfor %}
";

lazy_static! {
    static ref DEFAULT: collections::HashMap<&'static str, &'static str> = [("pages", INDEX_MD), ("posts", POST_MD)].iter().cloned().collect();
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
        format!(
            "New file {:?} not project directory ({:?})",
            file, config.source
        )
    })?;

    let pages = config.pages.clone().build()?;
    let posts = config.posts.clone().build()?;
    let file_type = if posts.pages.includes_file(&file)
        || posts
            .drafts
            .map(|d| d.includes_file(&file))
            .unwrap_or_default()
    {
        posts.slug.as_str()
    } else if pages.pages.includes_file(&file)
        || pages
            .drafts
            .map(|d| d.includes_file(&file))
            .unwrap_or_default()
    {
        pages.slug.as_str()
    } else {
        bail!(
            "Target file wouldn't be a member of any collection: {:?}",
            file
        );
    };

    let source_path = config
        .source
        .join(format!("_defaults/{}.{}", file_type, extension));
    let source = if source_path.is_file() {
        cobalt_model::files::read_file(&source_path)
            .chain_err(|| format!("Failed to read default: {:?}", source_path))?
    } else {
        debug!(
            "No custom default provided ({:?}), falling back to built-in",
            source_path
        );
        if extension != "md" {
            bail!(
                "No builtin default for `{}` files, only `md`: {:?}",
                extension,
                file
            );
        }
        // For custom collections, use a post default.
        DEFAULT.get(file_type).unwrap_or(&POST_MD).to_string()
    };

    let doc = cobalt_model::DocumentBuilder::<cobalt_model::FrontmatterBuilder>::parse(&source)?;
    let (front, content) = doc.parts();
    let front = front.set_title(title.to_owned());
    let doc =
        cobalt_model::DocumentBuilder::<cobalt_model::FrontmatterBuilder>::new(front, content);
    let doc = doc.to_string();

    create_file(&file, &doc)?;
    info!("Created new {} {:?}", file_type, rel_file);

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
        .chain_err(|| format!("Failed to create file {:?}", path))?;

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
    let doc = cobalt_model::DocumentBuilder::<cobalt_model::FrontmatterBuilder>::parse(&doc)?;
    let (front, content) = doc.parts();

    let pages = config.pages.clone().build()?;
    let posts = config.posts.clone().build()?;
    let full_front = if posts.pages.includes_file(&target)
        || posts
            .drafts
            .map(|d| d.includes_file(&target))
            .unwrap_or_default()
    {
        // Can't rely on this for drafts atm
        let rel_src = target
            .strip_prefix(&config.source)
            .expect("file was found under the root");
        front
            .clone()
            .merge_path(rel_src)
            .merge(posts.default.clone())
    } else if pages.pages.includes_file(&target)
        || pages
            .drafts
            .map(|d| d.includes_file(&target))
            .unwrap_or_default()
    {
        // Can't rely on this for drafts atm
        let rel_src = target
            .strip_prefix(&config.source)
            .expect("file was found under the root");
        front
            .clone()
            .merge_path(rel_src)
            .merge(pages.default.clone())
    } else {
        bail!(
            "Target file wouldn't be a member of any collection: {:?}",
            target
        );
    };
    let full_front = full_front.build()?;

    let new_front = front.set_title(Some(title.to_string()));
    let doc =
        cobalt_model::DocumentBuilder::<cobalt_model::FrontmatterBuilder>::new(new_front, content);
    let doc = doc.to_string();
    cobalt_model::files::write_document_file(doc, target)?;

    if !full_front.is_draft {
        warn!("Renaming a published page might invalidate links");
    }
    fs::remove_file(source)?;

    Ok(())
}

pub fn publish_document(file: &path::Path) -> Result<()> {
    let doc = cobalt_model::files::read_file(file)?;
    let doc = cobalt_model::DocumentBuilder::<cobalt_model::FrontmatterBuilder>::parse(&doc)?;
    let (front, content) = doc.parts();

    let date = cobalt_model::DateTime::now();
    let front = front.set_draft(false).set_published_date(date);

    let doc =
        cobalt_model::DocumentBuilder::<cobalt_model::FrontmatterBuilder>::new(front, content);
    let doc = doc.to_string();

    cobalt_model::files::write_document_file(doc, file)?;

    Ok(())
}
