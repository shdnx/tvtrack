mod add;
mod config;
mod notify;
mod result;
mod state;
mod tmdb;
mod update;

use config::AppConfig;
use result::{AnyError, Result};
use state::{ApplicationState, SeriesState};
use tmdb::{EpisodeDetails, SeriesDetails, SeriesId, SeriesStatus};
use update::SeriesDetailsChanges;

struct CmdContext {
    config: AppConfig,
    tmdb_client: tmdb::Client,
    now: chrono::DateTime<chrono::Utc>,
    app_state_changed: bool,
}

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

fn main() -> Result<()> {
    // TODO: take optionally as a command line argument, only check env if that is not present
    let config = std::env::var("TVTRACK_CONFIG_FILE")
        .map_err(AnyError::from)
        .and_then(config::AppConfig::try_read)
        .inspect_err(|err| {
            eprintln!("Error: TVTRACK_CONFIG_FILE references invalid file: {err:?}");
        })?;

    let mut app_state = ApplicationState::read_from_or_new(&config.state_file_path.0)?;
    let mut ctx = {
        // TODO: this is a bit ugly, just pass &config.tmdb instead?
        let tmdb_client = tmdb::Client::new(
            config.tmdb.api_key.clone(),
            config.tmdb.api_access_token.clone(),
        );

        CmdContext {
            config,
            tmdb_client,
            now: chrono::Utc::now(), // used to ensure that all series update timestamps are exactly the same if they are updated together
            app_state_changed: false,
        }
    };

    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let command = args.first().map(String::as_str).unwrap_or("help");

    match command {
        "a" | "add" => {
            let title = args.get(1).expect("Expected title of series to add");
            let first_air_year = args.get(2).and_then(|a| a.parse::<i32>().ok());
            add::add_series(&mut ctx, &mut app_state, title, first_air_year)?;
        }
        "add-id" => {
            let series_id = SeriesId(
                args.get(1)
                    .and_then(|a| a.parse::<i32>().ok())
                    .expect("Expected TMDB series ID to add"),
            );
            add::add_series_by_id(&mut ctx, &mut app_state, series_id)?;
        }
        "add-all" => {
            let file_path = args
                .get(1)
                .expect("Expected path to file containing series titles to add");
            add::add_all_series(&mut ctx, &mut app_state, file_path)?;
        }
        "u" | "update" => {
            let all_changes = match args.get(1).map(|s| s.as_ref()) {
                Some(series_id) if series_id.parse::<i32>().is_ok() => {
                    let series_id =
                        SeriesId(series_id.parse::<i32>().expect("Series ID to update"));

                    let series_state = match app_state.tracked_series.get_mut(&series_id) {
                        Some(s) => s,
                        None => {
                            eprintln!("Error: no tracked series with ID {series_id}");
                            return Ok(()); // TODO: should return an error
                        }
                    };

                    // TODO: not great arg handling
                    let force = args
                        .get(2)
                        .map(|a| a == "-f" || a == "--force")
                        .unwrap_or(false);

                    match update::update_one_series(&mut ctx, series_state, force)? {
                        None => Vec::new(),
                        Some(changes) => vec![changes],
                    }
                }
                Some("-f") | Some("--force") => {
                    update::update_all_series(&mut ctx, &mut app_state, true)?
                }
                Some(unknown_arg) => {
                    eprintln!("Error: unknown argument {unknown_arg}");
                    return Ok(()); // TODO: should return an error
                }
                None => update::update_all_series(&mut ctx, &mut app_state, false)?,
            };

            // TODO: allow notifications to be only printed, for testing/debugging
            if !all_changes.is_empty() {
                notify::send_email_notifications(&ctx, &app_state, all_changes)?;
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

    if ctx.app_state_changed {
        app_state.write_to(&ctx.config.state_file_path.0)?;
    }

    Ok(())
}
