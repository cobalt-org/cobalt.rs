#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Assets {
    pub sass: Sass,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Sass {
    #[serde(skip)]
    pub import_dir: &'static str,
    pub style: SassOutputStyle,
}

impl Default for Sass {
    fn default() -> Self {
        Self {
            import_dir: "_sass",
            style: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub enum SassOutputStyle {
    Nested,
    Expanded,
    Compact,
    Compressed,
}

impl Default for SassOutputStyle {
    fn default() -> Self {
        SassOutputStyle::Nested
    }
}
