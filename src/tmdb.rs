mod client;
mod episode;
mod optional_date;
mod result;
mod search;
mod series;

pub use client::Client;
pub use episode::{EpisodeDetails, EpisodeId, EpisodeType};
pub use optional_date::OptionalDate;
pub use result::{Error, Result};
pub use search::{SearchResults, SeriesFound};
pub use series::{SeriesDetails, SeriesId, SeriesStatus};
