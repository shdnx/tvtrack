use std::{fmt, io::Read};

use anyhow::{bail, Context};
use rusqlite::types::{FromSql, ToSql};

use crate::tmdb;

// TODO: this is a mess, move poster and mime-type stuff from tmdb::Client here

#[derive(Debug, PartialEq, Eq)]
pub struct MimeType(pub String);

impl fmt::Display for MimeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl ToSql for MimeType {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl FromSql for MimeType {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        <String as FromSql>::column_result(value).map(MimeType)
    }
}

pub fn fetch_poster_image(
    tmdb_client: &mut tmdb::Client,
    series: &tmdb::SeriesDetails,
) -> anyhow::Result<(Box<[u8]>, MimeType)> {
    // TODO: once we switch to SQLite, store the images inside
    let file_ext = series
        .poster_extension()
        .expect("Poster path without valid extension?");

    let cache_dir_path = "posters-cache";
    let cache_file_path = format!("{cache_dir_path}/{}.{file_ext}", series.id);

    match std::fs::create_dir(cache_dir_path) {
        Ok(()) => (),
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => (),
        // TODO: is this a good way?
        Err(err) => bail!("Posters cache directory could not be created: {err}"),
    };

    let (data, mime_type) = match std::fs::File::open(&cache_file_path) {
        Ok(mut file) => {
            let mut data: Vec<u8> = vec![];
            file.read_to_end(&mut data)
                .with_context(|| format!("Reading poster cache file {cache_file_path}"))?;

            let mime_type = tmdb::Client::try_determine_mime_type(&cache_file_path)?;
            (data.into_boxed_slice(), mime_type)
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            let (data, mime_type) = tmdb_client.get_poster(&series.poster_path)?;
            std::fs::write(&cache_file_path, &data)
                .with_context(|| format!("Writing poster cache file {cache_file_path}"))?;
            (data, mime_type)
        }
        Err(err) => return Err(err.into()),
    };

    Ok((data, MimeType(mime_type.to_owned())))
}
