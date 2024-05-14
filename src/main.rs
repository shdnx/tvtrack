mod result;
mod state;
mod tmdb;

use result::Result;
use state::{ApplicationState, SeriesState};
use tmdb::{EpisodeDetails, SeriesDetails, SeriesId, SeriesStatus};

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

struct CmdContext {
    tmdb_client: tmdb::Client,
    now: chrono::DateTime<chrono::Utc>,
    app_state_changed: bool,
}

fn add_series(
    ctx: &mut CmdContext,
    app_state: &mut ApplicationState,
    title: &str,
    first_air_year: Option<i32>,
) -> Result<bool> {
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

    add_series_by_id(ctx, app_state, best_match.id)
}

fn add_series_by_id(
    ctx: &mut CmdContext,
    app_state: &mut ApplicationState,
    id: SeriesId,
) -> Result<bool> {
    println!("Adding series by id: {id}");

    if app_state.tracked_series.contains_key(&id) {
        println!("-- Ignoring: series is already tracked");
        return Ok(true);
    }

    let series_details = ctx.tmdb_client.get_series_details(id)?;

    println!(
        "-- In production: {} | status: {}",
        series_details.in_production, series_details.status
    );

    if !series_details.in_production || series_details.status != SeriesStatus::ReturningSeries {
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

    app_state.tracked_series.insert(
        series_details.id,
        SeriesState {
            details: series_details,
            timestamp: chrono::Utc::now(),
        },
    );
    ctx.app_state_changed = true;

    Ok(true)
}

fn add_all_series(
    ctx: &mut CmdContext,
    app_state: &mut ApplicationState,
    file_path: &str,
) -> Result<()> {
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

        add_series(ctx, app_state, title, first_air_year)?;
        println!();
    }

    Ok(())
}

struct SeriesDetailsChanges {
    in_production_change: Option<(bool, bool)>,
    status_change: Option<(SeriesStatus, SeriesStatus)>,
    next_episode_change: Option<(Option<EpisodeDetails>, Option<EpisodeDetails>)>,
    episode_count_change: Option<(i32, i32)>,
    // TODO: we should also check for episodes that have aired, as next_episode_to_air may change in bulk (e.g. on Netflix where a whole season is released all at once)
}

impl SeriesDetailsChanges {
    fn has_any_changes(&self) -> bool {
        self.in_production_change.is_some()
            || self.status_change.is_some()
            || self.next_episode_change.is_some()
            || self.episode_count_change.is_some()
    }
}

fn collect_series_details_changes(
    old_details: &SeriesDetails,
    new_details: &SeriesDetails,
) -> SeriesDetailsChanges {
    assert_eq!(old_details.id, new_details.id);

    let mut changes = SeriesDetailsChanges {
        in_production_change: None,
        status_change: None,
        next_episode_change: None,
        episode_count_change: None,
    };

    if old_details.in_production != new_details.in_production {
        changes.in_production_change = Some((old_details.in_production, new_details.in_production));
    }

    if old_details.status != new_details.status {
        changes.status_change = Some((old_details.status, new_details.status));
    }

    // we don't check for or report changes on other details about the next episode
    let old_next_ep_date = old_details.next_episode_date();
    let new_next_ep_date = new_details.next_episode_date();
    if old_next_ep_date != new_next_ep_date {
        changes.next_episode_change = Some((
            old_details.next_episode_to_air.clone(),
            new_details.next_episode_to_air.clone(),
        ));
    }

    if old_details.number_of_episodes != new_details.number_of_episodes {
        changes.episode_count_change = Some((
            old_details.number_of_episodes,
            new_details.number_of_episodes,
        ));
    }

    changes
}

fn update_and_collect_changes(
    ctx: &mut CmdContext,
    series_state: &mut SeriesState,
) -> Result<(SeriesDetailsChanges, chrono::DateTime<chrono::Utc>)> {
    let new_details = ctx
        .tmdb_client
        .get_series_details(series_state.details.id)?;
    let changes = collect_series_details_changes(&series_state.details, &new_details);

    series_state.details = new_details;

    let old_timestamp = series_state.timestamp;
    series_state.timestamp = ctx.now;

    Ok((changes, old_timestamp))
}

