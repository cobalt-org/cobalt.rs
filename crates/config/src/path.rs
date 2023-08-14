use deunicode;
use itertools::Itertools;

static SLUG_INVALID_CHARS: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"([^a-zA-Z0-9]+)").unwrap());

/// Create a slug for a given file.  Correlates to Jekyll's :slug path tag
pub fn slugify<S: AsRef<str>>(name: S) -> liquid_core::model::KString {
    slugify_str(name.as_ref())
}

fn slugify_str(name: &str) -> liquid_core::model::KString {
    let name = deunicode::deunicode_with_tofu(name, "-");
    let slug = SLUG_INVALID_CHARS.replace_all(&name, "-");
    let slug = slug.trim_matches('-').to_lowercase();
    slug.into()
}

/// Format a user-visible title out of a slug.  Correlates to Jekyll's "title" attribute
pub fn titleize_slug<S: AsRef<str>>(slug: S) -> liquid_core::model::KString {
    titleize_slug_str(slug.as_ref())
}

fn titleize_slug_str(slug: &str) -> liquid_core::model::KString {
    slug.split('-').map(title_case).join(" ").into()
}

/// Title-case a single word
fn title_case(s: &str) -> liquid_core::model::KString {
    let mut c = s.chars();
    let title = match c.next() {
        None => String::new(),
        Some(f) => f
            .to_uppercase()
            .chain(c.flat_map(|t| t.to_lowercase()))
            .collect(),
    };
    title.into()
}

#[derive(
    Default,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
#[repr(transparent)]
#[serde(try_from = "String")]
pub struct RelPath(relative_path::RelativePathBuf);

impl RelPath {
    pub fn new() -> Self {
        let path = relative_path::RelativePathBuf::new();
        Self(path)
    }

    pub fn from_path(value: impl AsRef<std::path::Path>) -> Option<Self> {
        let value = value.as_ref();
        let path: Option<relative_path::RelativePathBuf> =
            value.components().map(|c| c.as_os_str().to_str()).collect();
        let path = path?.normalize();
        Some(Self(path))
    }

    pub fn from_unchecked(value: impl AsRef<std::path::Path>) -> Self {
        Self::from_path(value).unwrap()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn as_path(&self) -> &std::path::Path {
        std::path::Path::new(self.as_str())
    }

    pub fn into_inner(self) -> relative_path::RelativePathBuf {
        self.0
    }
}

impl PartialEq<str> for RelPath {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        *self == RelPath::from_unchecked(other)
    }
}

impl<'s> PartialEq<&'s str> for RelPath {
    #[inline]
    fn eq(&self, other: &&'s str) -> bool {
        *self == RelPath::from_unchecked(*other)
    }
}

impl std::fmt::Display for RelPath {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(fmt)
    }
}

impl<'s> std::convert::TryFrom<&'s str> for RelPath {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let path = std::path::Path::new(value);
        if path.is_absolute() || path.has_root() {
            Err("Absolute paths are not supported")
        } else {
            Ok(Self::from_unchecked(value))
        }
    }
}

impl std::convert::TryFrom<String> for RelPath {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.as_str();
        Self::try_from(value)
    }
}

