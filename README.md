# ![Cobalt](https://raw.githubusercontent.com/cobalt-org/logos/master/cobald.logo.02.resize.png)  

[![](https://travis-ci.org/cobalt-org/cobalt.rs.svg?branch=master)](https://travis-ci.org/cobalt-org/cobalt.rs)
[![](https://coveralls.io/repos/cobalt-org/cobalt.rs/badge.svg?branch=master&service=github)](https://coveralls.io/github/cobalt-org/cobalt.rs?branch=master)
[![Gitter](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/cobalt-org/cobalt.rs?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)

A static site generator written in [Rust](http://www.rust-lang.org/).

## Installation

```
  $ cargo install --git https://github.com/cobalt-org/cobalt.rs
```

## Usage

```
  $ cobalt build -s path/to/your/source -d path/to/your/destination
```

See more options with

```
  $ cobalt -h
```

### Layouts

You can have custom layouts in the ```_layouts``` directory.

Layouts will be compiled as [liquid](https://github.com/cobalt-org/liquid-rust) templates.

### Posts

Posts live in ```_posts```.

Example:

```text
extends: posts.tpl

title:   My first Blogpost
date:    01 Jan 2016 21:00:00 +0100
---
Hey there this is my first blogpost and this is super awesome.

My Blog is lorem ipsum like, yes it is..
```

The content before ```---``` are meta attributes made accessible to the template via their key (see below).

The ```extends``` attribute specifies which layout will be used.

The ```date``` attribute will be used to sort blog posts (from last to first). ```date``` must have the format `%dd %Mon %YYYY %HH:%MM:%SS %zzzz`, so for example `27 May 2016 21:00:30 +0100`.

### Other files

Any file with the .tpl file extension will be parsed for metadata and compiled using liquid, like a post.

Unlike posts, files outside the ``_posts`` directory will not be indexed as blog posts and not passed to the index file in the list of contents.

All other files and directories in the source folder will be recursively added to your destination folder.

### Attributes

All template files have access to a set of attributes.

In example above _title_ is accessible via ```{{ title }}``` and _date_ via ```{{ date }}```, for the layout template as well as the post template.

### Special Attributes

#### content

`{{ content }}` is accessible only to layouts and contains the compiled text below the ```---``` block of the post.

#### posts

`{{ posts }}` is a list of the attributes of all templates in the `_posts` directory. Example usage on a page listing all blog posts:

```
{% for post in posts %}
 <a href="blog/{{post.name}}.html">{{ post.title }}</a>
{% endfor %}
```
