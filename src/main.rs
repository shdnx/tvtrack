mod add;
mod cli;
mod config;
mod context;
mod db;
mod notify;
mod tmdb;
mod update;

use std::path::PathBuf;

use anyhow::bail;
use clap::Parser;
use config::AppConfig;
use context::AppContext;
use db::Db;
use tmdb::{EpisodeDetails, SeriesDetails, SeriesId, SeriesStatus};
use update::SeriesDetailsChanges;

fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();

    let config = {
        let file_path = match args.config {
            Some(file_path) => file_path,
            None => PathBuf::from(
                std::env::var("TVTRACK_CONFIG_FILE").expect("TVTRACK_CONFIG_FILE not set"),
            ),
        };
        AppConfig::try_read(&file_path)?
    };

    let mut ctx = {
        let db = Db::open(&config.state_file_path.0)?;

        // TODO: this is a bit ugly, just pass &config.tmdb instead?
        let tmdb_client = tmdb::Client::new(
            config.tmdb.api_key.clone(),
            config.tmdb.api_access_token.clone(),
        );

        AppContext {
            config,
            db,
            tmdb: tmdb_client,
        }
    };

    match &args.command {
        cli::Command::AddByTitle {
            title,
            first_air_year,
        } => {
            add::add_series(&mut ctx, title, *first_air_year)?;
        }
        cli::Command::AddById { tmdb_id } => {
            add::add_series_by_id(&mut ctx, SeriesId(*tmdb_id))?;
        }
        cli::Command::AddFrom { file_path } => {
            add::add_all_series(&mut ctx, file_path)?;
        }
        cli::Command::Update { tmdb_id, force } => {
            let force = force.unwrap_or(false);

            let all_series_changes = if let Some(series_id) = tmdb_id {
                let series_id = SeriesId(*series_id);

                let Some(mut series) = ctx.db.get_series_by_id(series_id)? else {
                    bail!("No tracked series with ID {series_id}")
                };

                match update::update_one_series(&mut ctx, &mut series, force)? {
                    None => Vec::new(),
                    Some(changes) => vec![(series, changes)],
                }
            } else {
                update::update_all_series(&mut ctx, force)?
            };

            // TODO: allow notifications to be only printed, for testing/debugging
            if !all_series_changes.is_empty() {
                notify::send_email_notifications(&mut ctx, all_series_changes)?;
            }
        }
    };

    Ok(())
}
