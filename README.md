# ![Cobalt](https://raw.githubusercontent.com/cobalt-org/logos/master/cobald.logo.02.resize.png)  

[![](https://travis-ci.org/cobalt-org/cobalt.rs.svg?branch=master)](https://travis-ci.org/cobalt-org/cobalt.rs)
[![](https://coveralls.io/repos/cobalt-org/cobalt.rs/badge.svg?branch=master&service=github)](https://coveralls.io/github/cobalt-org/cobalt.rs?branch=master)
[![Gitter](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/cobalt-org/cobalt.rs?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)

A static site generator written in [Rust](http://www.rust-lang.org/).

## Installation

```
  $ cargo install --git https://github.com/cobalt-org/cobalt.rs
```

## Examples

There are a few people already using `cobalt`. Here is a list of projects and their source code that use `cobalt`.

- [tak1n.github.io](https://tak1n.github.io) [Source](https://github.com/tak1n/tak1n.github.io)
- [Amethyst.rs](https://amethyst.rs)  [Source](https://github.com/amethyst/website)
- [johannh.me](http://johannh.me) [Source](https://github.com/johannhof/johannhof.github.io)

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

Posts live in `_posts`. You can use a different directory for posts with the `-p` flag.

Example:

```yaml
extends: posts.liquid

title:   My first Blogpost
date:    01 Jan 2016 21:00:00 +0100
---
Hey there this is my first blogpost and this is super awesome.

My Blog is lorem ipsum like, yes it is..
```

The content before ```---``` are meta attributes ("front matter") made accessible to the template via their key (see below).

The ```extends``` attribute specifies which layout will be used.

The ```date``` attribute will be used to sort blog posts (from last to first). ```date``` must have the format `%dd %Mon %YYYY %HH:%MM:%SS %zzzz`, so for example `27 May 2016 21:00:30 +0100`.

### Other files

Any file with the .md or .liquid file extension is considered a liquid template and will be parsed for metadata and compiled using liquid, like a post.

Unlike posts, files outside the ``_posts`` directory will not be indexed as blog posts and not passed to the index file in the list of contents.

All other files and directories in the source folder will be recursively added to your destination folder.

You can specify different template extensions by setting the `template_extensions` field in your config file:

```yaml
template_extensions: ['txt', 'lqd']
```

#### Custom paths

Custom paths are much like permalinks in Jekyll, but with a bit more flexibility. You can specify a `path` attribute in the front matter of
any document to give it a custom path. The path is always relative to the document root, independent of where the file is located.

Example:

```yaml
extends: posts.liquid

title:   My first Blogpost
path: /some/other/path/
```

would result in a file with the url `your-website.com/some/other/path/index.html`.

Any attribute in the front matter can be interpolated into the path. If you set a `date` attribute you have access to several other custom attributes. See the Jekyll documentation.

More examples:

```yaml
date: 01 Jan 2016 21:00:00 +0100
path: /:year/:month/:day/thing.html
```
-> `/2016/01/01/thing.html`

```yaml
date: 01 Jan 2016 21:00:00 +0100
author: johann
path: /:author/:year/:month/:day/title
```
-> `/johann/2016/01/01/title/index.html`

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
 <a href="{{post.path}}">{{ post.title }}</a>
{% endfor %}
```

### RSS

To generate an RSS file from the metadata of your `_posts`, you need to provide the following data in your config.file:

```yaml
# path where the RSS file should be generated
rss: rss.xml
name: My blog!
description: Blog description
link: http://example.com
```

None of these fields are optional, as by the [RSS 2.0 spec]().

Make sure to also provide the fields `title`, `date` and `description` in the front matter of your posts.

### Import

To import your site to your `gh-pages` branch you can either pass a `build --import` flag when you build the site or after you have build the site with `build` you can run `import`. There are also some flags that can be found via `import --help`.

**Note:** to import to gitlab pages you can pass `import --branch gl-pages`
