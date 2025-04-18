use chrono::Datelike;
use rusqlite::types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use std::fmt;
use strum::{VariantArray, VariantNames};

use super::{EpisodeDetails, OptionalDate};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct SeriesId(pub i32);

impl fmt::Display for SeriesId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ToSql for SeriesId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl FromSql for SeriesId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        <i32 as FromSql>::column_result(value).map(SeriesId)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString, strum::VariantNames,
)]
pub enum SeriesStatus {
    /// This seems to be used to mean yet-unreleased series only.
    /// Note that there's also `SeriesDetails::in_production`
    #[strum(to_string = "In Production")]
    InProduction,

    #[strum(to_string = "Returning Series")]
    ReturningSeries,

    Ended,
    Canceled,
}

impl Serialize for SeriesStatus {
    fn serialize<S>(&self, s: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SeriesStatus {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = SeriesStatus;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "A valid SeriesStatus")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse::<SeriesStatus>()
                    .map_err(|_| serde::de::Error::unknown_variant(v, SeriesStatus::VARIANTS))
            }
        }

        d.deserialize_str(Visitor)
    }
}

impl ToSql for SeriesStatus {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(self.to_string().into())
    }
}

impl FromSql for SeriesStatus {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        value
            .as_str()?
            .parse()
            .map_err(|e| rusqlite::types::FromSqlError::Other(Box::new(e)))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SeriesDetails {
    pub id: SeriesId,

    /// Title
    pub name: String,
    pub first_air_date: OptionalDate,

    pub number_of_seasons: i32,
    pub number_of_episodes: i32,

    pub last_episode_to_air: Option<EpisodeDetails>,
    pub next_episode_to_air: Option<EpisodeDetails>,

    pub status: SeriesStatus,
    pub in_production: bool,

    pub poster_path: String,
}

impl SeriesDetails {
    // TODO: deprecate
    pub fn identify(&self) -> String {
        self.to_string()
    }

    pub fn last_episode_date(&self) -> OptionalDate {
        self.last_episode_to_air
            .as_ref()
            .and_then(|ep| ep.air_date.0)
            .into()
    }

    pub fn next_episode_date(&self) -> OptionalDate {
        self.next_episode_to_air
            .as_ref()
            .and_then(|ep| ep.air_date.0)
            .into()
    }

    pub fn poster_extension(&self) -> Option<&str> {
        std::path::Path::new(&self.poster_path)
            .extension()
            .and_then(|ext| ext.to_str())
    }
}

impl std::fmt::Display for SeriesDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(first_air_date) = self.first_air_date.0 {
            write!(f, "[{}] {} ({})", self.id, self.name, first_air_date.year())
        } else {
            write!(f, "[{}] {} (unreleased)", self.id, self.name)
        }
    }
}
