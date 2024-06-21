use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StateFilePath(pub String);

impl Default for StateFilePath {
    fn default() -> Self {
        Self("tvtrack.state.json".to_owned())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub state_file_path: StateFilePath,

    pub tmdb: TMDBConfig,
    pub smtp: SMTPConfig,
    pub emails: EmailsConfig,
}

impl AppConfig {
    pub fn try_read(file_path: String) -> anyhow::Result<AppConfig> {
        let json = &std::fs::read_to_string(&file_path)
            .with_context(|| format!("Reading config file {file_path:?}"))?;
        serde_json::from_str::<AppConfig>(json)
            .with_context(|| format!("Parsing JSON config file {file_path:?}"))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TMDBConfig {
    pub api_key: String,
    pub api_access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SMTPConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailsConfig {
    #[serde(default)]
    pub from_name: Option<String>,
    pub from_address: String,
    #[serde(default)]
    pub to_name: Option<String>,
    pub to_address: String,
}
