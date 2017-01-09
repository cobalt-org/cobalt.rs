# This script takes care of testing your crate

set -ex

main() {
    TARGET=$TARGET cross build --target $TARGET
    TARGET=$TARGET cross build --target $TARGET --release

    TARGET=$TARGET cross test --target $TARGET
    TARGET=$TARGET cross test --target $TARGET --release

    if [ $TRAVIS_RUST_VERSION == nightly ]; then
      cargo clippy
    fi

    cargo fmt -- --write-mode=diff
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi

