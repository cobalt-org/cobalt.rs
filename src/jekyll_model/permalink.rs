use std::fmt;

use itertools;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Part {
    Constant(String),
    Variable(String),
}

impl Part {
    pub fn resolve<R>(self, resolver: &R) -> Self
    where
        R: Fn(&str) -> Part,
    {
        match self {
            Part::Constant(constant) => Part::Constant(constant),
            Part::Variable(var) => resolver(&var),
        }
    }
}

impl fmt::Display for Part {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Part::Constant(ref constant) => write!(f, "{}", constant),
            Part::Variable(ref var) => write!(f, ":{}", var),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Permalink(Vec<Part>);

impl Permalink {
    pub fn parse(perma: &str) -> Self {
        parse_permalink(perma, VARIABLES)
    }

    pub fn resolve<R>(self, resolver: &R) -> Self
    where
        R: Fn(&str) -> Part,
    {
        let v: Vec<Part> = self.0.into_iter().map(|p| p.resolve(resolver)).collect();
        Permalink(v)
    }

    pub fn push(&mut self, new: Part) {
        self.0.push(new)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for Permalink {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut value = itertools::join(self.0.iter().map(|p| p.to_string()), "");
        if !value.starts_with('/') {
            value.insert(0, '/');
        }
        write!(f, "{}", value)
    }
}

pub const VARIABLES: &[&str] = &[
    "path",
    "filename",
    "slug",
    "categories",
    "output_ext",
    "year",
    "month",
    "i_month",
    "day",
    "i_day",
    "hour",
    "minute",
    "second",
];

fn split_variable(var: &str, variables: &[&str]) -> Vec<Part> {
    for supported in variables {
        if var.starts_with(supported) {
            let remaining = var.trim_left_matches(supported);
            let supported: &str = supported;
            let var = supported.to_owned();
            let var = Part::Variable(var);
            if remaining.is_empty() {
                return vec![var];
            } else {
                let constant = Part::Constant(remaining.to_owned());
                return vec![var, constant];
            }
        }
    }

    // Assume the whole thing is a variable
    let var = Part::Variable(var.to_owned());
    vec![var]
}

fn parse_permalink(perma: &str, variables: &[&str]) -> Permalink {
    let mut perma = perma.split(':');

    let mut result = Permalink::default();

    let constant = perma
        .next()
        .and_then(|s| if s.is_empty() { None } else { Some(s) })
        .map(|s| Part::Constant(s.to_owned()));
    if let Some(constant) = constant {
        result.push(constant);
    }

    for part in perma.flat_map(|s| split_variable(s, variables)) {
        result.push(part);
    }

    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_empty() {
        let permalink = Permalink::parse("");
        assert!(permalink.is_empty());
    }

    #[test]
    fn parse_constant() {
        let actual = &Permalink::parse("hello/world").0;
        let expected = &[Part::Constant("hello/world".to_owned())];
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_variable() {
        let actual = &Permalink::parse(":path").0;
        let expected = &[Part::Variable("path".to_owned())];
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_leading_constant() {
        let actual = &Permalink::parse("hello/world/:path").0;
        let expected = &[
            Part::Constant("hello/world/".to_owned()),
            Part::Variable("path".to_owned()),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_trailing_constant() {
        let actual = &Permalink::parse(":path/hello/world").0;
        let expected = &[
            Part::Variable("path".to_owned()),
            Part::Constant("/hello/world".to_owned()),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_mixed() {
        let actual = &Permalink::parse("hello/:path/world/:i_day/").0;
        let expected = &[
            Part::Constant("hello/".to_owned()),
            Part::Variable("path".to_owned()),
            Part::Constant("/world/".to_owned()),
            Part::Variable("i_day".to_owned()),
            Part::Constant("/".to_owned()),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_invalid_variable() {
        let actual = &Permalink::parse("hello/:party/world/:i_day/").0;
        let expected = &[
            Part::Constant("hello/".to_owned()),
            Part::Variable("party/world/".to_owned()),
            Part::Variable("i_day".to_owned()),
            Part::Constant("/".to_owned()),
        ];
        assert_eq!(actual, expected);
    }
}
