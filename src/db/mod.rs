mod poster;
mod series;
mod table_model;
mod user;

pub use self::poster::{Poster, PosterId};
pub use self::series::Series;
pub use self::user::User;

use self::table_model::TableModel;
use crate::tmdb;
use anyhow::Context;

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
            .query_row_and_then(&sql, (id,), |row| {
                T::from_full_row(row).with_context(|| {
                    format!(
                        "Deserializing {} row with id = {id}: {row:?}",
                        T::table_name()
                    )
                })
            })
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

        let rows = stmt
            .query_and_then((), |row| {
                T::from_full_row(row).with_context(|| {
                    format!("Error deserializing {} row: {row:?}", T::table_name())
                })
            })
            .with_context(|| format!("Querying all {}", T::table_name()))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn insert_series(&mut self, new_series: &Series) -> anyhow::Result<()> {
        self.conn.execute(
            "INSERT INTO series (tmdb_id, title, first_air_date, poster_id, status, in_production, last_episode_air_date, next_episode_air_date, details, update_timestamp) VALUES (:tmdb_id, :title, :first_air_date, :poster_id, :status, :in_production, :last_episode_air_date, :next_episode_air_date, :details, :update_timestamp)",
            rusqlite::named_params! {
                ":tmdb_id": new_series.tmdb_id,
                ":title": new_series.title,
                ":first_air_date": new_series.first_air_date,
                ":poster_id": new_series.poster_id,
                ":status": new_series.status,
                ":in_production": new_series.in_production,
                ":last_episode_air_date": new_series.last_episode_air_date,
                ":next_episode_air_date": new_series.next_episode_air_date,
                ":details": new_series.details_json,
                ":update_timestamp": new_series.update_timestamp,
            }
        ).with_context(|| format!("Inserting series {} into the database: {:?}", new_series.details.identify(), new_series))?;
        Ok(())
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

        let rows = stmt.query_and_then((series_id,), |row| {
            User::from_full_row(row).with_context(|| format!("deserializing user row {row:?}"))
        })?;

        let mut result = Vec::new();
        for (row_idx, row) in rows.into_iter().enumerate() {
            result.push(row.with_context(|| format!("Error deserializing user row {row_idx}"))?);
        }
        Ok(result)
    }
}
