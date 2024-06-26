#![allow(unused_imports, unused_import_braces)]

mod client;
mod episode;
mod mime_type;
mod optional_date;
mod poster;
mod search;
mod series;

pub use client::Client;
pub use episode::{EpisodeDetails, EpisodeId, EpisodeType};
pub use mime_type::MimeType;
pub use optional_date::OptionalDate;
pub use poster::Poster;
pub use search::{SearchResults, SeriesFound};
pub use series::{SeriesDetails, SeriesId, SeriesStatus};
