# ![Cobalt](https://raw.githubusercontent.com/cobalt-org/logos/master/cobald.logo.02.resize.png)  
[![](https://img.shields.io/crates/v/cobalt-bin.svg?maxAge=25920)](https://crates.io/crates/cobalt-bin)
[![](https://travis-ci.org/cobalt-org/cobalt.rs.svg?branch=master)](https://travis-ci.org/cobalt-org/cobalt.rs) [![](https://ci.appveyor.com/api/projects/status/gp2mmvk8dpe8wsmi/branch/master?svg=true)](https://ci.appveyor.com/project/johannhof/cobalt-rs/branch/master) 
[![](https://coveralls.io/repos/cobalt-org/cobalt.rs/badge.svg?branch=master&service=github)](https://coveralls.io/github/cobalt-org/cobalt.rs?branch=master)
[![](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/cobalt-org/cobalt.rs?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)

A static site generator written in [Rust](http://www.rust-lang.org/).

## Content

- [Installation](#installation)
- [Examples](#examples)
- [Usage](#usage)
  - [Layouts](#layouts)
  - [Posts](#posts)
  - [Other Files](#other-files)
  - [Attributes](#attributes)
  - [RSS](#rss)
  - [Import](#import)
- [Deployment](#deployment)
  - [Travis CI](#with-travis-ci)
  - [GitLab CI](#with-gitlab-ci)

## Installation

### Using the install script

No prerequisites

```
$ curl -LSfs https://japaric.github.io/trust/install.sh | sh -s -- --git cobalt-org/cobalt.rs
```

You can also manually download all releases [here](https://github.com/cobalt-org/cobalt.rs/releases).
If your platform is not supported yet, please try installing from source (see below) and file an issue.

### Using cargo

Requires Rust

```
$ cargo install cobalt-bin
```

## Examples

There are a few people already using `cobalt`. Here is a list of projects and their source code that use `cobalt`.

- [tak1n.github.io](https://tak1n.github.io) [Source](https://github.com/tak1n/tak1n.github.io)
- [amethyst.rs](https://amethyst.rs)  [Source](https://github.com/amethyst/website)
- [johannh.me](http://johannh.me) [Source](https://github.com/johannhof/johannhof.github.io)
- [kstep.me](http://kstep.me) [Source](https://github.com/kstep/kstep.github.com)

## Usage

```
  $ cobalt new myBlog
  $ cobalt build -s myBlog -d path/to/your/destination
```

See more options with

```
  $ cobalt -h
```

### Layouts

You can have custom layouts in the ```_layouts``` directory.

Layouts will be compiled as [liquid](https://github.com/cobalt-org/liquid-rust) templates.

### Posts

Posts live in `posts` by default, but you can use a different directory for posts with the `-p` flag or by setting the `posts` variable in your .cobalt.yml.

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

#### Drafts

Cobalt supports leaving posts in "draft" state. Drafts will not be rendered unless Cobalt is run with the `--drafts` flag.

To mark a post as draft you can either set `draft: true` in your front matter or add it to the drafts folder (`_drafts` by default). The draft folder location can be specified using the `draft` key in your .cobalt.yml.

### Other files

Any file with the .md or .liquid file extension is considered a liquid template and will be parsed for metadata and compiled using liquid, like a post.

Unlike posts, files outside the ``posts`` directory will not be indexed as blog posts and not passed to the index file in the list of contents.

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

You can also set a global attribute `post_path` in your .cobalt.yml that will be used for all posts.

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

`{{ posts }}` is a list of the attributes of all templates in the `posts` directory. Example usage on a page listing all blog posts:

```
{% for post in posts %}
 <a href="{{post.path}}">{{ post.title }}</a>
{% endfor %}
```

### RSS

To generate an RSS file from the metadata of your posts, you need to provide the following data in your config file:

```yaml
# path where the RSS file should be generated
rss: rss.xml
name: My blog!
description: Blog description
link: http://example.com
```

None of these fields are optional, as by the [RSS 2.0 spec]().

Make sure to also provide the fields `title`, `date` and `description` in the front matter of your posts.

### Syntax Highlighting

> This feature is currently experimental and causes installation to fail on Windows. To enable syntax highlighting, you need to install Cobalt using cargo like this:
>  ```
>  cargo install cobalt-bin --features="syntax-highlight"
>  ```

If you [annotate your Markdown code blocks](https://help.github.com/articles/creating-and-highlighting-code-blocks/#syntax-highlighting), Cobalt will automatically highlight source code using [Syntect](https://github.com/trishume/syntect/).

### Import

To import your site to your `gh-pages` branch you can either pass a `build --import` flag when you build the site or after you have build the site with `build` you can run `import`. There are also some flags that can be found via `import --help`.

## Deployment

### With Travis CI

You can easily deploy a cobalt site to `gh-pages`! To do this with travis is also very easy. You will need to have rust available on travis. In your `travis.yml` you will need to have something similar to this:

```yaml
sudo: false
language: rust

before_script:
  - cargo install cobalt-bin
  - export PATH="$PATH:/home/travis/.cargo/bin"

script:
  - cobalt build

after_success: |
  [ $TRAVIS_BRANCH = master ] &&  
  [ $TRAVIS_PULL_REQUEST = false ] &&  
  cobalt import &&
  git config user.name "Cobalt Site Deployer" &&
  git config user.email "name@example.com" &&
  git push -fq https://${GH_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages
```

For the `GH_TOKEN` you will need to create a personal access token, which can be found [here](https://github.com/settings/tokens), then you will need to use the [travis cli](https://github.com/travis-ci/travis.rb#the-travis-client-) tool to encrypt your personal access token. You can do this like so `travis encrypt GH_TOKEN=... --add env.global`

### With GitLab CI

You can also deploy a cobalt site to [GitLab Pages](http://pages.gitlab.io/) using GitLab CI. GitLab CI uses [Docker](https://docs.docker.com/), you can use [nott/cobalt](https://hub.docker.com/r/nott/cobalt/) or any other image with `cobalt` in `PATH`.

An example of `.gitlab-ci.yml`:

```yaml
image: nott/cobalt:latest

pages:
  script:
  - mkdir -p public
  - cobalt build -d public
  artifacts:
    paths:
    - public/
  only:
  - master
```
