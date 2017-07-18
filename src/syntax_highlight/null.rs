use error;

pub fn has_syntax_theme(_name: &str) -> error::Result<bool> {
    bail!("Themes are unsupported in this build.");
}

pub fn list_syntax_themes<'a>() -> Vec<&'a String> {
    vec![]
}

pub fn list_syntaxes<'a>() -> Vec<String> {
    vec![]
}
