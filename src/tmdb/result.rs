use std::fmt;

#[derive(Debug)]
pub enum Error {
    Status(u16, String),
    Json(serde_json::Error), // TODO: remove?
    Other(Box<dyn std::error::Error>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Status(code, message) => write!(f, "API: {code} {message}"),
            Error::Json(error) => write!(f, "JSON: {error}"),
            Error::Other(error) => write!(f, "Other: {error}"),
        }
    }
}

impl From<ureq::Error> for Error {
    fn from(value: ureq::Error) -> Self {
        match value {
            ureq::Error::Status(status, response) => {
                Error::Status(status, response.into_string().unwrap())
            }
            _ => Error::Other(value.into()),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Json(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Other(value.into())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
