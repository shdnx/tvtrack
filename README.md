# tvtrack

Data is queried from [The Movie Database (TMDB)](https://www.themoviedb.org/).
Sending e-mails happens by SMTP, so any provider will do. Currently using [MailTrap](https://mailtrap.io/).

## Build and run

To run: `cargo run --config ./data/tvtrack.test.config.json <command>`.
Instead of passing `--config`, you can also set the `TVTRACK_CONFIG_FILE` environment variable.
`run.sh` provides a convenient shortcut for the above.

## E-mails

MailTrap's shared IP apparently has a really bad reputation, so e-mails from it are extremely likely to be marked as SPAM.
Bulk e-mails require an unsubscribe link, and the one that MailTrap inserts causes the e-mails to be detected as SPAM due to the unsubscribe link pointing to a domain that's in the [Invaluement database](https://www.invaluement.com/). So use the transactional e-mails SMTP endpoint instead.

## TODO

= Make TableModel derive-able, see eg https://github.com/dtolnay/syn/blob/master/examples/heapsize/heapsize_derive/src/lib.rs
- Implement a dry-run mode, where we don't update the database nor send e-mails; for testing
- Set up on NAS, auto-schedule execution of `update`
    - Need to set up some monitoring
- Apparently not having a plaintext version of the e-mail is suspicious for some SPAM detectors
- Perhaps save a backup of the database every time we make changes? Or save previous version of series in a separate table?
- Properly support multiple users. For example, when adding a series, also ask which users to register it for. Or add another mechanism for subscribing.

## Improvement ideas

- Also keep track of unreleased movies and notify when they come out
- Could also notify about release of new books, if GoodReads or TheStoryGraph has a public API
- We could also keep track of popular shows using https://developer.themoviedb.org/reference/movie-popular-list and send some recommendations based on that?
- Track releases on NCore as well?
- We could track changes via https://developer.themoviedb.org/reference/changes-tv-list but I think for now this would lead to having to many more requests than by just querying details directly
