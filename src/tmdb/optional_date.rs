use std::{cmp, fmt};

use rusqlite::types::{FromSql, ToSql};

/// Mostly equivalent to `Option<chrono::NaiveDate>` except in JSON parses an empty string to `None`.
/// This is needed because TMDB will send empty strings for missing dates instead of `null`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OptionalDate(pub Option<chrono::NaiveDate>);

impl From<chrono::NaiveDate> for OptionalDate {
    fn from(value: chrono::NaiveDate) -> Self {
        Self(Some(value))
    }
}

impl From<Option<chrono::NaiveDate>> for OptionalDate {
    fn from(value: Option<chrono::NaiveDate>) -> Self {
        Self(value)
    }
}

pub(crate) struct DeserializeVisitor;

impl serde::de::Visitor<'_> for DeserializeVisitor {
    type Value = super::OptionalDate;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a formatted date string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if value.is_empty() {
            return Ok(super::OptionalDate(None));
        }

        match value.parse::<chrono::NaiveDate>() {
            Ok(parsed_date) => Ok(super::OptionalDate(Some(parsed_date))),
            Err(parse_error) => Err(E::custom(parse_error)),
        }
    }
}

impl<'de> serde::Deserialize<'de> for OptionalDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(DeserializeVisitor)
    }
}

impl serde::Serialize for OptionalDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0 {
            None => serializer.serialize_str(""),
            Some(date) => serializer.serialize_str(&date.to_string()),
        }
    }
}

impl PartialOrd for OptionalDate {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OptionalDate {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl fmt::Display for OptionalDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            None => write!(f, "<unknown>"),
            Some(dt) => dt.fmt(f),
        }
    }
}

impl std::ops::Deref for OptionalDate {
    type Target = Option<chrono::NaiveDate>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ToSql for OptionalDate {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl FromSql for OptionalDate {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        <Option<chrono::NaiveDate> as FromSql>::column_result(value).map(OptionalDate)
    }
}
