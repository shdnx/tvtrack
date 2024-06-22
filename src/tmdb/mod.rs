#![allow(unused_imports, unused_import_braces)]

mod client;
mod episode;
mod optional_date;
mod search;
mod series;

pub use client::Client;
pub use episode::{EpisodeDetails, EpisodeId, EpisodeType};
pub use optional_date::OptionalDate;
pub use search::{SearchResults, SeriesFound};
pub use series::{SeriesDetails, SeriesId, SeriesStatus};
