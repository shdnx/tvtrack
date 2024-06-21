use std::path::Path;

use anyhow::{anyhow, Context};
use lettre::message::header::ContentType;

use super::{SearchResults, SeriesDetails, SeriesFound, SeriesId};

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
            agent: ureq::AgentBuilder::new().build(),
            api_key,
            api_access_token,
        }
    }

    fn get(&mut self, path: &str) -> ureq::Request {
        self.agent.get(&format!("{API_ROOT_URL}{path}")).set(
            "Authorization",
            &format!("Bearer {}", self.api_access_token),
        )
    }

    pub fn make_series_url(&self, id: SeriesId) -> String {
        format!("https://www.themoviedb.org/tv/{id}")
    }

    // TODO: move somewhere else
    pub fn try_determine_mime_type(path: &str) -> anyhow::Result<&'static str> {
        let ext = Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .with_context(|| format!("File path {path} does not have a valid extension"))?;

        match ext {
            "jpg" | "jpeg" => Ok("image/jpeg"),
            "png" => Ok("image/png"),
            _ => Err(anyhow!(
                "Could not determine MIME type for path {path} with unknown extension {ext}"
            )),
        }
    }

    pub fn get_poster(&self, path: &str) -> anyhow::Result<(Box<[u8]>, &'static str)> {
        // TODO: we should be getting the base url and the image width closest to what we want from the TMDB API; see https://developer.themoviedb.org/docs/image-basics
        let url = format!("https://image.tmdb.org/t/p/w92{path}");
        let mime_type = Self::try_determine_mime_type(path)?;

        let mut buf = vec![];
        self.agent
            .get(&url)
            .set(
                "Authorization",
                &format!("Bearer {}", self.api_access_token),
            )
            .set("Accept", mime_type)
            .call()
            .with_context(|| format!("TMDB::get_poster({path:?})"))?
            .into_reader()
            .read_to_end(&mut buf)?;

        Ok((buf.into_boxed_slice(), mime_type))
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
                query = query.query("first_air_date_year", &year.to_string());
            }

            query
                .call()
                .with_context(|| format!("TMDB::search_series({title:?}, {first_air_year:?})"))?
                .into_string()?
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
            .into_string()?;

        serde_json::from_str::<SeriesDetails>(&result_json).with_context(|| {
            format!(
                "TMDB::get_series_details({id}) JSON parse error: {}",
                &result_json
            )
        })
    }
}
