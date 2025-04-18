use std::{io::Read, path::Path};

use anyhow::{Context, anyhow};
use lettre::message::header::ContentType;

use super::{MimeType, Poster, SearchResults, SeriesDetails, SeriesFound, SeriesId};

static API_ROOT_URL: &str = "https://api.themoviedb.org/3/";

pub struct Client {
    agent: ureq::Agent,
    #[allow(dead_code)]
    api_key: String,
    api_access_token: String,
}

impl Client {
    pub fn new(api_key: String, api_access_token: String) -> Client {
        Client {
            agent: ureq::Agent::new_with_defaults(),
            api_key,
            api_access_token,
        }
    }

    fn get(&mut self, path: &str) -> ureq::RequestBuilder<ureq::typestate::WithoutBody> {
        self.agent.get(&format!("{API_ROOT_URL}{path}")).header(
            "Authorization",
            &format!("Bearer {}", self.api_access_token),
        )
    }

    pub fn make_series_url(&self, id: SeriesId) -> String {
        format!("https://www.themoviedb.org/tv/{id}")
    }

    pub fn get_poster(&self, path: &str) -> anyhow::Result<Poster> {
        // TODO: we should be getting the base url and the image width closest to what we want from the TMDB API; see https://developer.themoviedb.org/docs/image-basics
        // TODO: use bigger posters, like w154 or w185, and in the email, align them below the titles
        let url = format!("https://image.tmdb.org/t/p/w92{path}");
        let mime_type = MimeType::identify_from_ext(path)?;

        let mut buf = vec![];
        self.agent
            .get(&url)
            .header(
                "Authorization",
                &format!("Bearer {}", self.api_access_token),
            )
            .header("Accept", mime_type.as_str())
            .call()
            .with_context(|| format!("TMDB::get_poster({path:?})"))?
            .into_body()
            .into_reader()
            .read_to_end(&mut buf)?;

        Ok(Poster {
            img_data: buf.into_boxed_slice(),
            mime_type,
            source_url: url,
        })
    }

    pub fn search_series(
        &mut self,
        title: &str,
        first_air_year: Option<i32>,
    ) -> anyhow::Result<SearchResults<SeriesFound>> {
        let result_json = {
            let mut query = self
                .get("search/tv")
                .query("query", title)
                .query("page", "1");

            if let Some(year) = first_air_year {
                query = query.query("first_air_date_year", year.to_string());
            }

            query
                .call()
                .with_context(|| format!("TMDB::search_series({title:?}, {first_air_year:?})"))?
                .into_body()
                .read_to_string()?
        };

        serde_json::from_str::<SearchResults<SeriesFound>>(&result_json).with_context(|| {
            format!(
                "TMDB::search_series({title:?}, {first_air_year:?}) JSON parse error: {}",
                &result_json
            )
        })
    }

    pub fn get_series_details(&mut self, id: SeriesId) -> anyhow::Result<SeriesDetails> {
        let result_json = self
            .get(&format!("tv/{id}"))
            .call()
            .with_context(|| format!("TMDB::get_series_details({id})"))?
            .into_body()
            .read_to_string()?;

        serde_json::from_str::<SeriesDetails>(&result_json).with_context(|| {
            format!(
                "TMDB::get_series_details({id}) JSON parse error: {}",
                &result_json
            )
        })
    }
}
