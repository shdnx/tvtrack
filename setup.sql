create table series (
    tmdb_id int not null primary key,
    title text not null,
    first_air_date text, /* may be null if unreleased */

    poster_data blob, /* image data */
    poster_mime_type text,

    /* note: these fields are also all present in in the `details` JSON */
    status text,
    in_production int, /* 0 or 1 */
    last_episode_air_date text,
    next_episode_air_date text,

    details text, /* json */

    update_timestamp text
);

create table users (
    id int primary key,
    name text not null,
    email text not null
);

create table tracked_series (
    user_id int not null references users(id),
    series_tmdb_id int not null references series(tmdb_id),
    start_timestamp text
);
CREATE UNIQUE INDEX tracked_series_idx ON tracked_series(user_id, series_tmdb_id);

insert into users ( name, email )
    values ( "Gabor", "id@gaborkozar.me" );