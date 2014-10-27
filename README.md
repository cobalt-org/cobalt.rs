# Cobalt

A static site generator written in [rust](http://www.rust-lang.org/).

Note: This is my first rust project, so it's possible that you wanna shoot me down for the code you will see here...

## Usage

### Layout / Design

You can have a custom layout in the ```_layouts``` Directory.

Same as Post files Layout files are written as [mustache](https://github.com/erickt/rust-mustache) templates.

### Posts

Posts live in ```_posts``` and are written in .tpl format and use mustache under the hood.

Example:

```text
@extends: posts.tpl

title:   My first Blogpost
date:    24/08/2014 at 15:36
---
Hey there this is my first blogpost and this is super awesome.

My Blog is lorem ipsum like, yes it is..
```

The content before ```---``` are meta attributes and accessible via their key.

Via ```@extends``` attribute you speficy which layout will be used.

In the specific layout file title is accessible via ```{{ title }}``` and the date via ```{{ date }}``` (Attributes are also accessible in the post content itself).

Also there is one standard attribute which is named statically - ```{{ content }}``` which is the whole text under the ```---``` block.


### Generate

For this given site layout:

    * path/to/repo/
        * index.tpl
        * _layouts/
            * default.tpl
            * posts.tpl
        * _posts/
            * 2014-08-24-my-first-blogpost.tpl
            * 2014-09-05-my-second-blogpost.tpl

Cobalt will generate:

    * path/to/repo/
        * build/
            * index.html
            * posts/
                * 2014-08-24-my-first-blogpost.html
                * 2014-09-05-my-second-blogpost.html

README will be completed soon...
