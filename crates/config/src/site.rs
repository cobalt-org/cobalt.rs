#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "unstable", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "unstable"), non_exhaustive)]
pub struct Site {
    pub title: Option<liquid_core::model::KString>,
    pub description: Option<liquid_core::model::KString>,
    pub base_url: Option<liquid_core::model::KString>,
    pub sitemap: Option<crate::RelPath>,
    pub data: Option<liquid_core::Object>,
    #[serde(skip)]
    pub data_dir: &'static str,
}

impl Default for Site {
    fn default() -> Self {
        Self {
            title: Default::default(),
            description: Default::default(),
            base_url: Default::default(),
            sitemap: Default::default(),
            data: Default::default(),
            data_dir: "_data",
        }
    }
}
