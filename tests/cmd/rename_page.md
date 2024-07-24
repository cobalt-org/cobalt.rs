```console
$ cobalt -v rename my-new-special-page.md "New and Improved"
DEBUG: Using config file `./_cobalt.yml`
DEBUG: Draft mode enabled
DEBUG: glob converted to regex: Glob { glob: "**/.*", re: "(?-u)^(?:/?|.*/)//.[^/]*$", opts: GlobOptions { case_insensitive: false, literal_separator: true, backslash_escape: true, empty_alternates: false }, tokens: Tokens([RecursivePrefix, Literal('.'), ZeroOrMore]) }
DEBUG: glob converted to regex: Glob { glob: "**/_*", re: "(?-u)^(?:/?|.*/)_[^/]*$", opts: GlobOptions { case_insensitive: false, literal_separator: true, backslash_escape: true, empty_alternates: false }, tokens: Tokens([RecursivePrefix, Literal('_'), ZeroOrMore]) }
DEBUG: built glob set; 6 literals, 0 basenames, 0 extensions, 0 prefixes, 0 suffixes, 0 required extensions, 2 regexes
WARN: Renaming a published page might invalidate links

$ cobalt -v build
DEBUG: Using config file `./_cobalt.yml`
Building from `.` into `./_site`
DEBUG: glob converted to regex: Glob { glob: "**/.*", re: "(?-u)^(?:/?|.*/)//.[^/]*$", opts: GlobOptions { case_insensitive: false, literal_separator: true, backslash_escape: true, empty_alternates: false }, tokens: Tokens([RecursivePrefix, Literal('.'), ZeroOrMore]) }
DEBUG: glob converted to regex: Glob { glob: "**/_*", re: "(?-u)^(?:/?|.*/)_[^/]*$", opts: GlobOptions { case_insensitive: false, literal_separator: true, backslash_escape: true, empty_alternates: false }, tokens: Tokens([RecursivePrefix, Literal('_'), ZeroOrMore]) }
DEBUG: built glob set; 6 literals, 0 basenames, 0 extensions, 0 prefixes, 0 suffixes, 0 required extensions, 2 regexes
DEBUG: Loading data from `./_data`
DEBUG: glob converted to regex: Glob { glob: "**/.*", re: "(?-u)^(?:/?|.*/)//.[^/]*$", opts: GlobOptions { case_insensitive: false, literal_separator: true, backslash_escape: true, empty_alternates: false }, tokens: Tokens([RecursivePrefix, Literal('.'), ZeroOrMore]) }
DEBUG: glob converted to regex: Glob { glob: "**/_*", re: "(?-u)^(?:/?|.*/)_[^/]*$", opts: GlobOptions { case_insensitive: false, literal_separator: true, backslash_escape: true, empty_alternates: false }, tokens: Tokens([RecursivePrefix, Literal('_'), ZeroOrMore]) }
DEBUG: built glob set; 0 literals, 0 basenames, 0 extensions, 0 prefixes, 0 suffixes, 0 required extensions, 2 regexes
DEBUG: Loading snippets from `./_includes`
DEBUG: Creating RSS file at ./_site/rss.xml
Build successful

```
