use super::poster::PosterId;
use super::table_model::TableModel;
use crate::tmdb::{self, OptionalDate};
use anyhow::Context;

#[derive(Debug)]
pub struct Series {
    pub tmdb_id: tmdb::SeriesId,
    pub poster_id: PosterId,

    pub title: String,
    pub first_air_date: OptionalDate,
    pub status: tmdb::SeriesStatus,
    pub in_production: bool,
    pub last_episode_air_date: OptionalDate,
    pub next_episode_air_date: OptionalDate,

    pub details: tmdb::SeriesDetails,
    pub details_json: serde_json::Value,

    pub update_timestamp: chrono::DateTime<chrono::Utc>,
}

impl Series {
    pub fn set_details(
        &mut self,
        new_details: tmdb::SeriesDetails,
        update_timestamp: chrono::DateTime<chrono::Utc>,
    ) {
        self.in_production = new_details.in_production;
        self.status = new_details.status;
        self.last_episode_air_date = new_details.last_episode_date();
        self.next_episode_air_date = new_details.next_episode_date();
        self.details_json = serde_json::to_value(new_details.clone()).unwrap();
        self.details = new_details;
        self.update_timestamp = update_timestamp;
    }
}

impl TableModel for Series {
    fn table_name() -> &'static str {
        "series"
    }

    fn from_full_row(row: &rusqlite::Row) -> anyhow::Result<Self> {
        let raw_details = row.get::<_, serde_json::Value>("details")?;
        let result = Self {
            tmdb_id: row.get("tmdb_id")?,
            poster_id: row.get("poster_id")?,
            title: row.get("title")?,
            first_air_date: row.get("first_air_date")?,
            status: row.get("status")?,
            in_production: row.get("in_production")?,
            last_episode_air_date: row.get("last_episode_air_date")?,
            next_episode_air_date: row.get("next_episode_air_date")?,
            details: serde_json::from_value::<tmdb::SeriesDetails>(raw_details.clone())
                .context("Deserializing tmdb::SeriesDetails from series.details")?,
            details_json: raw_details,
            update_timestamp: row.get("update_timestamp")?,
        };
        Ok(result)
    }
}
