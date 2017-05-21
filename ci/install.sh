set -ex

main() {
    curl https://sh.rustup.rs -sSf | \
        sh -s -- -y --default-toolchain $TRAVIS_RUST_VERSION

    # Install rustfmt
    cargo install rustfmt --force --vers 0.8.3

    if [ "$TRAVIS_RUST_VERSION" = "nightly" ]; then
      cargo install clippy --force
    fi
}

main
