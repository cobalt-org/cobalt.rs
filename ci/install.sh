set -ex

main() {
    curl https://sh.rustup.rs -sSf | \
        sh -s -- -y --default-toolchain $TRAVIS_RUST_VERSION

    local target=
    if [ $TRAVIS_OS_NAME = linux ]; then
        target=x86_64-unknown-linux-gnu
    else
        target=x86_64-apple-darwin
    fi

    curl -LSfs https://japaric.github.io/trust/install.sh | \
        sh -s -- \
           --force \
           --git japaric/cross \
           --tag v0.1.4 \
           --target $target

    # Install rustfmt
    curl -LSfs https://japaric.github.io/trust/install.sh | \
        sh -s -- \
           --force \
           --crate rustfmt \
           --git japaric/rustfmt-bin \
           --tag v0.6.3-20170107 \
           --target $target

    if [ $TRAVIS_RUST_VERSION == nightly ]; then
      cargo install clippy || true
    fi
}

main
