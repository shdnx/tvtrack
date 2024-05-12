use serde::{Deserialize, Serialize};
use std::{default, fmt};

use super::OptionalDate;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct EpisodeId(pub i32);

impl fmt::Display for EpisodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EpisodeType {
    #[default]
    Standard,
    Finale,
}

impl fmt::Display for EpisodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::Finale => write!(f, "finale"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EpisodeDetails {
    pub id: EpisodeId,

    pub season_number: i32,
    pub episode_number: i32,

    /// Title
    pub name: String,

    pub episode_type: EpisodeType,
    pub air_date: OptionalDate,
}
