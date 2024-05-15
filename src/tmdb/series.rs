use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::{EpisodeDetails, OptionalDate};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct SeriesId(pub i32);

impl fmt::Display for SeriesId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeriesStatus {
    /// This seems to be used to mean yet-unreleased series only.
    /// Note that there's also `SeriesDetails::in_production`
    InProduction,
    ReturningSeries,
    Ended,
    Canceled,
}

impl fmt::Display for SeriesStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InProduction => write!(f, "in production"),
            Self::ReturningSeries => write!(f, "returning series"),
            Self::Ended => write!(f, "ended"),
            Self::Canceled => write!(f, "canceled"),
        }
    }
}

impl Serialize for SeriesStatus {
    fn serialize<S>(&self, s: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::InProduction => s.serialize_str("In Production"),
            Self::ReturningSeries => s.serialize_str("Returning Series"),
            Self::Ended => s.serialize_str("Ended"),
            Self::Canceled => s.serialize_str("Canceled"),
        }
    }
}

impl<'de> Deserialize<'de> for SeriesStatus {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = SeriesStatus;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    f,
                    "One of 'In Production', 'Returning Series', 'Canceled', or 'Ended'"
                )
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "In Production" => Ok(SeriesStatus::InProduction),
                    "Returning Series" => Ok(SeriesStatus::ReturningSeries),
                    "Canceled" => Ok(SeriesStatus::Canceled),
                    "Ended" => Ok(SeriesStatus::Ended),
                    val => Err(serde::de::Error::unknown_variant(
                        val,
                        &["In Production", "Returning Series", "Canceled", "Ended"],
                    )),
                }
            }
        }

        d.deserialize_str(Visitor)
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

    //pub last_air_date: OptionalDate,
    pub last_episode_to_air: Option<EpisodeDetails>,
    pub next_episode_to_air: Option<EpisodeDetails>,

    pub status: SeriesStatus,
    pub in_production: bool,
}

impl SeriesDetails {
    pub fn identify(&self) -> String {
        format!(
            "[{}] {} ({})",
            self.id,
            self.name,
            self.first_air_date.unwrap().year()
        )
    }

    pub fn next_episode_date(&self) -> Option<chrono::NaiveDate> {
        self.next_episode_to_air
            .as_ref()
            .and_then(|ep| ep.air_date.0)
    }
}
