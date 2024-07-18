use std::fmt;

use anyhow::Context;
use rusqlite::types::{FromSql, ToSql};

use crate::tmdb::{self, MimeType, OptionalDate};

pub trait TableModel: Sized {
    fn table_name() -> &'static str;
    fn from_full_row(row: &rusqlite::Row) -> anyhow::Result<Self>;
}

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

    pub fn get_by_id<T: TableModel>(&mut self, id: i64) -> anyhow::Result<Option<T>> {
        let sql = format!("SELECT * FROM {} WHERE id = ? LIMIT 1", T::table_name());

        let result = self
            .conn
            .query_row_and_then(&sql, (id,), T::from_full_row)
            .with_context(|| format!("Querying {} for ID {id}", T::table_name()));

        Self::optional_single_row_result(result)
    }

    pub fn get_poster_by_id(&mut self, id: PosterId) -> anyhow::Result<Option<Poster>> {
        self.get_by_id::<Poster>(id.0)
    }

    pub fn get_series_by_id(&mut self, id: tmdb::SeriesId) -> anyhow::Result<Option<Series>> {
        self.get_by_id::<Series>(id.0 as i64)
    }

    pub fn get_all<T: TableModel>(&mut self) -> anyhow::Result<Vec<T>> {
        let sql = format!("SELECT * FROM {}", T::table_name());
        let mut stmt = self
            .conn
            .prepare(&sql)
            .with_context(|| format!("Querying all {}", T::table_name()))?;

        let rows = stmt.query_and_then((), T::from_full_row)?;
        let mut result = Vec::new();
        for (row_idx, row) in rows.into_iter().enumerate() {
            result.push(row.with_context(|| {
                format!("Error deserializing {} row {row_idx}", T::table_name())
            })?);
        }
        Ok(result)
    }

    pub fn get_all_series(&mut self) -> anyhow::Result<Vec<Series>> {
        self.get_all::<Series>()
    }

    pub fn get_all_users_subscribed_to_series(
        &mut self,
        series_id: tmdb::SeriesId,
    ) -> anyhow::Result<Vec<User>> {
        let mut stmt = self
            .conn
            .prepare("SELECT users.id AS id, users.name AS name, users.email AS email FROM tracked_series INNER JOIN users ON tracked_series.user_id = users.id WHERE series_tmdb_id = ?")
            .with_context(|| format!("Querying all users subscribed to series {}", series_id))?;

        let rows = stmt.query_and_then((series_id,), User::from_full_row)?;

        let mut result = Vec::new();
        for (row_idx, row) in rows.into_iter().enumerate() {
            result.push(row.with_context(|| format!("Error deserializing user row {row_idx}"))?);
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

impl TableModel for Poster {
    fn table_name() -> &'static str {
        "posters"
    }

    fn from_full_row(row: &rusqlite::Row) -> anyhow::Result<Self> {
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

impl TableModel for Series {
    fn table_name() -> &'static str {
        "series"
    }

    fn from_full_row(row: &rusqlite::Row) -> anyhow::Result<Self> {
        let raw_details = row.get::<_, serde_json::Value>(8)?;
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

#[derive(Debug)]
pub struct User {
    pub id: i64, // TODO: UserId
    pub name: String,
    pub email: String,
}

impl TableModel for User {
    fn table_name() -> &'static str {
        "users"
    }

    fn from_full_row(row: &rusqlite::Row) -> anyhow::Result<Self> {
        let result = Self {
            id: row.get("id")?,
            name: row.get("name")?,
            email: row.get("email")?,
        };
        Ok(result)
    }
}
