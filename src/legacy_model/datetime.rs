use std::convert;
use std::fmt;
use std::ops;

use chrono;
use chrono::TimeZone;
use serde;

use cobalt_model;

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
pub struct DateTime(chrono::DateTime<chrono::FixedOffset>);

impl DateTime {
    pub fn now() -> DateTime {
        let d = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east(0));
        DateTime(d)
    }

    pub fn parse<S: AsRef<str>>(d: S) -> Option<DateTime> {
        DateTime::parse_str(d.as_ref())
    }

    fn parse_str(d: &str) -> Option<DateTime> {
        chrono::DateTime::parse_from_str(d, "%d %B %Y %H:%M:%S %z")
            .ok()
            .map(DateTime)
    }

    pub fn format(&self) -> String {
        self.0.format("%d %B %Y %H:%M:%S %z").to_string()
    }

    pub fn with_offset(&self, secs: i32) -> Option<DateTime> {
        let timezone = chrono::FixedOffset::east_opt(secs);
        timezone.map(|tz| self.0.with_timezone(&tz).into())
    }
}

impl Default for DateTime {
    fn default() -> DateTime {
        let d = chrono::Utc
            .timestamp(0, 0)
            .with_timezone(&chrono::FixedOffset::east(0));
        DateTime(d)
    }
}

impl ops::Deref for DateTime {
    type Target = chrono::DateTime<chrono::FixedOffset>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for DateTime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl convert::From<chrono::DateTime<chrono::FixedOffset>> for DateTime {
    fn from(v: chrono::DateTime<chrono::FixedOffset>) -> Self {
        DateTime(v)
    }
}

impl convert::From<DateTime> for chrono::DateTime<chrono::FixedOffset> {
    fn from(v: DateTime) -> Self {
        v.0
    }
}

impl convert::From<DateTime> for cobalt_model::DateTime {
    fn from(v: DateTime) -> Self {
        v.0.into()
    }
}

impl chrono::Datelike for DateTime {
    #[inline]
    fn year(&self) -> i32 {
        self.0.year()
    }
    #[inline]
    fn month(&self) -> u32 {
        self.0.month()
    }
    #[inline]
    fn month0(&self) -> u32 {
        self.0.month0()
    }
    #[inline]
    fn day(&self) -> u32 {
        self.0.day()
    }
    #[inline]
    fn day0(&self) -> u32 {
        self.0.day0()
    }
    #[inline]
    fn ordinal(&self) -> u32 {
        self.0.ordinal()
    }
    #[inline]
    fn ordinal0(&self) -> u32 {
        self.0.ordinal0()
    }
    #[inline]
    fn weekday(&self) -> chrono::Weekday {
        self.0.weekday()
    }
    #[inline]
    fn iso_week(&self) -> chrono::IsoWeek {
        self.0.iso_week()
    }

    #[inline]
    fn with_year(&self, year: i32) -> Option<DateTime> {
        self.0.with_year(year).map(|d| d.into())
    }

    #[inline]
    fn with_month(&self, month: u32) -> Option<DateTime> {
        self.0.with_month(month).map(|d| d.into())
    }

    #[inline]
    fn with_month0(&self, month0: u32) -> Option<DateTime> {
        self.0.with_month0(month0).map(|d| d.into())
    }

    #[inline]
    fn with_day(&self, day: u32) -> Option<DateTime> {
        self.0.with_day(day).map(|d| d.into())
    }

    #[inline]
    fn with_day0(&self, day0: u32) -> Option<DateTime> {
        self.0.with_day(day0).map(|d| d.into())
    }

    #[inline]
    fn with_ordinal(&self, ordinal: u32) -> Option<DateTime> {
        self.0.with_ordinal(ordinal).map(|d| d.into())
    }

    #[inline]
    fn with_ordinal0(&self, ordinal0: u32) -> Option<DateTime> {
        self.0.with_ordinal0(ordinal0).map(|d| d.into())
    }
}

impl chrono::Timelike for DateTime {
    #[inline]
    fn hour(&self) -> u32 {
        self.0.hour()
    }
    #[inline]
    fn minute(&self) -> u32 {
        self.0.minute()
    }
    #[inline]
    fn second(&self) -> u32 {
        self.0.second()
    }
    #[inline]
    fn nanosecond(&self) -> u32 {
        self.0.nanosecond()
    }

    #[inline]
    fn with_hour(&self, hour: u32) -> Option<DateTime> {
        self.0.with_hour(hour).map(|d| d.into())
    }

    #[inline]
    fn with_minute(&self, min: u32) -> Option<DateTime> {
        self.0.with_minute(min).map(|d| d.into())
    }

    #[inline]
    fn with_second(&self, sec: u32) -> Option<DateTime> {
        self.0.with_second(sec).map(|d| d.into())
    }

    #[inline]
    fn with_nanosecond(&self, nano: u32) -> Option<DateTime> {
        self.0.with_nanosecond(nano).map(|d| d.into())
    }
}

impl serde::Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        serializer.collect_str(&self.format())
    }
}

struct DateTimeVisitor;

impl<'de> serde::de::Visitor<'de> for DateTimeVisitor {
    type Value = DateTime;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a formatted date and time string")
    }

    fn visit_str<E>(self, value: &str) -> Result<DateTime, E>
        where E: serde::de::Error
    {
        DateTime::parse(value).ok_or_else(|| E::custom(format!("Invalid datetime '{}'", value)))
    }
}

impl<'de> serde::de::Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::de::Deserializer<'de>
    {
        deserializer.deserialize_str(DateTimeVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    #[test]
    fn format() {
        let d = DateTime::default()
            .with_year(2016)
            .and_then(|d| d.with_month(1))
            .and_then(|d| d.with_day(1))
            .and_then(|d| d.with_hour(20))
            .and_then(|d| d.with_offset(1 * 60 * 60))
            .unwrap();
        assert_eq!(d.format(), "01 January 2016 21:00:00 +0100");
    }

    #[test]
    fn parse_short_month() {
        let expected = DateTime::default()
            .with_year(2016)
            .and_then(|d| d.with_month(1))
            .and_then(|d| d.with_day(1))
            .and_then(|d| d.with_hour(20))
            .and_then(|d| d.with_offset(1 * 60 * 60))
            .unwrap();
        assert_eq!(DateTime::parse("01 Jan 2016 21:00:00 +0100").unwrap(),
                   expected);
    }

    #[test]
    fn parse_long_month() {
        let expected = DateTime::default()
            .with_year(2016)
            .and_then(|d| d.with_month(1))
            .and_then(|d| d.with_day(1))
            .and_then(|d| d.with_hour(20))
            .and_then(|d| d.with_offset(1 * 60 * 60))
            .unwrap();
        assert_eq!(DateTime::parse("01 January 2016 21:00:00 +0100").unwrap(),
                   expected);
    }
}
