use super::{ApplicationState, CmdContext, Result, SeriesId, SeriesState, EpisodeDetails};

pub fn add_series(
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

    for sr in search_result.results.iter() {
        println!(
            "-- Result: #{} {} ({}): {}",
            sr.id, sr.name, sr.first_air_date, sr.overview
        );
    }

    // NOTE: sometimes the search results seem to contain non-series results, perhaps episodes? those always have `first_air_date == None`, but we can't just use that as a filter because then we prevent announced yet unreleased series to be added
    let mut candidates: Vec<_> = search_result.results;

    // see if any candidates have an exact name match, if so, we want an exact match
    // TODO: this is not foolproof if a release year is not specified, as it may be that an ancient title happens to match the exact spelling while we actually wanted one that has an extra dot or color or whatever
    if candidates.iter().any(|sr| sr.name == title) {
        candidates.retain(|sr| sr.name == title);
    }

    // if there are multiple candidates, prefer the one that is already released and most recently so
    // this is already how `Option<chrono::NaiveDate>` and so `OptionalDate` are ordered, so we can just take the max
    let best_match = candidates.iter().max_by_key(|sr| sr.first_air_date);

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

pub fn add_series_by_id(
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

    println!(
        "-- Next episode: {}",
        series_details
            .next_episode_to_air
            .as_ref()
            .map(EpisodeDetails::identify)
            .unwrap_or("unknown".to_owned())
    );

    let next_update_timestamp = ctx.determine_next_update_timestamp(&series_details);
    app_state.tracked_series.insert(
        series_details.id,
        SeriesState {
            details: series_details,
            timestamp: ctx.now,
            next_update_timestamp,
        },
    );
    ctx.app_state_changed = true;

    Ok(true)
}

pub fn add_all_series(
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
