```console
$ cobalt -v build --destination _dest
DEBUG: Using config file `./_cobalt.yml`
Building from `.` into `[CWD]/_dest`
DEBUG: glob converted to regex: Glob { glob: "**/.*", re: "(?-u)^(?:/?|.*/)//.[^/]*$", opts: GlobOptions { case_insensitive: false, literal_separator: true, backslash_escape: true }, tokens: Tokens([RecursivePrefix, Literal('.'), ZeroOrMore]) }
DEBUG: glob converted to regex: Glob { glob: "**/_*", re: "(?-u)^(?:/?|.*/)_[^/]*$", opts: GlobOptions { case_insensitive: false, literal_separator: true, backslash_escape: true }, tokens: Tokens([RecursivePrefix, Literal('_'), ZeroOrMore]) }
DEBUG: glob converted to regex: Glob { glob: ".well-known/**/*", re: "(?-u)^//.well//-known(?:/|/.*/)[^/]*$", opts: GlobOptions { case_insensitive: false, literal_separator: true, backslash_escape: true }, tokens: Tokens([Literal('.'), Literal('w'), Literal('e'), Literal('l'), Literal('l'), Literal('-'), Literal('k'), Literal('n'), Literal('o'), Literal('w'), Literal('n'), RecursiveZeroOrMore, ZeroOrMore]) }
DEBUG: built glob set; 5 literals, 3 basenames, 1 extensions, 0 prefixes, 0 suffixes, 0 required extensions, 3 regexes
DEBUG: Loading data from `./_data`
DEBUG: glob converted to regex: Glob { glob: "**/.*", re: "(?-u)^(?:/?|.*/)//.[^/]*$", opts: GlobOptions { case_insensitive: false, literal_separator: true, backslash_escape: true }, tokens: Tokens([RecursivePrefix, Literal('.'), ZeroOrMore]) }
DEBUG: glob converted to regex: Glob { glob: "**/_*", re: "(?-u)^(?:/?|.*/)_[^/]*$", opts: GlobOptions { case_insensitive: false, literal_separator: true, backslash_escape: true }, tokens: Tokens([RecursivePrefix, Literal('_'), ZeroOrMore]) }
DEBUG: built glob set; 0 literals, 0 basenames, 0 extensions, 0 prefixes, 0 suffixes, 0 required extensions, 2 regexes
DEBUG: Loading snippets from `./_includes`
DEBUG: Copying `./.htaccess` to `[CWD]/_dest/.htaccess`
DEBUG: Copying `./.well-known/file` to `[CWD]/_dest/.well-known/file`
DEBUG: Copying `./some.js` to `[CWD]/_dest/some.js`
DEBUG: Copying `./style/blog.css` to `[CWD]/_dest/style/blog.css`
Build successful

```
