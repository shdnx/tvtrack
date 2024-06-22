extern crate tvtrack;

use anyhow::Context;
use rusqlite::named_params;
// TODO: inline state.rs here once the main code no longer uses it
use tvtrack::state::ApplicationState as JsonAppState;

fn main() -> anyhow::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let config_file = args
        .first()
        .cloned()
        .unwrap_or("tvtrack.config.json".to_owned());
    let input_file = args
        .get(1)
        .cloned()
        .unwrap_or("tvtrack.state.json".to_owned());
    let output_file = args
        .get(2)
        .cloned()
        .unwrap_or("tvtrack.state.sqlite".to_owned());

    let config =
        tvtrack::config::AppConfig::try_read(&config_file).context("Failed to read config file")?;
    let app_state =
        JsonAppState::read_from_or_new(&input_file).context("Failed to read state file")?;

    let mut tmdb_client = tvtrack::tmdb::Client::new(
        config.tmdb.api_key.clone(),
        config.tmdb.api_access_token.clone(),
    );

    let conn = rusqlite::Connection::open(output_file).context("Failed to open SQLite DB")?;

    conn.execute("BEGIN;", ())?;

    let mut insert_series_query = conn.prepare("insert into series (tmdb_id, title, first_air_date, poster_data, poster_mime_type, status, in_production, last_episode_air_date, next_episode_air_date, details, update_timestamp) values (:tmdb_id, :title, :first_air_date, :poster_data, :poster_mime_type, :status, :in_production, :last_episode_air_date, :next_episode_air_date, :details, :update_timestamp)")?;

    for (series_id, series) in app_state.tracked_series.iter() {
        let (poster_data, poster_type) =
            tvtrack::poster::fetch_poster_image(&mut tmdb_client, &series.details)
                .with_context(|| format!("Poster for series {}", series.details.identify()))?;

        insert_series_query.execute(named_params! {
            ":tmdb_id": series_id,
            ":title": series.details.name,
            ":first_air_date": series.details.first_air_date,
            ":poster_data": poster_data,
            ":poster_mime_type": poster_type,
            ":status": series.details.status,
            ":in_production": series.details.in_production,
            ":last_episode_air_date": series.details.last_episode_to_air.as_ref().and_then(|ep| ep.air_date.0),
            ":next_episode_air_date": series.details.next_episode_to_air.as_ref().and_then(|ep| ep.air_date.0),
            ":details": serde_json::to_value(&series.details).unwrap(),
            ":update_timestamp": series.timestamp,
        })?;
    }

    conn.execute("COMMIT;", ())?;
    Ok(())
}