impl std::ops::Deref for RelPath {
    type Target = relative_path::RelativePath;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl AsRef<str> for RelPath {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<std::path::Path> for RelPath {
    #[inline]
    fn as_ref(&self) -> &std::path::Path {
        self.as_path()
    }
}

impl AsRef<relative_path::RelativePath> for RelPath {
    #[inline]
    fn as_ref(&self) -> &relative_path::RelativePath {
        &self.0
    }
}

#[cfg(test)]
mod test_slug {
    use super::*;

    #[test]
    fn test_slugify() {
        let actual = slugify("___filE-worldD-__09___");
        assert_eq!(actual, "file-worldd-09");
    }

    #[test]
    fn test_slugify_unicode() {
        let actual = slugify("__Æneid__北亰-worldD-__09___");
        assert_eq!(actual, "aeneid-bei-jing-worldd-09");
    }

    #[test]
    fn test_titleize_slug() {
        let actual = titleize_slug("tItLeIzE-sLuG");
        assert_eq!(actual, "Titleize Slug");
    }
}

pub fn split_ext(name: &str) -> (&str, Option<&str>) {
    name.rsplit_once('.')
        .map(|(n, e)| (n, Some(e)))
        .unwrap_or_else(|| (name, None))
}

static DATE_PREFIX_REF: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| {
    regex::Regex::new(r"^(\d{4})-(\d{1,2})-(\d{1,2})[- ](.*)$").unwrap()
});

pub fn parse_file_stem(stem: &str) -> (Option<crate::DateTime>, liquid_core::model::KString) {
    let parts = DATE_PREFIX_REF.captures(stem).map(|caps| {
        let year: i32 = caps
            .get(1)
            .expect("unconditional capture")
            .as_str()
            .parse()
            .expect("regex gets back an integer");
        let month: u8 = caps
            .get(2)
            .expect("unconditional capture")
            .as_str()
            .parse()
            .expect("regex gets back an integer");
        let day: u8 = caps
            .get(3)
            .expect("unconditional capture")
            .as_str()
            .parse()
            .expect("regex gets back an integer");
        let published = crate::DateTime::from_ymd(year, month, day);
        (
            Some(published),
            liquid_core::model::KString::from_ref(
                caps.get(4).expect("unconditional capture").as_str(),
            ),
        )
    });

    parts.unwrap_or_else(|| (None, liquid_core::model::KString::from_ref(stem)))
}

#[cfg(test)]
mod test_stem {
    use super::*;

    #[test]
    fn parse_file_stem_empty() {
        assert_eq!(parse_file_stem(""), (None, "".into()));
    }

    #[test]
    fn parse_file_stem_none() {
        assert_eq!(
            parse_file_stem("First Blog Post"),
            (None, "First Blog Post".into())
        );
    }

    #[test]
    #[should_panic]
    fn parse_file_stem_out_of_range_month() {
        assert_eq!(
            parse_file_stem("2017-30-5 First Blog Post"),
            (None, "2017-30-5 First Blog Post".into())
        );
    }

    #[test]
    #[should_panic]
    fn parse_file_stem_out_of_range_day() {
        assert_eq!(
            parse_file_stem("2017-3-50 First Blog Post"),
            (None, "2017-3-50 First Blog Post".into())
        );
    }

    #[test]
    fn parse_file_stem_single_digit() {
        assert_eq!(
            parse_file_stem("2017-3-5 First Blog Post"),
            (
                Some(crate::DateTime::from_ymd(2017, 3, 5)),
                "First Blog Post".into()
            )
        );
    }

    #[test]
    fn parse_file_stem_double_digit() {
        assert_eq!(
            parse_file_stem("2017-12-25 First Blog Post"),
            (
                Some(crate::DateTime::from_ymd(2017, 12, 25)),
                "First Blog Post".into()
            )
        );
    }

    #[test]
    fn parse_file_stem_double_digit_leading_zero() {
        assert_eq!(
            parse_file_stem("2017-03-05 First Blog Post"),
            (
                Some(crate::DateTime::from_ymd(2017, 3, 5)),
                "First Blog Post".into()
            )
        );
    }

    #[test]
    fn parse_file_stem_dashed() {
        assert_eq!(
            parse_file_stem("2017-3-5-First-Blog-Post"),
            (
                Some(crate::DateTime::from_ymd(2017, 3, 5)),
                "First-Blog-Post".into()
            )
        );
    }
}

#[cfg(test)]
mod test_rel_path {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test_try_from_cwd_is_empty() {
        assert_eq!(RelPath::new().as_str(), "");
        assert_eq!(RelPath::default().as_str(), "");
        assert_eq!(RelPath::try_from(".").unwrap().as_str(), "");
        assert_eq!(RelPath::try_from("./").unwrap().as_str(), "");
    }

    #[test]
    fn test_try_from_relpath_works() {
        assert_eq!(RelPath::try_from("./foo/bar").unwrap().as_str(), "foo/bar");
        assert_eq!(RelPath::try_from("foo/bar").unwrap().as_str(), "foo/bar");
    }

    #[test]
    fn test_try_from_abspath_fails() {
        let case = RelPath::try_from("/foo/bar");
        println!("{:?}", case);
        assert!(case.is_err());
    }
}
