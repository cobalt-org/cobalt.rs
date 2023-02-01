```console
$ cobalt
? failed
Static site generator

Usage: cobalt[EXE] [OPTIONS] <COMMAND>

Commands:
  init     Create a document
  new      Create a document
  rename   Rename a document
  publish  Publish a document
  build    Build the cobalt project at the source dir
  clean    Cleans `destination` directory
  serve    Build, serve, and watch the project at the source dir
  debug    Print site debug information
  help     Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...    More output per occurrence
  -q, --quiet...      Less output per occurrence
      --color <WHEN>  Controls when to use color [default: auto] [possible values: auto, always,
                      never]
  -h, --help          Print help
  -V, --version       Print version

$ cobalt --non-existent
? failed
error: unexpected argument '--non-existent' found

Usage: cobalt[EXE] [OPTIONS] <COMMAND>

For more information, try '--help'.

```
