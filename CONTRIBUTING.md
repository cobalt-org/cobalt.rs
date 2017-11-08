Thanks for contributing! :snowman:

Feel free to create issues and make pull requests, we'll try to quickly review them.

If you're looking for things to do check out the [open issues](https://github.com/cobalt-org/cobalt.rs/issues), especially those with the [easy](https://github.com/cobalt-org/cobalt.rs/issues?q=is%3Aissue+is%3Aopen+label%3Aeasy) flag. Or take a grep through [all TODO comments](https://github.com/cobalt-org/cobalt.rs/search?q=TODO) in the code and feel free to help us out there!

🌈 **Here's a checklist for the perfect pull request:**
- [ ] Make sure existing tests still work by running `cargo test` locally.
- [ ] Add new tests for any new feature or regression tests for bugfixes.
- [ ] Install [Clippy](https://github.com/Manishearth/rust-clippy) and run `rustup run nightly cargo clippy` to catch common mistakes (will be checked by Travis)
- [ ] Install [Rustfmt](https://github.com/rust-lang-nursery/rustfmt) and run `cargo fmt` to format your code (will also be checked by Travis)

If you need assistance, you can join the `#cobalt` channel on `irc.mozilla.org` or the Gitter chat [![Gitter](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/cobalt-org/cobalt.rs)

We want you to feel safe and welcome and will enforce the [The Rust Code of Conduct](https://www.rust-lang.org/conduct.html) on all communication platforms of this project.
Please contact [@johannhof](https://github.com/johannhof) for questions or in cases of violation.

# Releasing

When we're ready to release, a project owner should do the following
- Determine what the next version is, according to semver
- Bump version in a commit
  - Run `clog --setversion <X>.<Y>.<Z>`, touch up the log
  - Update the version in `Cargo.toml`
  - Run `cargo check` to update `Cargo.lock`
- Tag the commit via `git tag -a v<X>.<Y>.<Z>`
- `git push upstream master --tag v<X>.<Y>.<Z>`
- Run `cargo publish` (run `cargo login` first if needed)
