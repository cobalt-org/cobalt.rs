#![allow(non_upper_case_globals)]

const cobalt_yml: &'static [u8] = b"name: cobalt blog
source: \".\"
dest: \"build\"
ignore:
  - .git/*
  - build/*
";

const default_liquid: &'static [u8] = b"<!DOCTYPE html>
<html>
    <head>
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

const post_liquid: &'static [u8] = b"<div>
  <h2>{{ title }}</h2>
  <p>
    {{content}}
  </p>
</div>
";

const post_1_md: &'static [u8] = b"extends: default.liquid

title: First Post
date: 14 January 2016 21:00:30 -0500
---

# This is our first Post!

Welcome to the first post ever on cobalt.rs!
";

const index_liquid: &'static [u8] = b"extends: default.liquid
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

use std::path::Path;
use std::fs::{DirBuilder, OpenOptions};
use std::io::Write;
use error::Result;

pub fn create_new_project<P: AsRef<Path>>(dest: P) -> Result<()> {
    let dest = dest.as_ref();

    try!(create_folder(&dest));

    try!(create_file(&dest.join(".cobalt.yml"), cobalt_yml));
    try!(create_file(&dest.join("index.liquid"), index_liquid));

    try!(create_folder(&dest.join("_layouts")));
    try!(create_file(&dest.join("_layouts/default.liquid"), default_liquid));
    try!(create_file(&dest.join("_layouts/post.liquid"), post_liquid));

    try!(create_folder(&dest.join("_posts")));
    try!(create_file(&dest.join("_posts/post-1.md"), post_1_md));

    Ok(())
}

fn create_folder<P: AsRef<Path>>(path: P) -> Result<()> {
    trace!("Creating folder {:?}", &path.as_ref());

    try!(DirBuilder::new()
        .recursive(true)
        .create(path));

    Ok(())
}

fn create_file<P: AsRef<Path>>(name: P, content: &[u8]) -> Result<()> {
    trace!("Creating file {:?}", &name.as_ref());

    let mut file = try!(OpenOptions::new()
        .write(true)
        .create(true)
        .open(name));

    try!(file.write_all(content));

    Ok(())
}
