use std::fmt;

use anyhow::Context;
use rusqlite::types::{FromSql, ToSql};

use crate::tmdb::{self, MimeType, OptionalDate};

pub struct Db {
    pub conn: rusqlite::Connection,
}

impl Db {
    pub fn open(file_path: &str) -> anyhow::Result<Self> {
        let conn = rusqlite::Connection::open_with_flags(
            file_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
        )
        .with_context(|| format!("Failed to open SQLite DB: {file_path}"))?;
        Ok(Self { conn })
    }

    pub fn optional_single_row_result<T>(result: anyhow::Result<T>) -> anyhow::Result<Option<T>> {
        match result {
            Ok(row) => Ok(Some(row)),
            Err(err) => match err.downcast::<rusqlite::Error>() {
                Ok(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Ok(rusqlite_err) => Err(rusqlite_err.into()),
                Err(err) => Err(err),
            },
        }
    }

    pub fn get_poster_by_id(&mut self, id: PosterId) -> anyhow::Result<Option<Poster>> {
        let result = self
            .conn
            .query_row_and_then(
                "SELECT * FROM posters WHERE id = ? LIMIT 1",
                (id,),
                Poster::from_full_row,
            )
            .with_context(|| format!("Querying poster with ID {id}"));

        Self::optional_single_row_result(result)
    }

    pub fn get_series_by_id(&mut self, id: tmdb::SeriesId) -> anyhow::Result<Option<Series>> {
        let result = self
            .conn
            .query_row_and_then(
                "SELECT * FROM series WHERE tmdb_id = ? LIMIT 1",
                (id,),
                Series::from_full_row,
            )
            .with_context(|| format!("Querying series with ID {id}"));

        Self::optional_single_row_result(result)
    }

    pub fn get_all_series(&mut self) -> anyhow::Result<Vec<Series>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM series")
            .context("Querying all series")?;

        let rows = stmt.query_and_then((), Series::from_full_row)?;
        let mut result = Vec::new();
        for (row_idx, row) in rows.into_iter().enumerate() {
            result.push(row.with_context(|| format!("Error deserializing series row {row_idx}"))?);
        }
        Ok(result)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct PosterId(pub i64);

impl fmt::Display for PosterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl ToSql for PosterId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl FromSql for PosterId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        <i64 as FromSql>::column_result(value).map(PosterId)
    }
}

#[derive(Debug)]
pub struct Poster {
    pub id: PosterId,
    pub img_data: Box<[u8]>,
    pub mime_type: MimeType,
    pub source_url: Option<String>, // TODO: shouldn't remain optional once the DB is migrated and we have the source url for everything
}

impl Poster {
    pub fn from_full_row(row: &rusqlite::Row) -> anyhow::Result<Self> {
        let result = Poster {
            id: row.get("id")?,
            img_data: row.get::<_, Vec<u8>>("img_data")?.into_boxed_slice(),
            mime_type: row.get("mime_type")?,
            source_url: row.get("source_url")?,
        };
        Ok(result)
    }
}

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
    pub fn from_full_row(row: &rusqlite::Row) -> anyhow::Result<Self> {
        let raw_details = row.get::<_, serde_json::Value>(8)?;
        let result = Series {
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
