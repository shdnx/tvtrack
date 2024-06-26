use anyhow::Context;
use rusqlite::OptionalExtension;

use crate::{db, tmdb::OptionalDate};

use super::{AppContext, EpisodeDetails, SeriesId};

pub fn add_series(
    ctx: &mut AppContext,
    title: &str,
    first_air_year: Option<i32>,
) -> anyhow::Result<bool> {
    println!(
        "Add series: {title}{}",
        first_air_year
            .map(|y| format!(" ({y})"))
            .unwrap_or_default()
    );
    let search_result = ctx.tmdb.search_series(title, first_air_year)?;

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

    add_series_by_id(ctx, best_match.id)
}

pub fn add_series_by_id(ctx: &mut AppContext, id: SeriesId) -> anyhow::Result<bool> {
    println!("Adding series by id: {id}");

    match ctx
        .db
        .conn
        .query_row(
            "SELECT title, first_air_date FROM series WHERE tmdb_id = ? LIMIT 1",
            (id,),
            |row| <(String, OptionalDate)>::try_from(row),
        )
        .optional()
        .with_context(|| format!("Looking for series with ID {id}"))?
    {
        Some((existing_title, existing_release_date)) => {
            println!(
                "-- Ignoring: series is already tracked: {existing_title} ({existing_release_date})"
            );
            return Ok(true);
        }
        None => (),
    };

    let series_details = ctx.tmdb.get_series_details(id)?;
    let series_poster = ctx.tmdb.get_poster(&series_details.poster_path)?;

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

    ctx.db.conn.execute(
        "INSERT INTO posters (img_data, mime_type, source_url) VALUES (:img_data, :mime_type, :source_url)",
        rusqlite::named_params! {
            ":img_data": series_poster.img_data,
            ":mime_type": series_poster.mime_type,
            ":source_url": series_poster.source_url,
        }
    ).with_context(|| format!("Inserting series {} poster from {}", series_details.identify(), series_poster.source_url))?;
    let new_poster_id = db::PosterId(ctx.db.conn.last_insert_rowid());

    ctx.db.conn.execute(
        "INSERT INTO series (tmdb_id, title, first_air_date, poster_id, status, in_production, last_episode_air_date, next_episode_air_date, details, update_timestamp) VALUES (:tmdb_id, :title, :first_air_date, :poster_data, :poster_mime_type, :status, :in_production, :last_episode_air_date, :next_episode_air_date, :details, NOW())",
        rusqlite::named_params! {
            ":tmdb_id": id,
            ":title": series_details.name,
            ":first_air_date": series_details.first_air_date,
            ":poster_id": new_poster_id,
            ":status": series_details.status,
            ":in_production": series_details.in_production,
            ":last_episode_air_date": series_details.last_episode_to_air.as_ref().and_then(|ep| ep.air_date.0),
            ":next_episode_air_date": series_details.next_episode_to_air.as_ref().and_then(|ep| ep.air_date.0),
            ":details": serde_json::to_value(&series_details).unwrap(),
        }
    ).with_context(|| format!("Inserting series {} into the database", series_details.identify()))?;

    Ok(true)
}

pub fn add_all_series(ctx: &mut AppContext, file_path: &str) -> anyhow::Result<()> {
    println!("Adding all series from file: {file_path}");

    // Allow the line to optionally end in the release (first air) year in parens, e.g. (2024).
    fn parse_line(line: &str) -> (&str, Option<i32>) {
        let Some((title, maybe_year)) = line.trim().rsplit_once(' ') else {
            return (line.trim(), None);
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
