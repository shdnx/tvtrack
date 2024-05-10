mod tmdb;

mod state;
use state::{ApplicationState, SeriesState};

mod result;
use result::{AnyError, Result};

use serde::{Deserialize, Serialize};
use std::{fmt::Display, io::Read};
use tmdb::SeriesId;

fn print_help() {
    println!(
        r#"Available commands:
 - a | add <title> [<release year>]: register a series with the specified title to track.
 - add-id <id>: add a series by TMDB ID.
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
    println!(
        "Add series: {title}{}",
        first_air_year
            .map(|y| format!(" ({y})"))
            .unwrap_or_default()
    );
    let search_result = ctx.tmdb_client.search_series(title, first_air_year)?;

    if search_result.results.len() > 1 {
        for sr in search_result.results.iter() {
            println!(
                "-- Result: #{} {} ({}): {}",
                sr.id, sr.name, sr.first_air_date, sr.overview
            );
        }
    }

    // filter out any results without a first air date, I think those are not series but maybe episodes?
    let mut candidates: Vec<_> = search_result
        .results
        .iter()
        .filter(|sr| sr.first_air_date.is_some())
        .collect();

    // see if any candidates have an exact name match, if so, we want an exact match
    // TODO: this is not foolproof if a release year is not specified, as it may be that an ancient title happens to match the exact spelling while we actually wanted one that has an extra dot or color or whatever
    if candidates.iter().any(|sr| sr.name == title) {
        candidates.retain(|sr| sr.name == title);
    }

    // if there are multiple candidates, assume the most recent one
    let best_match = candidates
        .iter()
        .max_by_key(|sr| sr.first_air_date.unwrap());

    let best_match = match best_match {
        None => {
            println!("-- No results");
            return Ok(false);
        }
        Some(bm) => bm,
    };

    println!(
        "-- Selected: {} ({}): {}",
        best_match.name, best_match.first_air_date, best_match.overview
    );

    add_series_by_id(ctx, best_match.id)
}

fn add_series_by_id(ctx: &mut CmdContext, id: SeriesId) -> Result<bool> {
    println!("Adding series by id: {id}");

    if ctx.app_state.tracked_series.contains_key(&id) {
        println!("-- Ignoring: series is already tracked");
        return Ok(true);
    }

    let series_details = ctx.tmdb_client.get_series_details(id)?;

    println!(
        "-- In production: {} | status: {}",
        series_details.in_production, series_details.status
    );

    if !series_details.in_production || series_details.status != "Returning Series" {
        // TODO: notify
    }

    if let Some(ref next_ep) = series_details.next_episode_to_air {
        println!(
            "-- Next episode: S{:02}E{:02} ({}) {} expected on {}",
            next_ep.season_number,
            next_ep.episode_number,
            next_ep.episode_type,
            next_ep.name,
            next_ep.air_date
        );

        // TODO: notify
    } else {
        println!("-- Next episode: unknown");
    }

    ctx.app_state.tracked_series.insert(
        series_details.id,
        SeriesState {
            details: series_details,
            timestamp: chrono::Utc::now(),
        },
    );
    ctx.app_state_changed = true;

    Ok(true)
}

fn add_all_series(ctx: &mut CmdContext, file_path: &str) -> Result<()> {
    println!("Adding all series from file: {file_path}");

    // Allow the line to optionally end in the release (first air) year in parens, e.g. (2024).
    fn parse_line(line: &str) -> (&str, Option<i32>) {
        let (title, maybe_year) = match line.trim().rsplit_once(' ') {
            Some(r) => r,
            None => return (line.trim(), None),
        };

        if !maybe_year.starts_with('(') || !maybe_year.ends_with(')') {
            return (line.trim(), None);
        }

        match maybe_year[1..maybe_year.len() - 1].parse() {
            Ok(year) => (title.trim_end(), Some(year)),
            Err(_) => (line.trim(), None),
        }
    }

    for line in std::fs::read_to_string(file_path)?.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (title, first_air_year) = parse_line(line);

        add_series(ctx, title, first_air_year)?;
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
        }
    };
    let state_file_path = "tvtrack.state.json"; // TODO: take optionally from command line arg?

    let mut ctx = CmdContext {
        tmdb_client: tmdb::Client::new(tmdb_api_token),
        app_state: ApplicationState::read_from_or_new(state_file_path)?,
        app_state_changed: false,
    };

    match command {
        "a" | "add" => {
            let title = args.get(1).expect("Expected title of series to add");
            let first_air_year = args.get(2).and_then(|a| a.parse::<i32>().ok());
            add_series(&mut ctx, title, first_air_year)?;
        }
        "add-id" => {
            let series_id = SeriesId(
                args.get(1)
                    .and_then(|a| a.parse::<i32>().ok())
                    .expect("Expected TMDB series ID to add"),
            );
            add_series_by_id(&mut ctx, series_id)?;
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
