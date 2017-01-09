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

    if [ $target = x86_64-unknown-linux-gnu ]; then
        # TODO: use https://github.com/japaric/rustfmt-bin
        cargo install rustfmt || true
    fi

    if [ $TRAVIS_RUST_VERSION == nightly ]; then
      cargo install clippy || true
    fi
}

main
