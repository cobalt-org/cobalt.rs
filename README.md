#![Cobalt](https://raw.githubusercontent.com/cobalt-org/logos/master/cobald.logo.02.resize.png) [![](https://travis-ci.org/cobalt-org/cobalt.rs.svg?branch=master)](https://travis-ci.org/cobalt-org/cobalt.rs)

A static site generator written in [Rust](http://www.rust-lang.org/).

## Installation

Cobalt is currently in development therefore installation requires to have the development toolchain.

Also cobalt currently uses unstable features therefore a nightly toolchain is needed.

It is recommended to use [multirust](http://www.github.com/brson/multirust):

```
  $ git clone git@github.com:cobalt-org/cobalt.rs.git && cd cobalt.rs
  $ multirust override nightly-2015-10-01
  $ cargo build
```

## Usage

### Layouts

You can have custom layouts in the ```_layouts``` directory.

Layouts will be compiled as [liquid](https://github.com/cobalt-org/liquid-rust) templates.

### Posts

Posts live in ```_posts```.

Example:

```text
@extends: posts.tpl

title:   My first Blogpost
date:    24/08/2014 at 15:36
---
Hey there this is my first blogpost and this is super awesome.

My Blog is lorem ipsum like, yes it is..
```

The content before ```---``` are meta attributes made accessible to the template via their key (see below).

The ```@extends``` attribute specifies which layout will be used.

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

`{{ posts }}` is a list of the attributes of all templates in the `_posts` directory. Example:
