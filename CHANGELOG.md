# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->
## [Unreleased] - ReleaseDate

## [0.17.4] - 2021-09-14

## [0.17.3] - 2021-09-14

## [0.17.2] - 2021-09-14

## [0.17.1] - 2021-09-13

## [0.17.0] - 2021-09-13

#### Features

- Allow disabling of templating
- vimwiki support

#### Bug Fixes

- Ensure we ignore `--destination` when building
- `cobalt new` creates dir if needed

## [0.16.5] - 2021-05-29

#### Bug Fixes

- Fix XML syntax for RSS
- Resolve infinite loop on Windows with `cobalt serve`

#### Features

- Minification support for HTML, CSS, and JS
- Strike through and task list support in Markdown

#### Performance

- Speed up liquid rendering

## 0.16.4 - 2020-10-17

#### Bug Fixes

- Invalid syntax highlighting in some cases (#802)

## 0.16.2 - 2020-08-03

#### Bug Fixes

- **new**: Create a valid `published_date`

## 0.16.1 - 2020-07-28

#### Bug Fixes

- **serv**: Provide content type for 404

## 0.16.0 - 2020-07-15

#### Features

- `where` filter

#### Bug Fixes

- Fixed liquid corner cases to be more conformant

#### Breaking Changes

- `{% include %}` is now strictly Liquid-style rather
  than Jekyll-style. It takes an expression rather than a bare-word for
  the path to include.

## 0.15.11 - 2020-03-09

#### Features

* Support opening the page in the browser with `cobalt serve --open`.

## 0.15.10 - 2020-02-24

#### Bug Fixes

* Report mime-type with `cobalt serve` (closes [#732](https://github.com/cobalt-org/cobalt.rs/issues/732)).
* Fix RSS content when excerpts are disabled (closes [#724](https://github.com/cobalt-org/cobalt.rs/issues/724)).

## 0.15.9 - 2020-02-11

#### Bug Fixes

* Provide a pre-built windows binary

## 0.15.7 - 2020-02-10

#### Bug Fixes

* Fixed "ctrl-c" during `cobalt serve` so that it will not hang.

## 0.15.6 - 2019-11-28


#### Bug Fixes

*   Fix windows support ([e5686a07](https://github.com/cobalt-org/cobalt.rs/commit/e5686a07afdbcdb75456be3831e031085f00acdf))
* **serve:**  All new implementation ([eb454576](https://github.com/cobalt-org/cobalt.rs/commit/eb454576f5ef0000e9ad2b4bba491b1a3116f13a))



## 0.15.5 - 2019-11-27


#### Bug Fixes

* **liquid:**  Upgrade to 0.19 ([95c85bbb](https://github.com/cobalt-org/cobalt.rs/commit/95c85bbb40e19c9cda30f9f205b78fae69f3a361), closes [#656](https://github.com/cobalt-org/cobalt.rs/issues/656))
* **md:**  Be more conformant ([3841da3d](https://github.com/cobalt-org/cobalt.rs/commit/3841da3db6a85b0568b608679948036c64c650a4))
* **pagination:**
  * revamp pagination permalink management ([e041aad2](https://github.com/cobalt-org/cobalt.rs/commit/e041aad29214a77d4e2326af4755ba2169908eb8))
  * fixes #666 and adjust tests ([9621b31a](https://github.com/cobalt-org/cobalt.rs/commit/9621b31a7cacb2ba3aa4d16c5e552c955d0778c1))

#### Features

*   Add support for sitemap generation ([9b6272d6](https://github.com/cobalt-org/cobalt.rs/commit/9b6272d6b494a7ef4a10990f170b247e48de3399))
* **liquid-jekyll-filter:**  activate jekyll filters ([5c7c568f](https://github.com/cobalt-org/cobalt.rs/commit/5c7c568fe3f4c48a5cbf4621ab3e8468689020f7))
* **pagination:**
  *  Sort by weight ([d197c652](https://github.com/cobalt-org/cobalt.rs/commit/d197c652ec1cff98a8521114448cb1d2320ed01a))
  *  pagination on dates ([4327ec1a](https://github.com/cobalt-org/cobalt.rs/commit/4327ec1a7b8096ce0b07a79576825b5f41efb3cb))
  *  indexation by categories ([2c116ae9](https://github.com/cobalt-org/cobalt.rs/commit/2c116ae922274f5496ec9a0c1770a11bcb35a23f))



## 0.15.3 - 2019-01-31


#### Bug Fixes

* **files:**  sort by filename to have a reproducibly walking iterator ([1aa7510b](https://github.com/cobalt-org/cobalt.rs/commit/1aa7510bb742947e1786d5a3e56eecccff126985))



## 0.15.2 - 2019-01-29


#### Features

* **experimental:**  pagination on tags ([1a968352](https://github.com/cobalt-org/cobalt.rs/commit/1a9683524bbc8450dc46c5ddd11f8edfbf794746))
* **publish:**  move post to `posts` if in `drafts_dir` ([c21ca310](https://github.com/cobalt-org/cobalt.rs/commit/c21ca3103357f42c9d8a94f3edb88d4481e85f47))



## 0.15.1 - 2019-01-24


#### Features

* **publish:**  Auto-prepend date ([95839ac5](https://github.com/cobalt-org/cobalt.rs/commit/95839ac51bfae82a7124b92d774de38088d500ca), closes [#562](https://github.com/cobalt-org/cobalt.rs/issues/562))

#### Bug Fixes

* **liquid:**  Raw/comment improved ([fe795248](https://github.com/cobalt-org/cobalt.rs/commit/fe795248baad055f0f20640f82967c84ba2fd8d9))



## 0.15.0 - 2018-12-28


#### Bug Fixes

* **new:**  Generate files using jekyll frontmatter format ([bb3b31ae](https://github.com/cobalt-org/cobalt.rs/commit/bb3b31aeb6f72059d327f9fc80c219626546c742))

#### Breaking Changes

* **liquid:**  Upgrade ([b9981489](https://github.com/cobalt-org/cobalt.rs/commit/b9981489ffec4e39a6e5b2e61a45aedd75911fba), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))

#### Performance

* **liquid:**  Upgrade ([b9981489](https://github.com/cobalt-org/cobalt.rs/commit/b9981489ffec4e39a6e5b2e61a45aedd75911fba), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))

#### Features

* **liquid:**  Upgrade ([b9981489](https://github.com/cobalt-org/cobalt.rs/commit/b9981489ffec4e39a6e5b2e61a45aedd75911fba), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))
* **tags:**  support `tags` in frontmatter ([59db868a](https://github.com/cobalt-org/cobalt.rs/commit/59db868a393f2e0fd1d4a818a7e0c3eca9612210))



## 0.14.0 - 2018-11-18


#### Performance

*   Switch to system allocator ([c0db5ac1](https://github.com/cobalt-org/cobalt.rs/commit/c0db5ac1da9282d352ba9950f6d01871d373fd73))

#### Features

*   Jekyll-style frontmatter divider ([ad62b2fd](https://github.com/cobalt-org/cobalt.rs/commit/ad62b2fddec662d202cd59ad702650a62935f9dd), closes [#431](https://github.com/cobalt-org/cobalt.rs/issues/431))
* **liquid:**  ([4ef1eded](https://github.com/cobalt-org/cobalt.rs/commit/4ef1eded4968eb1b8956b166c164d95077885e50), [a1d0006f](https://github.com/cobalt-org/cobalt.rs/commit/a1d0006fef22cc07051cfc47f80e810b9312292a))
  * Index by variables
  * `for` block parameters can be variables
  * New filters: `at_most`, `at_least`, `push`, `pop`, `unshift`, `shift`, `array_to_sentence_string`
  * New tags: `tablerow`, `ifchanged`, `increment`, `decerement`
  * Slightly improved error reporting
  * Arrays now have `.first` and `.last` variables
  * `if` conditions support `or`/`and`
* **pagination (prototype):**
  *  core logic ([da6360d2](https://github.com/cobalt-org/cobalt.rs/commit/da6360d223d5e9ac80516b66b9cb0c43bc6dff91))
  *  pagination frontmatter ([f35eec22](https://github.com/cobalt-org/cobalt.rs/commit/f35eec22618f84ab0222afc90130c5e8cb666d21))

#### Breaking Changes

* Empty frontmatters followed by `---` will no longer build ([ad62b2fd](https://github.com/cobalt-org/cobalt.rs/commit/ad62b2fddec662d202cd59ad702650a62935f9dd), closes [#431](https://github.com/cobalt-org/cobalt.rs/issues/431))
* **liquid:**  `for` block ranges are now inclusive ([4ef1eded](https://github.com/cobalt-org/cobalt.rs/commit/4ef1eded4968eb1b8956b166c164d95077885e50))

#### Bug Fixes

* **liquid:**  ([4ef1eded](https://github.com/cobalt-org/cobalt.rs/commit/4ef1eded4968eb1b8956b166c164d95077885e50), [a1d0006f](https://github.com/cobalt-org/cobalt.rs/commit/a1d0006fef22cc07051cfc47f80e810b9312292a))
  * `for` looping over a range is now inclusive to align with shopify liquid
  * deeply nested array indexes work again (`a.b[0]`)

## 0.13.2 - 2018-10-05


#### Features

* **liquid:**  filter input can index ([5a1eb91a](https://github.com/cobalt-org/cobalt.rs/commit/5a1eb91acd9fe3954949b24ad80588df2f1d636a))



## 0.13.0 - 2018-10-04


#### Features

*   Liquid for-block works on hashes ([68ca7b25](https://github.com/cobalt-org/cobalt.rs/commit/68ca7b25e9a6d946b69718a3946e83cf50bd9f98))

#### Breaking Changes

* **jekyll:**  Split out migration ([673a4cf6](https://github.com/cobalt-org/cobalt.rs/commit/673a4cf6334c7d5d1b5c04ce9da13fc6d62902a2), closes [#438](https://github.com/cobalt-org/cobalt.rs/issues/438), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))



## 0.12.2 - 2018-07-21


#### Features

* **sass:**
  *  Enable by default ([5cd0f120](https://github.com/cobalt-org/cobalt.rs/commit/5cd0f12006937a08090b5d2177a2839c06479241))
  *  add support for Sass's indented syntax ([8ca7cb81](https://github.com/cobalt-org/cobalt.rs/commit/8ca7cb819af8308fc765d9e5d8415ec88de23af9))
*   add runtime configuration to built in syntax highlighting ([3f73c817](https://github.com/cobalt-org/cobalt.rs/commit/3f73c81748070e690f8e9a3a90b8600bbcfc0cca))



## 0.12.1 - 2018-03-24

No user-visible change; just working around CI problems.


## 0.12.0 - 2018-03-22

This release drops support for migrating from pre-0.11.  Please use 0.11 to migrate first.

#### Performance

* **layouts:**  Switch from lazy to eager loading ([b2cbe13a](https://github.com/cobalt-org/cobalt.rs/commit/b2cbe13aaa1ee25b4f7710230a2a6c350e545381))

#### Features

* **`new`:**
  *  Load default files from disk ([3382901c](https://github.com/cobalt-org/cobalt.rs/commit/3382901c15634191e7358bbeb3e9c83cd18a97f0), closes [#355](https://github.com/cobalt-org/cobalt.rs/issues/355))
  *  `--with-ext` to specify type without `--file` ([1fd83fce](https://github.com/cobalt-org/cobalt.rs/commit/1fd83fce544b2178f4f6e9b4438a8f5c5b861f84))
* New `rename` subcommand ([1fee0775](https://github.com/cobalt-org/cobalt.rs/commit/1fee0775295b644c68f5464696ecc5ffaf3e2342), closes [#393](https://github.com/cobalt-org/cobalt.rs/issues/393))
* **liquid:**
  * Add `page.slug` ([46de52e5](https://github.com/cobalt-org/cobalt.rs/commit/46de52e502c51c6f525fe5318a29a7c4c23f06b9))
  * Basic `compact` support ([8fca4340](https://github.com/cobalt-org/cobalt.rs/commit/8fca43406de568f9d163e70077f25267cc1b46a0))
  * Whole number (integer) support ([8fca4340](https://github.com/cobalt-org/cobalt.rs/commit/8fca43406de568f9d163e70077f25267cc1b46a0))
  * Provide context on errors ([8fca4340](https://github.com/cobalt-org/cobalt.rs/commit/8fca43406de568f9d163e70077f25267cc1b46a0), closes [#136](https://github.com/cobalt-org/cobalt.rs/issues/136))
* **`serve`:** Custom host support ([e654541](https://github.com/cobalt-org/cobalt.rs/commit/e654541dae4a9ef2f74ff115fcc5f5eb16bcddb5))
* **`debug`:**
  *  Print pages and posts files ([29d5a74e](https://github.com/cobalt-org/cobalt.rs/commit/29d5a74e4438744332f2719de7b6ae18578c16e0))
  *  Dump config ([b5ac9c5c](https://github.com/cobalt-org/cobalt.rs/commit/b5ac9c5c0f42dafc84fb991f91aed0fce5f5c3c2))
* More details available with `--trace` ([0ed5247](https://github.com/cobalt-org/cobalt.rs/commit/0ed5247e896b9b152ef3f24196e49e499fb5ddae))

#### Breaking Changes

* **`migrate`:**  Removing migration support ([20b29932](https://github.com/cobalt-org/cobalt.rs/commit/20b29932f794b17fe71b467fb3d05901982cc331))
* **`import`:**  `import` is self-contained ([4e0be270](https://github.com/cobalt-org/cobalt.rs/commit/4e0be270aaaef92a4b9638b170a3acfcd96ae0a3), closes [#394](https://github.com/cobalt-org/cobalt.rs/issues/394))
* **layouts:**  Switch from lazy to eager ([b2cbe13a](https://github.com/cobalt-org/cobalt.rs/commit/b2cbe13aaa1ee25b4f7710230a2a6c350e545381))
* **liquid:**
  * Whole number (integer) support ([8fca4340](https://github.com/cobalt-org/cobalt.rs/commit/8fca43406de568f9d163e70077f25267cc1b46a0))
  * Improve value coercion ([8fca4340](https://github.com/cobalt-org/cobalt.rs/commit/8fca43406de568f9d163e70077f25267cc1b46a0))

#### Bug Fixes

* **`clean`:**  Don't error on double-clean ([a380fd15](https://github.com/cobalt-org/cobalt.rs/commit/a380fd15fb400eb1074a06e5d779345154d7b3be))
* **`import`:** `import` is self-contained ([4e0be270](https://github.com/cobalt-org/cobalt.rs/commit/4e0be270aaaef92a4b9638b170a3acfcd96ae0a3), closes [#394](https://github.com/cobalt-org/cobalt.rs/issues/394))
* **`new`:**
  *  Dont assume dirs are files ([35bed0c5](https://github.com/cobalt-org/cobalt.rs/commit/35bed0c5e2461176ff90689e793d08b1924747f5), closes [#401](https://github.com/cobalt-org/cobalt.rs/issues/401))
  *  Error if file isn't a part of a collection ([99a0d017](https://github.com/cobalt-org/cobalt.rs/commit/99a0d017ce5260ac27d23665d39f6b389dced729))
* **`serve`:** Fix site.base_url by adding http:// ([1a3ae383](https://github.com/cobalt-org/cobalt.rs/commit/1a3ae383910cd799605567db6182244b33bd098c))
* **slug:**  Dont ignore non-ascii in slug ([08f0621c](https://github.com/cobalt-org/cobalt.rs/commit/08f0621cd94fb4be0cde9c999a4a5b2d72691864), closes [#383](https://github.com/cobalt-org/cobalt.rs/issues/383))
* **liquid:**
  * `date_in_tz` correctly parse date strings ([8fca4340](https://github.com/cobalt-org/cobalt.rs/commit/8fca43406de568f9d163e70077f25267cc1b46a0))
  * Improve value coercion ([8fca4340](https://github.com/cobalt-org/cobalt.rs/commit/8fca43406de568f9d163e70077f25267cc1b46a0))
  * `if` can do existence check again ([0ed5247](https://github.com/cobalt-org/cobalt.rs/commit/0ed5247e896b9b152ef3f24196e49e499fb5ddae))
* **config**: Bug fixes with `ignore` ([0ed5247](https://github.com/cobalt-org/cobalt.rs/commit/0ed5247e896b9b152ef3f24196e49e499fb5ddae))


## 0.11.1 - 2018-01-10


#### Features

* **liquid:**  Support contains operator ([668f4726](https://github.com/cobalt-org/cobalt.rs/commit/668f47260d1e7db7666a77d8c100997da2531dee))



## 0.11.0 - 2018-01-09

This release focused on unleashing a lot of breaking changes that have been
stacking up for a while which also expose a lot of features that have been
inaccessible.  The hope is that from now on, breaking changes will be minor
(like small changes to config) rather than sweeping changes to every file like
this.

#### Bug Fixes

*   Reducing logging noise ([a7acd2c8](https://github.com/cobalt-org/cobalt.rs/commit/a7acd2c858a12e92ae80d8c8ed0bbbd64aa84824))
* **rss:**  Don't error if the RSS folder doesn't exist. ([357cb4b8](https://github.com/cobalt-org/cobalt.rs/commit/357cb4b8e97c7b37af84164ca4380faf4de8c3ab))
* **watch:**  Don't stop on error ([3c4d086b](https://github.com/cobalt-org/cobalt.rs/commit/3c4d086bfa39dd3ce5d9c446c5602d53e5ffa2c9), closes [#347](https://github.com/cobalt-org/cobalt.rs/issues/347))

#### Features

*   Migrate support for changing _layouts to _includes ([28ae870d](https://github.com/cobalt-org/cobalt.rs/commit/28ae870dab175264a87d1b1020bcbf23c85a60c1))
* **config:**
  *  Change .cobalt.yml to _cobalt.yml ([c4ee83b3](https://github.com/cobalt-org/cobalt.rs/commit/c4ee83b3f5f60410c3e53cc8d14271d9e8c0f42f), closes [#348](https://github.com/cobalt-org/cobalt.rs/issues/348))
  *  Stablize the format ([34e9d545](https://github.com/cobalt-org/cobalt.rs/commit/34e9d545b03e5696afd05a5922fcee49eacf5ec2), closes [#199](https://github.com/cobalt-org/cobalt.rs/issues/199), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/), [#](https://github.com/cobalt-org/cobalt.rs/issues/))
* **front:**
  *  Stablize fronmatter format ([9089c721](https://github.com/cobalt-org/cobalt.rs/commit/9089c721a2910152b6685b497da8af26a37b64e8), closes [#257](https://github.com/cobalt-org/cobalt.rs/issues/257), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))
  *  Change date to YYYY-MM-DD ([1e19ae07](https://github.com/cobalt-org/cobalt.rs/commit/1e19ae070dcde429c86212cb6c39e5298c841c92), closes [#349](https://github.com/cobalt-org/cobalt.rs/issues/349), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))
  *  Change permalink variable names ([e78b806c](https://github.com/cobalt-org/cobalt.rs/commit/e78b806c9561d8ad9bd3b31b5ab161ea2e79faa8), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))
  *  Change permalink to well-defined format ([c6c4d7ac](https://github.com/cobalt-org/cobalt.rs/commit/c6c4d7aca4e25820a7bf670a38d8c6023c902d5a), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))
  *  Standardize permalink behavior ([6730eb68](https://github.com/cobalt-org/cobalt.rs/commit/6730eb686d083b12c4307a25875ed0a9b7034236), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))
* **excerpt:**  Better define non-existent behavior ([129c5747](https://github.com/cobalt-org/cobalt.rs/commit/129c5747352c9aada646d738754f2d7477d5d2c8))
* **page:**
  *  Upgrade liquid ([2ec3f24b](https://github.com/cobalt-org/cobalt.rs/commit/2ec3f24bb8eb3e7b09eb68209fa39430760d1d18))
  *  Generalize is_post / posts ([d280a353](https://github.com/cobalt-org/cobalt.rs/commit/d280a3532958fd1c973c1f7e6bd0b725eca9e102), closes [#323](https://github.com/cobalt-org/cobalt.rs/issues/323), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))
  *  Make page variables future-proof ([6f62dea8](https://github.com/cobalt-org/cobalt.rs/commit/6f62dea85679a23b7179f26573e8f26132fc32c6), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))
  *  page.file.parent variable ([dce1d59c](https://github.com/cobalt-org/cobalt.rs/commit/dce1d59c31d99e249db9b0b9334049905600a64a), closes [#338](https://github.com/cobalt-org/cobalt.rs/issues/338))
* **serve:**
  *  Adjust base_url for localhost ([e75e1398](https://github.com/cobalt-org/cobalt.rs/commit/e75e139880964c0abc1668c7c70e2da71004ff55), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))
  *  Merge `serve` and `watch` ([d2f22d51](https://github.com/cobalt-org/cobalt.rs/commit/d2f22d51fe2f7e3dbe729069f8eaa2719880d86f), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))
* **debug:**
  *  Report asset files ([5d77b7fc](https://github.com/cobalt-org/cobalt.rs/commit/5d77b7fc1a08bf2994a599e852d0cf6462a7431d))
  *  Generalize debug commands ([087d9919](https://github.com/cobalt-org/cobalt.rs/commit/087d99195ae66525445e5ce91246e0757e945214), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))

#### Breaking Changes

`cobalt migrate` was created to help mitigate the cost of most of these breaking changes.

* **config:**  Stablize the format ([34e9d545](https://github.com/cobalt-org/cobalt.rs/commit/34e9d545b03e5696afd05a5922fcee49eacf5ec2), closes [#199](https://github.com/cobalt-org/cobalt.rs/issues/199), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/), [#](https://github.com/cobalt-org/cobalt.rs/issues/))
* **debug:**  Generalize debug commands ([087d9919](https://github.com/cobalt-org/cobalt.rs/commit/087d99195ae66525445e5ce91246e0757e945214), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))
* **front:**
  *  Change date to YYYY-MM-DD ([1e19ae07](https://github.com/cobalt-org/cobalt.rs/commit/1e19ae070dcde429c86212cb6c39e5298c841c92), closes [#349](https://github.com/cobalt-org/cobalt.rs/issues/349), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))
  *  Change permalink variable names ([e78b806c](https://github.com/cobalt-org/cobalt.rs/commit/e78b806c9561d8ad9bd3b31b5ab161ea2e79faa8), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))
  *  Change permalink to well-defined format ([c6c4d7ac](https://github.com/cobalt-org/cobalt.rs/commit/c6c4d7aca4e25820a7bf670a38d8c6023c902d5a), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))
  *  Standardize permalink behavior ([6730eb68](https://github.com/cobalt-org/cobalt.rs/commit/6730eb686d083b12c4307a25875ed0a9b7034236), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))
  *  Stablize fronmatter format ([9089c721](https://github.com/cobalt-org/cobalt.rs/commit/9089c721a2910152b6685b497da8af26a37b64e8), closes [#257](https://github.com/cobalt-org/cobalt.rs/issues/257), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))
* **page:**
  *  Liquid errors on undefined variables ([2ec3f24b](https://github.com/cobalt-org/cobalt.rs/commit/2ec3f24bb8eb3e7b09eb68209fa39430760d1d18))
    * This was done to help catch migration problems and to move us in the direction of easier debugging of problems
    * The restriction might be loosened in some cases (like `{% if var %}`).
  *  Generalize is_post / posts ([d280a353](https://github.com/cobalt-org/cobalt.rs/commit/d280a3532958fd1c973c1f7e6bd0b725eca9e102), closes [#323](https://github.com/cobalt-org/cobalt.rs/issues/323), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))
  *  Make page variables future-proof ([6f62dea8](https://github.com/cobalt-org/cobalt.rs/commit/6f62dea85679a23b7179f26573e8f26132fc32c6), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))
* **serve:**
  *  Adjust base_url for localhost ([e75e1398](https://github.com/cobalt-org/cobalt.rs/commit/e75e139880964c0abc1668c7c70e2da71004ff55), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))
  *  Merge `serve` and `watch` ([d2f22d51](https://github.com/cobalt-org/cobalt.rs/commit/d2f22d51fe2f7e3dbe729069f8eaa2719880d86f), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))

#### Performance

* **serve:**  Reduce duplicate rebuilds ([8f2679ce](https://github.com/cobalt-org/cobalt.rs/commit/8f2679ced60013c9cabd40bd04f5123e4132764e))



## 0.10.0 - 2018-01-04


#### Features

* **error:**  Identify layout errors ([0ec3a3d0](https://github.com/cobalt-org/cobalt.rs/commit/0ec3a3d0fbab0d51e20304d3e1a5726e987e36da))

#### Bug Fixes

*   Correctly copy scss files when sass is disabled ([f04bd9d2](https://github.com/cobalt-org/cobalt.rs/commit/f04bd9d2f38daf988c3bf244c56fe3ff97c3a420))
*   Log context for failures ([be258bfb](https://github.com/cobalt-org/cobalt.rs/commit/be258bfb31bb5a0d7131469eba27b13b82a5256f))
* **config:**  On empty file, use right root ([a99ca197](https://github.com/cobalt-org/cobalt.rs/commit/a99ca197564cead19c247ca91429077b163cd794))
* **drafts:**  Config can enable again ([729e0b18](https://github.com/cobalt-org/cobalt.rs/commit/729e0b18fdc9ad5ec173c73d015ff8cb0364a675))
* **front:**  Ensure excerpt is rendered before used ([9e714697](https://github.com/cobalt-org/cobalt.rs/commit/9e7146979e8df4afaa9bdc1890fe725b0b551bf7))
* **includes:**  Support including hidden files ([cb577c42](https://github.com/cobalt-org/cobalt.rs/commit/cb577c42624720740e382c9336510807f67ca0ca))

#### Breaking Changes

* **error:**  Identify layout errors ([0ec3a3d0](https://github.com/cobalt-org/cobalt.rs/commit/0ec3a3d0fbab0d51e20304d3e1a5726e987e36da))

## 0.9.0 - 2017-11-30


#### Features

* **liquid:**  Dedicated _includes dir ([dc4b9cef](https://github.com/cobalt-org/cobalt.rs/commit/dc4b9cefcd10d26bfa7a8cce1ef431dc53eafe29), closes [#328](https://github.com/cobalt-org/cobalt.rs/issues/328), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))

#### Breaking Changes

* **liquid:**  Dedicated _includes dir ([dc4b9cef](https://github.com/cobalt-org/cobalt.rs/commit/dc4b9cefcd10d26bfa7a8cce1ef431dc53eafe29), closes [#328](https://github.com/cobalt-org/cobalt.rs/issues/328))



## 0.8.0 - 2017-11-09


#### Bug Fixes

* **new:**
  *  Pages start as md by default ([892d798d](https://github.com/cobalt-org/cobalt.rs/commit/892d798d5d56099f2aad1d8883395bc2063447d9))
  *  Auto-created posts start as drafts ([0cfc1581](https://github.com/cobalt-org/cobalt.rs/commit/0cfc15818c8cb026d90ae127301d5364bd90f262))
* **watch:**  Rebuilding ignores dest ([b72863b9](https://github.com/cobalt-org/cobalt.rs/commit/b72863b94e904054cb12e8be640cd5e9d0ec27ab))
*   Auto-ignore dest in more cases ([8676a3a8](https://github.com/cobalt-org/cobalt.rs/commit/8676a3a8c3263212e61c886ce3ff85dc059ee3b1))
*   Don't ignore dest look-alikes ([33c7d0de](https://github.com/cobalt-org/cobalt.rs/commit/33c7d0dec590e7ac3621a82e32c97c6ae2ed0f69))
*   source/dest are now relative to config ([ce95b395](https://github.com/cobalt-org/cobalt.rs/commit/ce95b395412888f11ce2ab74d92dd297bbb74d45))
* **clean:**  Better detect what we can't clean ([78bbfc3e](https://github.com/cobalt-org/cobalt.rs/commit/78bbfc3eb3248c150f9058d5d528022c769abbe7))
* **cli:**  Clarify role of --destination ([a9fce407](https://github.com/cobalt-org/cobalt.rs/commit/a9fce407ba0a46ffd983819bfd30661acc435298))
* **config:**
  *  Don't support absolute paths ([6fd9af96](https://github.com/cobalt-org/cobalt.rs/commit/6fd9af96a0659eb9d85f3295924be5ae50dfb413), closes [#319](https://github.com/cobalt-org/cobalt.rs/issues/319))
* **jekyll:**  Clean up flag names ([80468b9f](https://github.com/cobalt-org/cobalt.rs/commit/80468b9f5d40be7771a0a83c0e82dac49a120773))
* **data-files:**  Provide information which file caused an error ([6b8e7018](https://github.com/cobalt-org/cobalt.rs/commit/6b8e7018336121447a8ee5d3e9f941bfc02627a5))
* **error:**  Report file path on parse error ([c1cf01cd](https://github.com/cobalt-org/cobalt.rs/commit/c1cf01cd52ee94f1361b9e7fa02320044c4e83f5))
* **log:**  Reduce noise when level is debug ([646d5897](https://github.com/cobalt-org/cobalt.rs/commit/646d5897dce4dac8af3269810349ec677646476f))

#### Breaking Changes

* **config:**
  * Auto-ignore dest in more cases ([8676a3a8](https://github.com/cobalt-org/cobalt.rs/commit/8676a3a8c3263212e61c886ce3ff85dc059ee3b1))
  * source/dest are now relative to config ([ce95b395](https://github.com/cobalt-org/cobalt.rs/commit/ce95b395412888f11ce2ab74d92dd297bbb74d45))
  * Remove layouts config setting ([137fb960](https://github.com/cobalt-org/cobalt.rs/commit/137fb960ca970a1b569a477ce1195ca45dc20ec7))
  * Find config in parent rather than default ([4e96a1fb](https://github.com/cobalt-org/cobalt.rs/commit/4e96a1fbeee7af20d243cc5930a0a2cc49e240bd))
* **cli:**
  * Remove global config flags ([b00aad63](https://github.com/cobalt-org/cobalt.rs/commit/b00aad63ff40f48997b4c9d7f797bbd383a393cf))
  * Remove source/posts/layouts flags ([70b549da](https://github.com/cobalt-org/cobalt.rs/commit/70b549dac31e444095768166bea37ab0f9f108a1))
* **jekyll:**  Clean up flag names ([80468b9f](https://github.com/cobalt-org/cobalt.rs/commit/80468b9f5d40be7771a0a83c0e82dac49a120773))

#### Features

* **init:**  Update defaults ([8a0eda99](https://github.com/cobalt-org/cobalt.rs/commit/8a0eda99e84d96653ca3950cb39a288821bc2ebe))
* **new:**  Clearer contract for `cobalt new` ([8e44311f](https://github.com/cobalt-org/cobalt.rs/commit/8e44311f33579c6964cf96eb595bef56cae02241))
*   New publish sub-command ([c0329df5](https://github.com/cobalt-org/cobalt.rs/commit/c0329df56eaaff938cca64f741c194c6772f24ad))
*   Expose config's site values ([7fea9ddf](https://github.com/cobalt-org/cobalt.rs/commit/7fea9ddf27f069ec6cb3a3157a39c6e0eae10cc8), closes [#216](https://github.com/cobalt-org/cobalt.rs/issues/216))
* **cli:**
  * Remove global config flags ([b00aad63](https://github.com/cobalt-org/cobalt.rs/commit/b00aad63ff40f48997b4c9d7f797bbd383a393cf))
  * Remove source/posts/layouts flags ([70b549da](https://github.com/cobalt-org/cobalt.rs/commit/70b549dac31e444095768166bea37ab0f9f108a1))
* **config:**
  *  Change future destination to `_site` ([da586c71](https://github.com/cobalt-org/cobalt.rs/commit/da586c71d1d1df911db2d143fe7b8777740d70ad))
  *  Remove layouts config setting ([137fb960](https://github.com/cobalt-org/cobalt.rs/commit/137fb960ca970a1b569a477ce1195ca45dc20ec7))
  *  Find config in parent rather than default ([4e96a1fb](https://github.com/cobalt-org/cobalt.rs/commit/4e96a1fbeee7af20d243cc5930a0a2cc49e240bd))
* **front:**
  *  Set `published_date` from filename ([ad69b1fc](https://github.com/cobalt-org/cobalt.rs/commit/ad69b1fcd22a57f6babac8b077c1f6453954144b))
* **liquid:**  Upgrade to 0.11 ([fd366fb9](https://github.com/cobalt-org/cobalt.rs/commit/fd366fb949aaf43164d8e7011141bf408a1a5c7f))
  * syntax: Add `arr[0]` and `obj["name"]` indexing (PR #141, fixes #127)
  * value: Add `nil` value to support foreign data (PR #140)



## 0.7.5 - 2017-10-22


#### Bug Fixes

* **jekyll:**  Remove crash ([7d07b2cc](https://github.com/cobalt-org/cobalt.rs/commit/7d07b2ccb3c91fd41630adf5c9f664c1bc59262e))
*   Don't dump a flag that isn't meant to be ([c3873e29](https://github.com/cobalt-org/cobalt.rs/commit/c3873e295342086f60b513fa893a5556f4f7987b))



## 0.7.4 - 2017-10-21


#### Bug Fixes

* **md:**  Add table/ref support ([1aa53d24](https://github.com/cobalt-org/cobalt.rs/commit/1aa53d2459b51db73d108f0ff532641ccf8a0287), closes [#303](https://github.com/cobalt-org/cobalt.rs/issues/303))

#### Features

*   Improve error reporting to user ([1e07708b](https://github.com/cobalt-org/cobalt.rs/commit/1e07708badd73523892e4ab7a7c17c40d090bae2))
* **data-files:**  Add data file support ([bb2d7c0f](https://github.com/cobalt-org/cobalt.rs/commit/bb2d7c0f3b841a6432bb68f5ccff83d8fe40050e), closes [#256](https://github.com/cobalt-org/cobalt.rs/issues/256))
* **scss:**  Implement compiling of SCSS files ([76b8d8ae](https://github.com/cobalt-org/cobalt.rs/commit/76b8d8ae665d597151a5386d07bebcb2418a74e6))

## 0.7.3 - 2017-10-05

#### Breaking Changes

* `page.path` now returns non-exploded path. ([7f571a8b](https://github.com/cobalt-org/cobalt.rs/commit/7f571a8bd5b75adcdfc5de103778a210bbc2f5e3))
* `--dump=liquid` is now split into `--dump=DocObject` and `--dump=DocTermplate` ([3439265b](https://github.com/cobalt-org/cobalt.rs/commit/3439265b64de9c7d87fad5f3c54501e0b33966f0))

#### Features

* Support `:categories` in permalink ([a9b4474f](https://github.com/cobalt-org/cobalt.rs/commit/a9b4474fdfa23279f0081a8864d7b6601f4e2b2d))
* Support nested frontmatter ([cafd42fa](https://github.com/cobalt-org/cobalt.rs/commit/cafd42fa3cdfb045dbc9eb2a5a8e35f2ff455e66))
* **syntax-highlight:**  Succeed on windows ([f1129fa8](https://github.com/cobalt-org/cobalt.rs/commit/f1129fa8fcef2c27583ed0a5cfe97c2dcb8246e4))
* **debug:**
  *  Dump doc with all defaults/globals ([e4ff582f](https://github.com/cobalt-org/cobalt.rs/commit/e4ff582f9937bee8013336e8c00ec437c67e5124))
  *  Link substitutions dump flag added ([3439265b](https://github.com/cobalt-org/cobalt.rs/commit/3439265b64de9c7d87fad5f3c54501e0b33966f0))

#### Bug Fixes

* **build:**  Do not attempt to build the output ([51f486a8](https://github.com/cobalt-org/cobalt.rs/commit/51f486a84d52530cb4af6b55e19f7c97674b35aa))
* **watch:**  Ignore dest rather than rebuild ([fce89368](https://github.com/cobalt-org/cobalt.rs/commit/fce8936800eb651e4234293df805a675f2a6fd0b), closes [#170](https://github.com/cobalt-org/cobalt.rs/issues/170))


## 0.7.2 - 2017-07-04

#### Bug Fixes

*   Bump syntect version to 1.7.0 and enable dump-load feature ([4d0e14a7](https://github.com/cobalt-org/cobalt.rs/commit/4d0e14a788f02de98b63ae94ad976e02c6e8334c))



## 0.7.1 - 2017-06-25

#### Bug Fixes

* **CI:**
  *  Show all build failures, not just first ([52916cd8](https://github.com/cobalt-org/cobalt.rs/commit/52916cd8b448e84c691e4d0517b53b287ff56efb))
  *  Re-enable Stable builds on Linux ([24f3b209](https://github.com/cobalt-org/cobalt.rs/commit/24f3b2093580b26a0aef5c9ed81fb53f6fc614d7))

## 0.7.0 - 2017-06-24

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
- Moved from `_posts` to `posts` ([@johannhof][])
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

<!-- next-url -->
[Unreleased]: https://github.com/assert-rs/predicates-rs/compare/v0.17.4...HEAD
[0.17.4]: https://github.com/assert-rs/predicates-rs/compare/v0.17.3...v0.17.4
[0.17.3]: https://github.com/assert-rs/predicates-rs/compare/v0.17.2...v0.17.3
[0.17.2]: https://github.com/assert-rs/predicates-rs/compare/v0.17.1...v0.17.2
[0.17.1]: https://github.com/assert-rs/predicates-rs/compare/v0.17.0...v0.17.1
[0.17.0]: https://github.com/assert-rs/predicates-rs/compare/v0.16.5...v0.17.0
[0.16.5]: https://github.com/cobalt-org/cobalt.rs/compare/v0.16.4...v0.16.5
