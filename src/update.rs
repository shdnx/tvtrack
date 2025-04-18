use anyhow::Context;
use chrono::TimeZone;

use crate::{db, tmdb};

use super::{AppContext, EpisodeDetails, SeriesDetails, SeriesStatus};

#[derive(Debug)]
pub struct SeriesDetailsChanges {
    pub id: tmdb::SeriesId,
    pub in_production_change: Option<(bool, bool)>,
    pub status_change: Option<(SeriesStatus, SeriesStatus)>,
    pub last_episode_change: Option<(Option<EpisodeDetails>, Option<EpisodeDetails>)>,
    pub next_episode_change: Option<(Option<EpisodeDetails>, Option<EpisodeDetails>)>,
    pub episode_count_change: Option<(i32, i32)>,
    // TODO: we should also check for episodes that have aired, as next_episode_to_air may change in bulk (e.g. on Netflix where a whole season is released all at once)
}

impl SeriesDetailsChanges {
    pub fn new(id: tmdb::SeriesId) -> SeriesDetailsChanges {
        SeriesDetailsChanges {
            id,
            in_production_change: None,
            status_change: None,
            last_episode_change: None,
            next_episode_change: None,
            episode_count_change: None,
        }
    }

    pub fn has_any_changes(&self) -> bool {
        self.in_production_change.is_some()
            || self.status_change.is_some()
            || self.last_episode_change.is_some()
            || self.next_episode_change.is_some()
            || self.episode_count_change.is_some()
    }

    pub fn summary(&self) -> String {
        if !self.has_any_changes() {
            return "no changes".to_owned();
        }

        let mut summary = String::with_capacity(256);
        summary += "\n";
        if let Some((old_in_prod, new_in_prod)) = self.in_production_change {
            summary += &format!(" - In production: {old_in_prod} => {new_in_prod}\n");
        }
        if let Some((old_status, new_status)) = self.status_change {
            summary += &format!(" - Status: {old_status} => {new_status}\n");
        }
        if let Some((ref old_last_ep, ref new_last_ep)) = self.last_episode_change {
            summary += &format!(
                " - Last episode: {} => {}\n",
                old_last_ep
                    .as_ref()
                    .map(|e| e.identify())
                    .unwrap_or("unknown".into()),
                new_last_ep
                    .as_ref()
                    .map(|e| e.identify())
                    .unwrap_or("unknown".into()),
            );
        }
        if let Some((ref old_next_ep, ref new_next_ep)) = self.next_episode_change {
            summary += &format!(
                " - Next episode: {} => {}\n",
                old_next_ep
                    .as_ref()
                    .map(|e| e.identify())
                    .unwrap_or("unknown".into()),
                new_next_ep
                    .as_ref()
                    .map(|e| e.identify())
                    .unwrap_or("unknown".into())
            );
        }
        if let Some((old_ep_count, new_ep_count)) = self.episode_count_change {
            summary += &format!(" - Episode count: {old_ep_count} => {new_ep_count}\n");
        }

        summary
    }
}

