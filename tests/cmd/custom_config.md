```console
$ cobalt -v build --destination _dest
WARN: No _cobalt.yml file found in current directory, using default config.
...

$ cobalt -v build --config _cobalt_testing.yml --destination _dest
Building from `.` into `[CWD]/_dest`
DEBUG: glob converted to regex: Glob { glob: "**/.*", re: "(?-u)^(?:/?|.*/)//.[^/]*$", opts: GlobOptions { case_insensitive: false, literal_separator: true, backslash_escape: true, empty_alternates: false }, tokens: Tokens([RecursivePrefix, Literal('.'), ZeroOrMore]) }
DEBUG: glob converted to regex: Glob { glob: "**/_*", re: "(?-u)^(?:/?|.*/)_[^/]*$", opts: GlobOptions { case_insensitive: false, literal_separator: true, backslash_escape: true, empty_alternates: false }, tokens: Tokens([RecursivePrefix, Literal('_'), ZeroOrMore]) }
DEBUG: built glob set; 5 literals, 0 basenames, 0 extensions, 0 prefixes, 0 suffixes, 0 required extensions, 2 regexes
DEBUG: Loading data from `./_data`
DEBUG: glob converted to regex: Glob { glob: "**/.*", re: "(?-u)^(?:/?|.*/)//.[^/]*$", opts: GlobOptions { case_insensitive: false, literal_separator: true, backslash_escape: true, empty_alternates: false }, tokens: Tokens([RecursivePrefix, Literal('.'), ZeroOrMore]) }
DEBUG: glob converted to regex: Glob { glob: "**/_*", re: "(?-u)^(?:/?|.*/)_[^/]*$", opts: GlobOptions { case_insensitive: false, literal_separator: true, backslash_escape: true, empty_alternates: false }, tokens: Tokens([RecursivePrefix, Literal('_'), ZeroOrMore]) }
DEBUG: built glob set; 0 literals, 0 basenames, 0 extensions, 0 prefixes, 0 suffixes, 0 required extensions, 2 regexes
DEBUG: Loading snippets from `./_includes`
Build successful

```
