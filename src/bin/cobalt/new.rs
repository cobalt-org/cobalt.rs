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
}

pub fn new_command(matches: &clap::ArgMatches) -> Result<()> {
    let config = args::get_config(matches)?;
    let config = config.build()?;

    let title = matches.value_of("TITLE").unwrap();

    let mut file = env::current_dir().expect("How does this fail?");
    if let Some(rel_file) = matches.value_of("file") {
        file.push(path::Path::new(rel_file))
    }

    create_new_document(&config, title, file)
        .chain_err(|| format!("Could not create `{}`", title))?;

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

    Ok(())
}

pub fn create_new_document(
    config: &cobalt_model::Config,
    title: &str,
    file: path::PathBuf,
) -> Result<()> {
    let file = if file.extension().is_none() {
        let file_name = format!("{}.md", cobalt_model::slug::slugify(title));
        let mut file = file;
        file.push(path::Path::new(&file_name));
        file
    } else {
        file
    };

    let rel_file = file.strip_prefix(&config.source).map_err(|_| {
        format!(
            "New file {:?} not project directory ({:?})",
            file, config.source
        )
    })?;

    let posts = config.posts.clone().build()?;
    let (file_type, doc) = if file.starts_with(posts.pages.subtree())
        || posts
            .drafts
            .as_ref()
            .map(|d| file.starts_with(d.subtree()))
            .unwrap_or(false)
    {
        ("post", POST_MD)
    } else {
        ("page", INDEX_MD)
    };

    let doc = cobalt_model::DocumentBuilder::<cobalt_model::FrontmatterBuilder>::parse(doc)?;
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
