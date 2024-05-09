use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct OptionalDate(pub Option<chrono::NaiveDate>);

pub(crate) struct DeserializeVisitor;

impl<'de> serde::de::Visitor<'de> for DeserializeVisitor {
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

impl fmt::Display for OptionalDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            None => write!(f, "<no date>"),
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
