create table series (
    tmdb_id int not null primary key,
    title text not null,
    first_air_date text, /* may be null if unreleased */

    poster_id int references posters(id),

    /*
        note: these fields are also all present in in the `details` JSON;
        we make them dedicated fields to make it easier to work with them
        but they always have to be consistent with the data in `details`
    */
    status text,
    in_production int, /* 0 or 1 */
    last_episode_air_date text,
    next_episode_air_date text,

    details text, /* json */

    update_timestamp text
);

create table posters (
    id int not null primary key,
    img_data blob not null,
    mime_type text not null,
    source_url text
);

create table users (
    id int not null primary key,
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