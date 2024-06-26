mod add;
mod config;
mod context;
mod db;
mod notify;
mod tmdb;
mod update;

use anyhow::bail;
use config::AppConfig;
use context::AppContext;
use db::Db;
use tmdb::{EpisodeDetails, SeriesDetails, SeriesId, SeriesStatus};
use update::SeriesDetailsChanges;

fn print_help() {
    println!(
        r#"Available commands:
 - a | add <title> [<release year>]: register a series with the specified title to track.
 - add-id <id>: add a series by TMDB ID.
 - add-all <file>: read the text file at the specified path, interpreting each line as a title to add as if individually done with the 'add' command.
 - update [<id>] [-f | --force]: update one specific or all tracked series, checking for updates by fetching the details from TMDB. Trigger a notification on the summary of changes. Force performs the update regardless of how much time passed since the last update.
 - h | help: show this help message.
"#
    );
}

fn main() -> anyhow::Result<()> {
    // TODO: take optionally as a command line argument, only check env if that is not present
    let config = {
        let file_path = std::env::var("TVTRACK_CONFIG_FILE").expect("TVTRACK_CONFIG_FILE not set");
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

    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let command = args.first().map(String::as_str).unwrap_or("help");

    match command {
        "a" | "add" => {
            let title = args.get(1).expect("Expected title of series to add");
            let first_air_year = args.get(2).and_then(|a| a.parse::<i32>().ok());
            add::add_series(&mut ctx, title, first_air_year)?;
        }
        "add-id" => {
            let series_id = SeriesId(
                args.get(1)
                    .and_then(|a| a.parse::<i32>().ok())
                    .expect("Expected TMDB series ID to add"),
            );
            add::add_series_by_id(&mut ctx, series_id)?;
        }
        "add-all" => {
            let file_path = args
                .get(1)
                .expect("Expected path to file containing series titles to add");
            add::add_all_series(&mut ctx, file_path)?;
        }
        "u" | "update" => {
            let all_series_changes = match args.get(1).map(|s| s.as_ref()) {
                Some(series_id) if series_id.parse::<i32>().is_ok() => {
                    let series_id =
                        SeriesId(series_id.parse::<i32>().expect("Series ID to update"));

                    let Some(mut series) = ctx.db.get_series_by_id(series_id)? else {
                        bail!("No tracked series with ID {series_id}")
                    };

                    // TODO: not great arg handling
                    let force = args
                        .get(2)
                        .map(|a| a == "-f" || a == "--force")
                        .unwrap_or(false);

                    match update::update_one_series(&mut ctx, &mut series, force)? {
                        None => Vec::new(),
                        Some(changes) => vec![(series, changes)],
                    }
                }
                Some("-f") | Some("--force") => update::update_all_series(&mut ctx, true)?,
                Some(unknown_arg) => bail!("Unknown argument {unknown_arg}"),
                None => update::update_all_series(&mut ctx, false)?,
            };

            // TODO: allow notifications to be only printed, for testing/debugging
            if !all_series_changes.is_empty() {
                notify::send_email_notifications(&mut ctx, all_series_changes)?;
            }
        }
        "h" | "help" | "-h" | "--help" => {
            print_help();
        }
        _ => {
            eprintln!("Error: unrecognized command {command}\n");
            print_help();
        }
    };

    Ok(())
}
