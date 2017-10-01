use std::path::Path;
use std::fs::{DirBuilder, OpenOptions};
use std::io::Write;
use error::Result;
use config::Config;

const COBALT_YML: &'static [u8] = b"name: cobalt blog
source: \".\"
dest: \"build\"
ignore:
  - .git/*
  - build/*
";

const DEFAULT_LAYOUT: &'static [u8] = b"<!DOCTYPE html>
<html>
    <head>
        <meta charset=\"utf-8\">
        {% if is_post %}
          <title>{{ title }}</title>
        {% else %}
          <title>Cobalt.rs Blog</title>
        {% endif %}
    </head>
    <body>
    <div>
      {% if is_post %}
        {% include '_layouts/post.liquid' %}
      {% else %}
        {{ content }}
      {% endif %}
    </div>
  </body>
</html>
";

const POST_LAYOUT: &'static [u8] = b"<div>
  <h2>{{ title }}</h2>
  <p>
    {{content}}
  </p>
</div>
";

const POST_MD: &'static [u8] = b"extends: default.liquid

title: First Post
date: 14 January 2016 21:00:30 -0500
---

# This is our first Post!

Welcome to the first post ever on cobalt.rs!
";

const INDEX_LIQUID: &'static [u8] = b"extends: default.liquid
---
<div >
  <h2>Blog!</h2>
  <!--<br />-->
  <div>
    {% for post in posts %}
      <div>
        <h4>{{post.title}}</h4>
        <h4><a href=\"{{post.path}}\">{{ post.title }}</a></h4>
      </div>
    {% endfor %}
  </div>
</div>
";

pub fn create_new_project<P: AsRef<Path>>(dest: P) -> Result<()> {
    let dest = dest.as_ref();

    create_folder(&dest)?;

    create_file(&dest.join(".cobalt.yml"), COBALT_YML)?;
    create_file(&dest.join("index.liquid"), INDEX_LIQUID)?;

    create_folder(&dest.join("_layouts"))?;
    create_file(&dest.join("_layouts/default.liquid"), DEFAULT_LAYOUT)?;
    create_file(&dest.join("_layouts/post.liquid"), POST_LAYOUT)?;

    create_folder(&dest.join("posts"))?;
    create_file(&dest.join("posts/post-1.md"), POST_MD)?;

    Ok(())
}

pub fn create_new_document(doc_type: &str, name: &str, config: &Config) -> Result<()> {
    let path = Path::new(&config.source);
    let full_path = &path.join(&config.posts).join(name);

    match doc_type {
        "page" => create_file(name, INDEX_LIQUID)?,
        "post" => create_file(full_path, POST_MD)?,
        _ => bail!("Unsupported document type {}", doc_type),
    }

    Ok(())
}

fn create_folder<P: AsRef<Path>>(path: P) -> Result<()> {
    trace!("Creating folder {:?}", &path.as_ref());

    DirBuilder::new().recursive(true).create(path)?;

    Ok(())
}

fn create_file<P: AsRef<Path>>(name: P, content: &[u8]) -> Result<()> {
    trace!("Creating file {:?}", &name.as_ref());

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(name)?;

    file.write_all(content)?;

    Ok(())
}
