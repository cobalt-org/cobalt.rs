#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
#[cfg_attr(feature = "unstable", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "unstable"), non_exhaustive)]
pub struct Site {
    pub title: Option<String>,
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub sitemap: Option<String>,
    pub data: Option<liquid_value::Object>,
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
