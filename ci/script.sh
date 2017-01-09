set -ex

main() {
    cargo build
    cargo test

    if [ "$TRAVIS_RUST_VERSION" = "nightly" ]; then
      cargo clippy -- --version
      cargo clippy
    fi

    cargo fmt -- --version
    cargo fmt -- --write-mode=diff
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi

