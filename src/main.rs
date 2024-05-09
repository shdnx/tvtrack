mod tmdb;

mod state;
use state::{ApplicationState, SeriesState};

mod result;
use result::{Result, AnyError};

use serde::{Deserialize, Serialize};
use std::{fmt::Display, io::Read};

fn print_help() {
    println!(
        r#"Available commands:
 - a | add <title>: register a series with the specified title to track. May ask for confirmation if there are multiple options.
 - add-all <file>: read the text file at the specified path, interpreting each line as a title to add as if individually done with the 'add' command.
 - check: magic!
 - h | help: show this help message.
"#
    );
}

struct CmdContext {
    tmdb_client: tmdb::Client,
    app_state: ApplicationState,
    app_state_changed: bool, // TODO: probably move into ApplicationState?
}

fn add_series(ctx: &mut CmdContext, title: &str, first_air_year: Option<i32>) -> Result<bool> {
    println!("Add series: {title}{}", first_air_year.map(|y| format!(" ({y})")).unwrap_or_default());
    let search_result = ctx.tmdb_client.search_series(title, first_air_year)?;

    // if there are multiple results, assume the most recent one
    let best_match = search_result
        .results
        .iter()
        .filter(|sr| sr.first_air_date.is_some())
        .max_by_key(|sr| sr.first_air_date.unwrap());

    let best_match = match best_match {
        None => {
            println!("-- No results found");
            return Ok(false);
        },
        Some(bm) => bm,
    };
    println!("-- Found: {} ({}): {}", best_match.name, best_match.first_air_date, best_match.overview);

    if ctx.app_state.tracked_series.contains_key(&best_match.id) {
        println!("-- Ignoring: series is already tracked (ID {})", best_match.id);
        return Ok(true);
    }

    // TODO: get series details
    // TODO: ctx.app_state.tracked_series.insert(best_match.id, best_match);
    //ctx.app_state_changed = true;

    Ok(true)
}

fn add_all_series(ctx: &mut CmdContext, file_path: &str) -> Result<()> {
    println!("Adding all series from file: {file_path}");

    for line in std::fs::read_to_string(file_path)?.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // TODO: allow the line to contain the release year
        add_series(ctx, line, None)?;
        println!();
    }

    Ok(())
}

fn perform_check(ctx: &mut CmdContext) -> Result<()> {
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("help");

    let tmdb_api_token = match std::env::var("TMDB_API_ACCESS_TOKEN") {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Error: TMDB_API_ACCESS_TOKEN is not set or invalid");
            return Err(err.into());
        },
    };
    let state_file_path = "tvtrack.state.json"; // TODO: take optionally from command line arg?

    let mut ctx = CmdContext{
        tmdb_client: tmdb::Client::new(tmdb_api_token),
        app_state: ApplicationState::read_from_or_new(state_file_path)?,
        app_state_changed: false,
    };

    match command {
        "a" | "add" => {
            let title = args.get(1).expect("Expected title of series to add");
            add_series(&mut ctx, title, None)?;
        }
        "add-all" => {
            let file_path = args
                .get(1)
                .expect("Expected path to file containing series titles to add");
            add_all_series(&mut ctx, file_path)?;
        }
        "check" => {
            perform_check(&mut ctx)?;
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
        ctx.app_state.write_to(state_file_path)?;
    }

    Ok(())
}
