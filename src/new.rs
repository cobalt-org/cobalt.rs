#![allow(non_upper_case_globals)]

const cobalt_yml: &'static [u8] = b"
name: Cobalt.rs Blog
source: \".\"
dest: \"build\"
ignore:
  - ./.git/*
  - ./build/*
";

const default_liquid: &'static [u8] = b"
<!DOCTYPE html>
<html>
    <head>
      <meta charset=\"utf-8\">
    	<meta http-equiv=\"X-UA-Compatible\" content=\"IE=edge\">
    	<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">

        {% if is_post %}
          <title>{{ title }}</title>
        {% else %}
       	  <title>Cobalt.rs Blog</title>
        {% endif %}

	    <!-- Latest compiled and minified CSS -->
     	<link rel=\"stylesheet\" href=\"https://maxcdn.bootstrapcdn.com/bootstrap/3.3.6/css/bootstrap.min.css\" integrity=\"sha384-1q8mTJOASx8j1Au+a5WDVnPi2lkFfwwEAa8hDDdjZlpLegxhjVME1fgjWPGmkzs7\" crossorigin=\"anonymous\">
      	<!-- Optional theme -->
     	<!--<link rel=\"stylesheet\" href=\"https://maxcdn.bootstrapcdn.com/bootstrap/3.3.6/css/bootstrap-theme.min.css\" integrity=\"sha384-fLW2N01lMqjakBkx3l/M9EahuwpSfeNvV63J5ezn3uZzapT0u7EYsXMjQV+0En5r\" crossorigin=\"anonymous\">-->

      <script src=\"https://ajax.googleapis.com/ajax/libs/jquery/1.11.3/jquery.min.js\"></script>
      	<!-- Latest compiled and minified JavaScript -->
     	<script src=\"https://maxcdn.bootstrapcdn.com/bootstrap/3.3.6/js/bootstrap.min.js\" integrity=\"sha384-0mSbJDEHialfmuBBQP6A4Qrprq5OVfW37PRR3j5ELqxss1yVqOtnepnHVP9aJ7xS\" crossorigin=\"anonymous\"></script>
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

const post_liquid: &'static [u8] = b"
<div>
  <h2>{{ title }}</h2>
  <p>
    {{content}}
  </p>
</div>
";

const post_1_md: &'static [u8] = b"
extends: default.liquid

title: First Post
date: 14 January 2016 21:00:30 -0500
---

# This is our first Post!

Welcome to the first post ever on cobalt.rs!
";

const index_liquid: &'static [u8] = b"
extends: default.liquid
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

    try!(create_file(&dest.join(".coblat.yml"), cobalt_yml));
    try!(create_file(&dest.join("index.liquid"), index_liquid));

    try!(create_folder(&dest.join("_layouts")));
    try!(create_file(&dest.join("_layouts/default.liquid"), default_liquid));
    try!(create_file(&dest.join("_layouts/post.liquid"), post_liquid));

    try!(create_folder(&dest.join("_posts")));
    try!(create_file(&dest.join("_posts/post-1.md"), post_1_md));

    Ok(())
}

fn create_folder<P: AsRef<Path>>(path: P) -> Result<()> {
    try!(DirBuilder::new()
                    .recursive(true)
                    .create(path));

    Ok(())
}

fn create_file<P: AsRef<Path>>(name: P, content: &[u8]) -> Result<()> {
    let mut file = try!(OpenOptions::new()
                     .write(true)
                     .create(true)
                     .open(name));

    try!(file.write_all(content));

    Ok(())
}
