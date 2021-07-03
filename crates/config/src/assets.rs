#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
#[cfg_attr(feature = "unstable", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "unstable"), non_exhaustive)]
pub struct Assets {
    pub sass: Sass,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
#[cfg_attr(feature = "unstable", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "unstable"), non_exhaustive)]
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
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "unstable"), non_exhaustive)]
pub enum SassOutputStyle {
    Nested,
    Expanded,
    Compact,
    Compressed,
    #[cfg(not(feature = "unstable"))]
    #[doc(hidden)]
    #[serde(other)]
    Unknown,
}

impl Default for SassOutputStyle {
    fn default() -> Self {
        SassOutputStyle::Nested
    }
}
