```console
$ cobalt
? failed
cobalt-bin 0.17.5
Static site generator

USAGE:
    cobalt[EXE] [OPTIONS] <SUBCOMMAND>

OPTIONS:
        --color <WHEN>    Controls when to use color [default: auto]
    -h, --help            Print help information
    -q, --quiet           Pass many times for less log output
    -v, --verbose         Pass many times for more log output
    -V, --version         Print version information

SUBCOMMANDS:
    build      Build the cobalt project at the source dir
    clean      Cleans `destination` directory
    debug      Print site debug information
    help       Print this message or the help of the given subcommand(s)
    init       Create a document
    new        Create a document
    publish    Publish a document
    rename     Rename a document
    serve      Build, serve, and watch the project at the source dir

$ cobalt --non-existent
? failed
error: Found argument '--non-existent' which wasn't expected, or isn't valid in this context

	If you tried to supply `--non-existent` as a value rather than a flag, use `-- --non-existent`

USAGE:
    cobalt[EXE] [OPTIONS] <SUBCOMMAND>

For more information try --help

```
