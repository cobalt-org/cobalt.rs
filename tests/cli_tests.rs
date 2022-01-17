#[test]
#[cfg(feature = "syntax-highlight")]
fn cli_tests() {
    let t = trycmd::TestCases::new();
    t.case("tests/cmd/*.md");
    #[cfg(not(feature = "serve"))]
    {
        t.skip("tests/cmd/errors.md");
    }
}
