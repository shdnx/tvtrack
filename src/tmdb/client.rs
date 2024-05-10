use super::{Result, SearchResults, SeriesDetails, SeriesFound, SeriesId};

static API_ROOT_URL: &str = "https://api.themoviedb.org/3/";

pub struct Client {
    agent: ureq::Agent,
    api_access_token: String,
}

impl Client {
    pub fn new(api_access_token: String) -> Client {
        Client {
            agent: ureq::AgentBuilder::new().build(),
            api_access_token,
        }
    }

    fn get(&mut self, path: &str) -> ureq::Request {
        self.agent.get(&format!("{API_ROOT_URL}{path}")).set(
            "Authorization",
            &format!("Bearer {}", self.api_access_token),
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
