# tvtrack

Data is queried from [The Movie Database (TMDB)](https://www.themoviedb.org/).
Sending e-mails happens by SMTP, so any provider will do. Currently using [MailTrap](https://mailtrap.io/).

## Build and run

To run, use `run.sh`.
The configuration file lives at `data/tvtrack.config.json` by default.
The database by default lives at `data/tvtrack.state.sqlite` by default.

## TODO

- Use proper logging
- Set up on NAS, auto-schedule execution of `update`
    - Need to set up some monitoring
- Write unit tests
- Perhaps save a backup of the database every time we make changes? Or save previous version of series in a separate table?
- Properly support multiple users. For example, when adding a series, also ask which users to register it for. Or add another mechanism for subscribing.

## Improvement ideas

- Also keep track of unreleased movies and notify when they come out
- We could also keep track of popular shows using https://developer.themoviedb.org/reference/movie-popular-list and send some recommendations based on that?
- Track releases on NCore as well?
- We could track changes via https://developer.themoviedb.org/reference/changes-tv-list but I think for now this would lead to having to many more requests than by just querying details directly
