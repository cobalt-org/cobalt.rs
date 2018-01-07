use super::sass;

#[derive(Debug, PartialEq, Default)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct AssetsBuilder {
    pub sass: sass::SassBuilder,
}

impl AssetsBuilder {
    pub fn build(self) -> Assets {
        Assets { sass: self.sass.build() }
    }
}

#[derive(Debug, PartialEq, Default)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Assets {
    pub sass: sass::SassCompiler,
}