fn update_one_series(
    ctx: &mut CmdContext,
    series_state: &mut SeriesState,
    force: bool,
) -> Result<bool> {
    fn get_update_frequency(series: &SeriesDetails) -> chrono::TimeDelta {
        match series.status {
            SeriesStatus::ReturningSeries => chrono::TimeDelta::days(3),
            SeriesStatus::Canceled | SeriesStatus::Ended => chrono::TimeDelta::weeks(1),
        }
    }

    let update_freq = get_update_frequency(&series_state.details);
    if !force && ctx.now - series_state.timestamp < update_freq {
        println!(
            "Not updating {} because not enough time passed since last update at {}",
            series_state.details.identify(),
            series_state.timestamp
        );
        return Ok(false);
    }

    let (changes, since_timestamp) = update_and_collect_changes(ctx, series_state)?;
    ctx.app_state_changed = true; // since we updated the series details

    if !changes.has_any_changes() {
        println!(
            "No changes to {} since last update at {since_timestamp}",
            series_state.details.identify()
        );
        return Ok(false);
    }

    // TODO: by e-mail
    println!("Series {} changes:", series_state.details.identify());
    if let Some((old_in_prod, new_in_prod)) = changes.in_production_change {
        println!(" - In production: {old_in_prod} => {new_in_prod}");
    }
    if let Some((old_status, new_status)) = changes.status_change {
        println!(" - Status: {old_status} => {new_status}");
    }
    if let Some((old_next_ep, new_next_ep)) = changes.next_episode_change {
        println!(
            " - Next episode: {} => {}",
            old_next_ep
                .map(|e| e.identify())
                .unwrap_or("unknown".into()),
            new_next_ep
                .map(|e| e.identify())
                .unwrap_or("unknown".into())
        );
    }
    if let Some((old_ep_count, new_ep_count)) = changes.episode_count_change {
        println!(" - Episode count: {old_ep_count} => {new_ep_count}");
    }

    Ok(true)
}

fn update_all_series(
    ctx: &mut CmdContext,
    app_state: &mut ApplicationState,
    force: bool,
) -> Result<()> {
    for (_series_id, series_state) in app_state.tracked_series.iter_mut() {
        match update_one_series(ctx, series_state, force) {
            Ok(_) => {}
            Err(err) => {
                eprintln!(
                    "Error while updating series {}: {err:?}",
                    series_state.details.identify()
                );
            }
        }
    }

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

    let mut app_state = ApplicationState::read_from_or_new(state_file_path)?;
    let mut ctx = CmdContext {
        tmdb_client: tmdb::Client::new(tmdb_api_token),
        now: chrono::Utc::now(), // optimization: take the time only once
        app_state_changed: false,
    };

    match command {
        "a" | "add" => {
            let title = args.get(1).expect("Expected title of series to add");
            let first_air_year = args.get(2).and_then(|a| a.parse::<i32>().ok());
            add_series(&mut ctx, &mut app_state, title, first_air_year)?;
        }
        "add-id" => {
            let series_id = SeriesId(
                args.get(1)
                    .and_then(|a| a.parse::<i32>().ok())
                    .expect("Expected TMDB series ID to add"),
            );
            add_series_by_id(&mut ctx, &mut app_state, series_id)?;
        }
        "add-all" => {
            let file_path = args
                .get(1)
                .expect("Expected path to file containing series titles to add");
            add_all_series(&mut ctx, &mut app_state, file_path)?;
        }
        "u" | "update" => {
            match args.get(1).map(|s| s.as_ref()) {
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

                    update_one_series(&mut ctx, series_state, force)?;
                }
                Some("-f") | Some("--force") => {
                    update_all_series(&mut ctx, &mut app_state, true)?;
                }
                Some(unknown_arg) => {
                    eprintln!("Error: unknown argument {unknown_arg}");
                }
                None => {
                    update_all_series(&mut ctx, &mut app_state, false)?;
                }
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
        app_state.write_to(state_file_path)?;
    }

    Ok(())
}
