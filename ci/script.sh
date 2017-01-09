# This script takes care of testing your crate

set -ex

main() {
    if [ $TARGET = x86_64-unknown-linux-gnu ]; then
        cargo fmt -- --write-mode=diff
    fi

    cross build --target $TARGET
    cross build --target $TARGET --release

    if [ -n $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET
    cross test --target $TARGET --release

    if [ $TRAVIS_RUST_VERSION == nightly ]; then
      cargo clippy
    fi

}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi

