use super::{Result, SearchResults, SeriesDetails, SeriesFound, SeriesId};

static API_ROOT_URL: &str = "https://api.themoviedb.org/3/";

pub struct Client {
    agent: ureq::Agent,
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

    pub fn make_poster_url(&self, poster_path: &str) -> String {
        // TODO: we should be getting the base url and the image width closest to what we want from the TMDB API; see https://developer.themoviedb.org/docs/image-basics
        // TODO: we should probably cache the poster image for series that are tracked, but that only works once this is public and/or has a domain associated with it
        format!(
            "https://image.tmdb.org/t/p/w92{}?api_key={}",
            poster_path, self.api_key
        )
    }

    pub fn search_series(
        &mut self,
        title: &str,
        first_air_year: Option<i32>,
    ) -> Result<SearchResults<SeriesFound>> {
        let result_json = {
            let mut query = self
                .get("search/tv")
                .query("query", title)
                .query("page", "1");

            if let Some(year) = first_air_year {
                query = query.query("first_air_date_year", &year.to_string());
            }

            query.call()?.into_string()?
        };

        serde_json::from_str::<SearchResults<SeriesFound>>(&result_json).map_err(|json_err| {
            println!(
                "search_series {title} failed to parse JSON response: {json_err:?} {}",
                &result_json
            );
            super::Error::from(json_err)
        })
    }

    pub fn get_series_details(&mut self, id: SeriesId) -> Result<SeriesDetails> {
        let result_json = self.get(&format!("tv/{id}")).call()?.into_string()?;

        serde_json::from_str::<SeriesDetails>(&result_json).map_err(|json_err| {
            println!(
                "get_series_details {id} failed to parse JSON response: {json_err:?} {}",
                &result_json
            );
            super::Error::from(json_err)
        })
    }
}
