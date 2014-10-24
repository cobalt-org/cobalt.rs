# Rustie

A static site generator for (rust)[http://www.rust-lang.org/].

## Usage (not implemented yet)

### Layout / Design

You can have a custom layout in the ```_layout``` Directory.

### Posts

Posts live in ```_posts``` and are written in .toml format.

### Generate

For this given site layout:

    * ```path/to/repo/```
        * ```index.html```
        * ```_layouts/```
            * ```default.html```
            * ```post.html```
        * ```_posts/```
            * ```post_1.toml```
            * ```post_2.toml```

Rustie will generate:

    * ```path/to/repo/```
        * ```build/```
            * ```index.html```
            * ```posts/```
                * ```post_1.html```
                * ```post_2.html```

README will be completed soon...
