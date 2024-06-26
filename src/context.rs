use crate::{config::AppConfig, db::Db, tmdb};

pub struct AppContext {
    pub config: AppConfig,
    pub db: Db,
    pub tmdb: tmdb::Client,
}
