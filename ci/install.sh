set -ex

main() {
    curl https://sh.rustup.rs -sSf | \
        sh -s -- -y --default-toolchain $TRAVIS_RUST_VERSION

    # Install rustfmt
    curl -LSfs https://japaric.github.io/trust/install.sh | \
        sh -s -- \
           --force \
           --crate rustfmt \
           --git japaric/rustfmt-bin \
           --tag v0.6.3-20170107 \

    if [ "$TRAVIS_RUST_VERSION" = "nightly" ]; then
      cargo install clippy || true
    fi
}

main
