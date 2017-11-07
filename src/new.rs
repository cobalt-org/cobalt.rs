use std::path::Path;
use std::fs;
use std::io::Write;
use error::*;
use config::Config;

const COBALT_YML: &'static [u8] = b"name: cobalt blog
source: \".\"
dest: \"./_site\"
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
draft: true
---

# This is our first Post!

Welcome to the first post ever on cobalt.rs!
";

const INDEX_MD: &'static [u8] = b"extends: default.liquid
---
## Blog!

{% for post in posts %}
#### {{post.title}}

#### [{{ post.title }}]({{ post.path }})
{% endfor %}
";

pub fn create_new_project<P: AsRef<Path>>(dest: P) -> Result<()> {
    create_new_project_for_path(dest.as_ref())
}

pub fn create_new_project_for_path(dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;

    create_file(&dest.join(".cobalt.yml"), COBALT_YML)?;
    create_file(&dest.join("index.md"), INDEX_MD)?;

    fs::create_dir_all(&dest.join("_layouts"))?;
    create_file(&dest.join("_layouts/default.liquid"), DEFAULT_LAYOUT)?;
    create_file(&dest.join("_layouts/post.liquid"), POST_LAYOUT)?;

    fs::create_dir_all(&dest.join("posts"))?;
    create_file(&dest.join("posts/post-1.md"), POST_MD)?;

    Ok(())
}

pub fn create_new_document(doc_type: &str, name: &str, config: &Config) -> Result<()> {
    let path = Path::new(&config.source);
    let full_path = &path.join(&config.posts.dir).join(name);

    match doc_type {
        "page" => create_file(name, INDEX_MD)?,
        "post" => create_file(full_path, POST_MD)?,
        _ => bail!("Unsupported document type {}", doc_type),
    }

    Ok(())
}

fn create_file<P: AsRef<Path>>(path: P, content: &[u8]) -> Result<()> {
    create_file_for_path(path.as_ref(), content)
}

fn create_file_for_path(path: &Path, content: &[u8]) -> Result<()> {
    trace!("Creating file {:?}", path);

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .chain_err(|| format!("Failed to create file {:?}", path))?;

    file.write_all(content)?;

    Ok(())
}
