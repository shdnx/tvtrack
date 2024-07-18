use anyhow::{bail, Context};
use lettre::message::header::ContentType;
use rusqlite::types::{FromSql, ToSql};
use std::{borrow::Borrow, fmt, path::Path};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct MimeType(String);

impl MimeType {
    fn file_ext_to_mime_type(ext: &str) -> Option<&'static str> {
        // TODO: how to do this better?
        let result = match ext {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            _ => {
                return None;
            }
        };
        Some(result)
    }

    pub fn validate(value: &str) -> anyhow::Result<()> {
        // if lettre accepts it as a Content-Type header, then that's good enough for us
        let _ =
            ContentType::parse(value).with_context(|| format!("Invalid MIME type: {value:?}"))?;
        Ok(())
    }

    pub fn new(value: String) -> anyhow::Result<Self> {
        Self::validate(&value)?;
        Ok(MimeType(value))
    }

    pub fn identify_from_ext(path: &str) -> anyhow::Result<Self> {
        Self::try_from(Path::new(path))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

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
        let value_str = <String as FromSql>::column_result(value)?;
        MimeType::new(value_str).map_err(|err| rusqlite::types::FromSqlError::Other(err.into()))
    }
}

impl TryFrom<&str> for MimeType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        MimeType::validate(value)?;
        Ok(MimeType(value.to_owned()))
    }
}

impl TryFrom<&Path> for MimeType {
    type Error = anyhow::Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let ext = value
            .extension()
            .and_then(|ext| ext.to_str())
            .with_context(|| format!("File path {value:?} does not have a valid extension"))?;

        let Some(mime_type) = Self::file_ext_to_mime_type(ext) else {
            bail!("Could not determine MIME type for path {value:?} with unknown extension {ext}");
        };
        Self::try_from(mime_type)
    }
}

impl Borrow<str> for MimeType {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for MimeType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<MimeType> for ContentType {
    fn from(value: MimeType) -> Self {
        ContentType::parse(value.as_str()).expect("Should have been checked-valid already")
    }
}
