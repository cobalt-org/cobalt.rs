set -ex

main() {
  # install kcov
  wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz
  tar xzf master.tar.gz
  mkdir kcov-master/build
  cd kcov-master/build
  cmake ..
  make
  make install DESTDIR=../tmp
  cd ../..

  ls target/debug

  # collect coverage
  for file in target/debug/cobalt-*; do
    ./kcov-master/tmp/usr/local/bin/kcov --exclude-pattern=/.cargo,target/ --verify target/kcov "$file"
  done

  ./kcov-master/tmp/usr/local/bin/kcov --exclude-pattern=/.cargo,target/ --verify target/kcov target/debug/mod-*

  # the last job should upload the merged data
  ./kcov-master/tmp/usr/local/bin/kcov --coveralls-id=$TRAVIS_JOB_ID --exclude-pattern=/.cargo,target/ --verify target/kcov target/debug/cli-*

  ls target/kcov
}

# only run coverage on the default job
if [ "$TRAVIS_RUST_VERSION" = "stable" -a "$TARGET" = "x86_64-unknown-linux-gnu" ]; then
    main
fi

