#[test]
#[cfg(feature = "syntax-highlight")]
fn cli_tests() {
    trycmd::TestCases::new().case("tests/cmd/*.md");
}
