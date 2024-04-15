Clean no-op
```console
$ cobalt -v clean
WARN: No _cobalt.yml file found in current directory, using default config.
DEBUG: No `./_site` to clean
DEBUG: [..]

```

Clean built site
```console
$ cobalt -qqq build

$ cobalt -v clean
WARN: No _cobalt.yml file found in current directory, using default config.
directory `[CWD]/_site` removed

```
