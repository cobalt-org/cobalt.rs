use chrono::Datelike;
use deunicode;
use itertools::Itertools;

static SLUG_INVALID_CHARS: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"([^a-zA-Z0-9]+)").unwrap());

/// Create a slug for a given file.  Correlates to Jekyll's :slug path tag
pub fn slugify<S: AsRef<str>>(name: S) -> String {
    slugify_str(name.as_ref())
}

fn slugify_str(name: &str) -> String {
    let name = deunicode::deunicode_with_tofu(name, "-");
    let slug = SLUG_INVALID_CHARS.replace_all(&name, "-");
    slug.trim_matches('-').to_lowercase()
}

/// Format a user-visible title out of a slug.  Correlates to Jekyll's "title" attribute
pub fn titleize_slug<S: AsRef<str>>(slug: S) -> String {
    titleize_slug_str(slug.as_ref())
}

fn titleize_slug_str(slug: &str) -> String {
    slug.split('-').map(title_case).join(" ")
}

/// Title-case a single word
fn title_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f
            .to_uppercase()
            .chain(c.flat_map(|t| t.to_lowercase()))
            .collect(),
    }
}

#[cfg(test)]
mod test_slug {
    use super::*;

    #[test]
    fn test_slugify() {
        let actual = slugify("___filE-worlD-__09___");
        assert_eq!(actual, "file-world-09");
    }

    #[test]
    fn test_slugify_unicode() {
        let actual = slugify("__Æneid__北亰-worlD-__09___");
        assert_eq!(actual, "aeneid-bei-jing-world-09");
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

pub fn parse_file_stem(stem: &str) -> (Option<crate::DateTime>, String) {
    let parts = DATE_PREFIX_REF.captures(stem).and_then(|caps| {
        let year: i32 = caps
            .get(1)
            .expect("unconditional capture")
            .as_str()
            .parse()
            .expect("regex gets back an integer");
        let month: u32 = caps
            .get(2)
            .expect("unconditional capture")
            .as_str()
            .parse()
            .expect("regex gets back an integer");
        let day: u32 = caps
            .get(3)
            .expect("unconditional capture")
            .as_str()
            .parse()
            .expect("regex gets back an integer");
        let published = crate::DateTime::default()
            .with_year(year)
            .and_then(|d| d.with_month(month))
            .and_then(|d| d.with_day(day));
        published.map(|p| {
            (
                Some(p),
                caps.get(4)
                    .expect("unconditional capture")
                    .as_str()
                    .to_owned(),
            )
        })
    });

    parts.unwrap_or_else(|| (None, stem.to_owned()))
}

#[cfg(test)]
mod test_stem {
    use super::*;

    #[test]
    fn parse_file_stem_empty() {
        assert_eq!(parse_file_stem(""), (None, "".to_owned()));
    }

    #[test]
    fn parse_file_stem_none() {
        assert_eq!(
            parse_file_stem("First Blog Post"),
            (None, "First Blog Post".to_owned())
        );
    }

    #[test]
    fn parse_file_stem_out_of_range_month() {
        assert_eq!(
            parse_file_stem("2017-30-5 First Blog Post"),
            (None, "2017-30-5 First Blog Post".to_owned())
        );
    }

    #[test]
    fn parse_file_stem_out_of_range_day() {
        assert_eq!(
            parse_file_stem("2017-3-50 First Blog Post"),
            (None, "2017-3-50 First Blog Post".to_owned())
        );
    }

    #[test]
    fn parse_file_stem_single_digit() {
        assert_eq!(
            parse_file_stem("2017-3-5 First Blog Post"),
            (
                Some(
                    crate::DateTime::default()
                        .with_year(2017)
                        .unwrap()
                        .with_month(3)
                        .unwrap()
                        .with_day(5)
                        .unwrap()
                ),
                "First Blog Post".to_owned()
            )
        );
    }

    #[test]
    fn parse_file_stem_double_digit() {
        assert_eq!(
            parse_file_stem("2017-12-25 First Blog Post"),
            (
                Some(
                    crate::DateTime::default()
                        .with_year(2017)
                        .unwrap()
                        .with_month(12)
                        .unwrap()
                        .with_day(25)
                        .unwrap()
                ),
                "First Blog Post".to_owned()
            )
        );
    }

    #[test]
    fn parse_file_stem_double_digit_leading_zero() {
        assert_eq!(
            parse_file_stem("2017-03-05 First Blog Post"),
            (
                Some(
                    crate::DateTime::default()
                        .with_year(2017)
                        .unwrap()
                        .with_month(3)
                        .unwrap()
                        .with_day(5)
                        .unwrap()
                ),
                "First Blog Post".to_owned()
            )
        );
    }

    #[test]
    fn parse_file_stem_dashed() {
        assert_eq!(
            parse_file_stem("2017-3-5-First-Blog-Post"),
            (
                Some(
                    crate::DateTime::default()
                        .with_year(2017)
                        .unwrap()
                        .with_month(3)
                        .unwrap()
                        .with_day(5)
                        .unwrap()
                ),
                "First-Blog-Post".to_owned()
            )
        );
    }
}
