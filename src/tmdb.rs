mod optional_date;
use optional_date::OptionalDate;

mod result;
pub use result::{Error, Result};

mod client;
pub use client::Client;

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct SeriesId(i32);

impl fmt::Display for SeriesId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct EpisodeId(i32);

impl fmt::Display for EpisodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EpisodeDetails {
    pub id: EpisodeId,

    pub season_number: i32,
    pub episode_number: i32,

    /// Title
    pub name: String,

    /// "standard" or "finale"
    pub episode_type: String,

    pub air_date: OptionalDate,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SeriesFound {
    pub id: SeriesId,

    /// Title
    pub name: String,

    pub overview: String,

    pub first_air_date: OptionalDate,
    pub popularity: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SeriesDetails {
    pub id: SeriesId,

    /// Title
    pub name: String,

    pub number_of_seasons: i32,
    pub number_of_episodes: i32,

    //pub last_air_date: OptionalDate,
    pub last_episode_to_air: Option<EpisodeDetails>,
    pub next_episode_to_air: Option<EpisodeDetails>,

    /// "Returning Series", "Ended", "Canceled"
    pub status: String,
    pub in_production: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResults<T> {
    pub page: i32,
    pub results: Vec<T>,
    pub total_pages: i32,
    pub total_results: i32,
}
