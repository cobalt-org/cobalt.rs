#[test]
fn cli_tests() {
    let t = trycmd::TestCases::new();
    t.case("tests/cmd/*.md");
    #[cfg(not(feature = "syntax-highlight"))]
    {
        t.skip("tests/cmd/init.md");
        t.skip("tests/cmd/log_level.md");
    }
    #[cfg(not(feature = "serve"))]
    {
        t.skip("tests/cmd/errors.md");
    }
}
