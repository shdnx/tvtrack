use super::table_model::TableModel;
use crate::tmdb::MimeType;
use rusqlite::types::{FromSql, ToSql};
use std::fmt;

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
