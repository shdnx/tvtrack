use serde::{Serialize, Deserialize};
use super::{OptionalDate, SeriesId};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResults<T> {
    pub page: i32,
    pub results: Vec<T>,
    pub total_pages: i32,
    pub total_results: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SeriesFound {
    pub id: SeriesId,

    /// Title
    pub name: String,

    pub overview: String,

    pub first_air_date: OptionalDate,
    pub popularity: f64,
}