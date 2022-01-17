#[test]
fn cli_tests() {
    let t = trycmd::TestCases::new();
    t.case("tests/cmd/*.md");
    #[cfg(not(feature = "syntax-highlight"))]
    {
        t.skip("tests/cmd/init.md");
        t.skip("tests/cmd/example.md");
        t.skip("tests/cmd/log_level.md");
        t.skip("tests/cmd/vimwiki_not_templated.md");
    }
    #[cfg(not(feature = "serve"))]
    {
        t.skip("tests/cmd/errors.md");
    }
    #[cfg(not(feature = "sass"))]
    {
        t.skip("tests/cmd/sass.md");
        t.skip("tests/cmd/sass_custom_config.md");
    }
    #[cfg(not(feature = "html-minifier"))]
    {
        t.skip("tests/cmd/example_minified.md");
    }
}
