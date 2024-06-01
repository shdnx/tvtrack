use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::result::Result as AnyResult;
use crate::tmdb;

#[derive(Debug, Default)]
pub struct ApplicationState {
    pub tracked_series: HashMap<tmdb::SeriesId, SeriesState>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonApplicationState {
    tracked_series: Vec<SeriesState>,
}

impl ApplicationState {
    pub fn new() -> ApplicationState {
        ApplicationState {
            tracked_series: HashMap::new(),
        }
    }

    /// Read the state file from the specified path if it exists, otherwise return a new (empty) one.
    pub fn read_from_or_new(file_path: &str) -> AnyResult<ApplicationState> {
        match std::fs::read_to_string(file_path) {
            Ok(state_str) => {
                println!("Loading state file {file_path}");
                let state_json = serde_json::from_str::<JsonApplicationState>(&state_str)?;
                Ok(Self::from_json(state_json))
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                println!("Note: state file {file_path} does not exist, loading empty state");
                Ok(ApplicationState::new())
            }
            Err(err) => Err(err.into()),
        }
    }

    pub fn write_to(&self, file_path: &str) -> AnyResult<()> {
        let state_str = serde_json::to_string_pretty(&self.to_json())?;
        std::fs::write(file_path, state_str)?;
        Ok(())
    }

    fn to_json(&self) -> JsonApplicationState {
        JsonApplicationState {
            tracked_series: self
                .tracked_series
                .values()
                .map(SeriesState::clone)
                .collect(),
        }
    }

    fn from_json(state_json: JsonApplicationState) -> ApplicationState {
        ApplicationState {
            tracked_series: state_json
                .tracked_series
                .into_iter()
                .map(|ts| (ts.details.id, ts))
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SeriesState {
    pub details: tmdb::SeriesDetails,
    /// The timestamp when this series was last updated
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The timestamp after which we should again poll this series for updates
    pub next_update_timestamp: chrono::DateTime<chrono::Utc>,
}
