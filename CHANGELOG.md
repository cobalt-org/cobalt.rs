## 0.7.2 (2017-07-04)


#### Bug Fixes

*   Bump syntect version to 1.7.0 and enable dump-load feature ([4d0e14a7](https://github.com/cobalt-org/cobalt.rs/commit/4d0e14a788f02de98b63ae94ad976e02c6e8334c))



## 0.7.1 (2017-06-25)

#### Bug Fixes

* **CI:**
  *  Show all build failures, not just first ([52916cd8](https://github.com/cobalt-org/cobalt.rs/commit/52916cd8b448e84c691e4d0517b53b287ff56efb))
  *  Re-enable Stable builds on Linux ([24f3b209](https://github.com/cobalt-org/cobalt.rs/commit/24f3b2093580b26a0aef5c9ed81fb53f6fc614d7))

## 0.7.0 (2017-06-24)

#### Bug Fixes

* **tests:**
  *  Harden CLI tests ([0a73edd0](https://github.com/cobalt-org/cobalt.rs/commit/0a73edd0af1f147db8de410ce3f530902af92897))
  *  Improve error reporting ([0d379935](https://github.com/cobalt-org/cobalt.rs/commit/0d37993546f1f00733996fbb02719559a201bc5f))
  *  Make it easier to add new tests ([03fdecda](https://github.com/cobalt-org/cobalt.rs/commit/03fdecda29d0659b06eeb24e2817b1376dbe6581))
* **tests/cli/log_levels:**  Improve comparison error reporting ([334c4d2e](https://github.com/cobalt-org/cobalt.rs/commit/334c4d2e3a2153a2b93bd4a8453c49513283e69a))
* Stabalize the CI ([5d739b5](https://github.com/cobalt-org/cobalt.rs/commit/5d739b50ddfdf90fb848921681d46d4a4b7e20f6))
* Upgrade to [liquid-rust 0.10.0](https://github.com/cobalt-org/liquid-rust/releases/tag/v0.10.0) ([2421679](https://github.com/cobalt-org/cobalt.rs/commit/24216795b6c83acff98de1a1fad22f54a00150bb))
* `cobalt serve` should ignore query strings ([eb9e0b0](https://github.com/cobalt-org/cobalt.rs/commit/eb9e0b05596313ee1213b20ed642777ee0a34139))
* Gracefully handle empty frontmatters ([5aa5813](https://github.com/cobalt-org/cobalt.rs/commit/5aa5813479c00683e321def73a8b2e6cbc14fa9e))

#### Performance

* **document:**  cache off Regex objects ([2b2525c1](https://github.com/cobalt-org/cobalt.rs/commit/2b2525c17808a0e4c4bd4627018d06599018c5fd))

#### Features

* More advanced blacklisting, now with whitelisting! ([be05d963](https://github.com/cobalt-org/cobalt.rs/commit/be05d963c7541357a01c9e437122d2b59dea27d8), closes [#221](https://github.com/cobalt-org/cobalt.rs/issues/221))
* **attributes:**  Add input-file based attributes ([95fb81f5](https://github.com/cobalt-org/cobalt.rs/commit/95fb81f52b7bfbace6d806cf526629b854e2db4e))
* **debug:**  Dump intermediate state ([fec65b37](https://github.com/cobalt-org/cobalt.rs/commit/fec65b373ab11189b6091fea52adbffeebdfdc4b))
* Upgrade to [liquid-rust 0.10.0](https://github.com/cobalt-org/liquid-rust/releases/tag/v0.10.0) ([2421679](https://github.com/cobalt-org/cobalt.rs/commit/24216795b6c83acff98de1a1fad22f54a00150bb))
* `cobalt new` added to create pages (former `cobalt new` renamed to `cobalt init`) ([4700f34](https://github.com/cobalt-org/cobalt.rs/commit/4700f3411d711548a85338598f507af80f65dabe))
* Customize post sort order with `post_order: "asc"` ([28df5e2](https://github.com/cobalt-org/cobalt.rs/commit/28df5e2903b335cb75a17982decf640a1843e43f))
* Posts now have `prev` and `next` attributes ([dbcaf7e](https://github.com/cobalt-org/cobalt.rs/commit/dbcaf7e864e8b95001b105862289cac137c80877))
* Documents now have `title`, `slug`, and `ext` attributes ([46b3b22](https://github.com/cobalt-org/cobalt.rs/commit/46b3b22f9e6877fc058fdfa2154f3a7c49bd0305))

#### Breaking Changes

* `.cobalt.yml`'s `ignore` changed to `gitingore` format ([be05d963](https://github.com/cobalt-org/cobalt.rs/commit/be05d963c7541357a01c9e437122d2b59dea27d8), closes [#221](https://github.com/cobalt-org/cobalt.rs/issues/221))
* `cobalt new` renamed to `cobalt init` ([fe3a246](https://github.com/cobalt-org/cobalt.rs/commit/fe3a246d74b3e01e40e2873b97e9398d3264e8e7))

## 0.5

- We now generate binaries for OSX, Windows, and Linux ([@johannhof][])
- Experimental Syntax Highlighting support (behind a compiler flag until we fix Windows support)([@gnunicorn][])
- Prevent `cargo clean` from deleting the current directory ([@kracekumar][])
- Set charset utf-8 in the default template ([@nott][])
- tons of other bugfixes and improvements (all of the above + [@benaryorg][], [@crodjer][] and [@uetoyo][])

## 0.4

- Improved Windows support ([@johannhof][])
- Migrated from getopt to clap, global parameters are now available in subcommands ([@jespino][])
- Added draft support ([@johannhof][])
- Added `content` and `excerpt` attributes to posts ([@johannhof][] and [@nott][])
- Added `guid` tag to RSS ([@nott][])
- Added `cobalt clean` ([@rjgoldsborough][])
- tons of other bugfixes and improvements (all of the above + [@LucioFranco][])

## 0.3

- Added `cobalt serve` ([@tak1n][] and [@DonRyuDragoni][])
- Added `cobalt watch` ([@LucioFranco][])
- Added `cobalt new` ([@LucioFranco][])
- Added `cobalt build --import` ([@LucioFranco][])
- Moved from _posts to posts ([@johannhof][])
- Ignore underscored directories by default ([@johannhof][])
- Added an `ignore` attribute to .cobalt.yml ([@jespino][])
- Implemented custom paths (permalinks) ([@johannhof][])
- Removed the .tpl file extension in favor of .liquid ([@tak1n][])
- tons of other bugfixes and improvements (all of the above + [@kstep][])

## 0.2

- Initial release

[@DonRyuDragoni]: https://github.com/DonRyuDragoni
[@LucioFranco]: https://github.com/LucioFranco
[@jespino]: https://github.com/jespino
[@johannhof]: https://github.com/johannhof
[@kstep]: https://github.com/kstep
[@nott]: https://github.com/nott
[@rjgoldsborough]: http://github.com/rjgoldsborough
[@tak1n]: https://github.com/tak1n
