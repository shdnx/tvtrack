use serde::{Deserialize, Serialize};
use std::fmt;

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
            Self::Standard => write!(f, "Standard"),
            Self::Finale => write!(f, "Finale"),
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

impl EpisodeDetails {
    // TODO: return a proxy value instead that can be formatted by implementing Display?
    pub fn identify(&self) -> String {
        let mut result = format!("S{:02}E{:02} {} on {}", self.season_number, self.episode_number, self.name, self.air_date);
        if self.episode_type != EpisodeType::Standard {
            result += format!(" {}", self.episode_type).as_ref();
        }
        result
    }
}