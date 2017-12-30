use std::path;
use std::fs;
use std::io::Write;

use error::*;
use cobalt_model::Config;
use cobalt_model::files;
use cobalt_model::slug;
use cobalt_model;

const COBALT_YML: &'static str = "
site:
  title: cobalt blog
  description: Blog Posts Go Here
  base_url: http://example.com
posts:
  rss: rss.xml
";

const DEFAULT_LAYOUT: &'static str = "<!DOCTYPE html>
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

const POST_MD: &'static str = "layout: default.liquid

title: First Post
is_draft: true
---

# This is our first Post!

Welcome to the first post ever on cobalt.rs!
";

const INDEX_MD: &'static str = "layout: default.liquid
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

pub fn create_new_document(config: &Config, title: &str, file: path::PathBuf) -> Result<()> {
    let file = if file.extension().is_none() {
        let file_name = format!("{}.md", slug::slugify(title));
        let mut file = file;
        file.push(path::Path::new(&file_name));
        file
    } else {
        file
    };

    let rel_file = file.strip_prefix(&config.source)
        .map_err(|_| {
                     format!("New file {:?} not project directory ({:?})",
                             file,
                             config.source)
                 })?;

    let (file_type, doc) = if rel_file.starts_with(path::Path::new(&config.posts.dir)) ||
                              config
                                  .posts
                                  .drafts_dir
                                  .as_ref()
                                  .map(|dir| rel_file.starts_with(path::Path::new(dir)))
                                  .unwrap_or(false) {
        ("post", POST_MD)
    } else {
        ("page", INDEX_MD)
    };

    let doc = cobalt_model::DocumentBuilder::<cobalt_model::FrontmatterBuilder>::parse(doc)?;
    let (front, content) = doc.parts();
    let front = front.set_title(title.to_owned());
    let doc = cobalt_model::DocumentBuilder::<cobalt_model::FrontmatterBuilder>::new(front,
                                                                                     content);
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
    let doc = files::read_file(file)?;
    let doc = cobalt_model::DocumentBuilder::<cobalt_model::FrontmatterBuilder>::parse(&doc)?;
    let (front, content) = doc.parts();

    let date = cobalt_model::DateTime::now();
    let front = front.set_draft(false).set_published_date(date);

    let doc = cobalt_model::DocumentBuilder::<cobalt_model::FrontmatterBuilder>::new(front,
                                                                                     content);
    let doc = doc.to_string();

    files::write_document_file(doc, file)?;

    Ok(())
}
