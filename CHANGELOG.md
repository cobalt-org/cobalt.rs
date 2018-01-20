<a name="0.11.1"></a>
## 0.11.1 (2018-01-10)


#### Features

* **liquid:**  Support contains operator ([668f4726](https://github.com/cobalt-org/cobalt.rs/commit/668f47260d1e7db7666a77d8c100997da2531dee))



<a name="0.11.0"></a>
## 0.11.0 (2018-01-09)

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



<a name="0.10.0"></a>
## 0.10.0 (2018-01-04)


#### Features

* **error:**  Identify layout errors ([0ec3a3d0](https://github.com/cobalt-org/cobalt.rs/commit/0ec3a3d0fbab0d51e20304d3e1a5726e987e36da))

#### Bug Fixes

*   Correctly copy scss files when sass is disabled ([f04bd9d2](https://github.com/cobalt-org/cobalt.rs/commit/f04bd9d2f38daf988c3bf244c56fe3ff97c3a420))
*   Log context for failures ([be258bfb](https://github.com/cobalt-org/cobalt.rs/commit/be258bfb31bb5a0d7131469eba27b13b82a5256f))
* **config:**  On empty file, use right root ([a99ca197](https://github.com/cobalt-org/cobalt.rs/commit/a99ca197564cead19c247ca91429077b163cd794))
* **drafts:**  Config can enable again ([729e0b18](https://github.com/cobalt-org/cobalt.rs/commit/729e0b18fdc9ad5ec173c73d015ff8cb0364a675))
* **front:**  Ensure exceprt is rendered before used ([9e714697](https://github.com/cobalt-org/cobalt.rs/commit/9e7146979e8df4afaa9bdc1890fe725b0b551bf7))
* **includes:**  Support including hidden files ([cb577c42](https://github.com/cobalt-org/cobalt.rs/commit/cb577c42624720740e382c9336510807f67ca0ca))

#### Breaking Changes

* **error:**  Identify layout errors ([0ec3a3d0](https://github.com/cobalt-org/cobalt.rs/commit/0ec3a3d0fbab0d51e20304d3e1a5726e987e36da))

<a name="0.9.0"></a>
## 0.9.0 (2017-11-30)


#### Features

* **liquid:**  Dedicated _includes dir ([dc4b9cef](https://github.com/cobalt-org/cobalt.rs/commit/dc4b9cefcd10d26bfa7a8cce1ef431dc53eafe29), closes [#328](https://github.com/cobalt-org/cobalt.rs/issues/328), breaks [#](https://github.com/cobalt-org/cobalt.rs/issues/))

#### Breaking Changes

* **liquid:**  Dedicated _includes dir ([dc4b9cef](https://github.com/cobalt-org/cobalt.rs/commit/dc4b9cefcd10d26bfa7a8cce1ef431dc53eafe29), closes [#328](https://github.com/cobalt-org/cobalt.rs/issues/328))



<a name="0.8.0"></a>
## 0.8.0 (2017-11-09)


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



<a name="0.7.5"></a>
## 0.7.5 (2017-10-22)


#### Bug Fixes

* **jekyll:**  Remove crash ([7d07b2cc](https://github.com/cobalt-org/cobalt.rs/commit/7d07b2ccb3c91fd41630adf5c9f664c1bc59262e))
*   Don't dump a flag that isn't meant to be ([c3873e29](https://github.com/cobalt-org/cobalt.rs/commit/c3873e295342086f60b513fa893a5556f4f7987b))



<a name="0.7.4"></a>
## 0.7.4 (2017-10-21)


#### Bug Fixes

* **md:**  Add table/ref support ([1aa53d24](https://github.com/cobalt-org/cobalt.rs/commit/1aa53d2459b51db73d108f0ff532641ccf8a0287), closes [#303](https://github.com/cobalt-org/cobalt.rs/issues/303))

#### Features

*   Improve error reporting to user ([1e07708b](https://github.com/cobalt-org/cobalt.rs/commit/1e07708badd73523892e4ab7a7c17c40d090bae2))
* **data-files:**  Add data file support ([bb2d7c0f](https://github.com/cobalt-org/cobalt.rs/commit/bb2d7c0f3b841a6432bb68f5ccff83d8fe40050e), closes [#256](https://github.com/cobalt-org/cobalt.rs/issues/256))
* **scss:**  Implement compiling of SCSS files ([76b8d8ae](https://github.com/cobalt-org/cobalt.rs/commit/76b8d8ae665d597151a5386d07bebcb2418a74e6))

<a name="0.7.3"></a>
## 0.7.3 (2017-10-05)

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