fn collect_series_details_changes(
    old_details: &SeriesDetails,
    new_details: &SeriesDetails,
) -> SeriesDetailsChanges {
    assert_eq!(old_details.id, new_details.id);
    let mut changes = SeriesDetailsChanges::new(new_details.id);

    if old_details.in_production != new_details.in_production {
        changes.in_production_change = Some((old_details.in_production, new_details.in_production));
    }

    if old_details.status != new_details.status {
        changes.status_change = Some((old_details.status, new_details.status));
    }

    // we don't check for or report changes on other details about the last and next episode
    let old_last_ep_date = old_details.last_episode_date();
    let new_last_ep_date = new_details.last_episode_date();
    if old_last_ep_date != new_last_ep_date {
        changes.last_episode_change = Some((
            old_details.last_episode_to_air.clone(),
            new_details.last_episode_to_air.clone(),
        ));
    }

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
    ctx: &mut AppContext,
    old_details: &SeriesDetails,
) -> anyhow::Result<(
    SeriesDetailsChanges,
    SeriesDetails,
    chrono::DateTime<chrono::Utc>,
)> {
    let series_id = old_details.id;
    let new_details = ctx.tmdb.get_series_details(series_id)?;
    let changes = collect_series_details_changes(old_details, &new_details);
    let update_timestamp = chrono::Utc::now();

    ctx.db.conn.execute(
        "UPDATE series SET status = :status, in_production = :in_production, last_episode_air_date = :last_episode_air_date, next_episode_air_date = :next_episode_air_date, details = :details, update_timestamp = :update_timestamp WHERE tmdb_id = :id",
        rusqlite::named_params! {
            ":id": series_id,
            ":status": new_details.status,
            ":in_production": new_details.in_production,
            ":last_episode_air_date": new_details.last_episode_date(),
            ":next_episode_air_date": new_details.next_episode_date(),
            ":details": serde_json::to_value(&new_details).unwrap(),
            ":update_timestamp": update_timestamp,
        }
    ).with_context(|| format!("Updating series {} in the database", old_details.identify()))?;

    Ok((changes, new_details, update_timestamp))
}

pub fn update_one_series(
    ctx: &mut AppContext,
    series: &mut db::Series,
    force: bool,
) -> anyhow::Result<Option<SeriesDetailsChanges>> {
    let (next_update_timestamp, reason) = determine_next_update_timestamp(series);
    if !force && chrono::Utc::now() < next_update_timestamp {
        log::info!(
            "Not updating {} again until {next_update_timestamp} because {reason} (last update: {})",
            series.details.identify(),
            series.update_timestamp
        );
        return Ok(None);
    }

    let (changes, new_details, update_timestamp) =
        update_and_collect_changes(ctx, &series.details)?;
    if !changes.has_any_changes() {
        log::info!(
            "No changes to {} since last update at {}",
            series.details.identify(),
            series.update_timestamp
        );
        return Ok(None);
    }

    log::info!("Series {} changes: {}", series.details, changes.summary());

    series.set_details(new_details, update_timestamp);
    Ok(Some(changes))
}

pub fn determine_next_update_timestamp(
    series: &db::Series,
) -> (chrono::DateTime<chrono::Utc>, &'static str) {
    if let Some(next_ep_dt) = series.next_episode_air_date.0 {
        // if we know when the next episode is airing, then we don't need to update it again until after that date
        return (
            chrono::Utc.from_utc_datetime(&next_ep_dt.and_hms_opt(23, 59, 59).unwrap()),
            "next episode release",
        );
    }

    let now = chrono::Utc::now();
    let (interval, reason) = match series.status {
        SeriesStatus::Ended | SeriesStatus::Canceled => {
            // if the last episode aired at least 4 weeks ago, then we consider things unlikely to change, so we don't have to update the series as often
            let last_ep_is_old = series
                .last_episode_air_date
                .map(|dt| now.date_naive().signed_duration_since(dt).num_weeks() >= 4)
                .unwrap_or(true);

            if last_ep_is_old {
                (chrono::Duration::weeks(4), "series is over for long")
            } else {
                (chrono::Duration::weeks(1), "series is recently over")
            }
        }
        SeriesStatus::InProduction | SeriesStatus::ReturningSeries => {
            (chrono::Duration::weeks(1), "series in production")
        }
    };
    (series.update_timestamp + interval, reason)
}

pub fn update_all_series(
    ctx: &mut AppContext,
    force: bool,
) -> anyhow::Result<Vec<(db::Series, SeriesDetailsChanges)>> {
    let series = ctx.db.get_all_series()?;
    let mut changes = Vec::with_capacity(series.len());

    for mut series in series.into_iter() {
        match update_one_series(ctx, &mut series, force) {
            Ok(None) => {}
            Ok(Some(series_changes)) => {
                changes.push((series, series_changes));
            }
            Err(err) => {
                log::error!("Failed to update series {}: {err:?}", series.details);
            }
        }
    }

    Ok(changes)
}
