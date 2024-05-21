use crate::tmdb::SeriesId;

use super::{
    ApplicationState, CmdContext, EpisodeDetails, Result, SeriesDetails, SeriesState, SeriesStatus,
};

#[derive(Debug)]
pub struct SeriesDetailsChanges {
    pub id: SeriesId,
    pub in_production_change: Option<(bool, bool)>,
    pub status_change: Option<(SeriesStatus, SeriesStatus)>,
    pub next_episode_change: Option<(Option<EpisodeDetails>, Option<EpisodeDetails>)>,
    pub episode_count_change: Option<(i32, i32)>,
    // TODO: we should also check for episodes that have aired, as next_episode_to_air may change in bulk (e.g. on Netflix where a whole season is released all at once)
}

impl SeriesDetailsChanges {
    pub fn new(id: SeriesId) -> SeriesDetailsChanges {
        SeriesDetailsChanges {
            id,
            in_production_change: None,
            status_change: None,
            next_episode_change: None,
            episode_count_change: None,
        }
    }

    pub fn has_any_changes(&self) -> bool {
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
    let mut changes = SeriesDetailsChanges::new(new_details.id);

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

pub fn update_one_series(
    ctx: &mut CmdContext,
    series_state: &mut SeriesState,
    force: bool,
) -> Result<Option<SeriesDetailsChanges>> {
    fn get_update_frequency(series: &SeriesDetails) -> chrono::TimeDelta {
        match series.status {
            SeriesStatus::InProduction | SeriesStatus::ReturningSeries => {
                chrono::TimeDelta::days(3)
            }
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
        return Ok(None);
    }

    let (changes, since_timestamp) = update_and_collect_changes(ctx, series_state)?;
    ctx.app_state_changed = true; // since we updated the series details

    if !changes.has_any_changes() {
        println!(
            "No changes to {} since last update at {since_timestamp}",
            series_state.details.identify()
        );
        return Ok(None);
    }

    // TODO: by e-mail
    println!("Series {} changes:", series_state.details.identify());
    if let Some((old_in_prod, new_in_prod)) = changes.in_production_change {
        println!(" - In production: {old_in_prod} => {new_in_prod}");
    }
    if let Some((old_status, new_status)) = changes.status_change {
        println!(" - Status: {old_status} => {new_status}");
    }
    if let Some((ref old_next_ep, ref new_next_ep)) = changes.next_episode_change {
        println!(
            " - Next episode: {} => {}",
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
    if let Some((old_ep_count, new_ep_count)) = changes.episode_count_change {
        println!(" - Episode count: {old_ep_count} => {new_ep_count}");
    }

    Ok(Some(changes))
}

pub fn update_all_series(
    ctx: &mut CmdContext,
    app_state: &mut ApplicationState,
    force: bool,
) -> Result<Vec<SeriesDetailsChanges>> {
    let mut changes = Vec::with_capacity(app_state.tracked_series.len());

    for (_series_id, series_state) in app_state.tracked_series.iter_mut() {
        match update_one_series(ctx, series_state, force) {
            Ok(None) => {}
            Ok(Some(series_changes)) => {
                changes.push(series_changes);
            }
            Err(err) => {
                eprintln!(
                    "Error while updating series {}: {err:?}",
                    series_state.details.identify()
                );
            }
        }
    }

    Ok(changes)
}
