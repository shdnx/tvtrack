mod email_html;
mod entry;

use self::email_html::make_email_html;
use self::entry::{series_changes_to_entries, SeriesEntry};
use crate::{db, AppContext, SeriesDetailsChanges};
use anyhow::Context;
use lettre::{
    message::{header::ContentType, Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use std::collections::HashMap;

pub fn send_email_notifications(
    ctx: &mut AppContext,
    changes: &[(db::Series, SeriesDetailsChanges)],
) -> anyhow::Result<()> {
    // NOTE: we are using CIDs to attach the poster image data inline with the e-mail
    // this is because we don't have a simple GET url for them without leaking our TMDB API key
    // however, some e-mail clients don't like CIDs and prefer external images
    // that is only feasible if we have hosting and a CDN set up though
    // reading on CIDs:
    // - https://mailtrap.io/blog/embedding-images-in-html-email-have-the-rules-changed/
    // - https://stackoverflow.com/a/40420648/128240
    // - https://users.rust-lang.org/t/add-attachment-to-message-builder-in-lettre-email-sender/68471

    let entries = series_changes_to_entries(ctx, changes)?;

    let mut users_to_entries = HashMap::<i64, (db::User, Vec<&SeriesEntry<'_>>)>::new();
    for entry in entries.iter() {
        let subscribed_users = ctx
            .db
            .get_all_users_subscribed_to_series(entry.series.details.id)?;

        if subscribed_users.is_empty() {
            eprintln!(
                "WARNING: no users subscribed to series {} (ID {})",
                entry.series.details.identify(),
                entry.series.tmdb_id
            );
        }

        for user in subscribed_users {
            users_to_entries
                .entry(user.id)
                .and_modify(|(_, user_entries)| user_entries.push(entry))
                .or_insert_with(|| (user, vec![entry]));
        }
    }

    // eprintln!("Users to notify about series ({}):", users_to_entries.len());
    // for (user, entries) in users_to_entries.values() {
    //     eprintln!(" - User: {user:?}");
    //     eprintln!(
    //         " - Entries: {:?}",
    //         entries
    //             .iter()
    //             .map(|ent| ent.series.details.identify())
    //             .collect::<Vec<_>>()
    //     );
    // }

    let credentials = Credentials::new(
        ctx.config.smtp.user.clone(),
        ctx.config.smtp.password.clone(),
    );

    let mailer = SmtpTransport::starttls_relay(&ctx.config.smtp.host)
        .context("Setting up STARTTLS for SMTP")?
        .port(ctx.config.smtp.port)
        .credentials(credentials)
        .build();

    let from_mailbox = Mailbox::new(
        ctx.config.emails.from_name.clone(),
        ctx.config.emails.from_address.parse()?,
    );
    let now = chrono::Local::now();

    for (user, series_entries) in users_to_entries.values() {
        let email_multipart_contents = MultiPart::mixed().multipart({
            let mut multipart = MultiPart::related().singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_HTML)
                    .body(make_email_html(&entries)),
            );

            for entry in series_entries.iter() {
                multipart = multipart.singlepart(entry.create_poster_attachment());
            }

            multipart
        });

        let email = Message::builder()
            .from(from_mailbox.clone())
            .to(Mailbox::new(Some(user.name.clone()), user.email.parse()?))
            .subject(format!("TVTrack updates {}", now.date_naive()))
            .multipart(email_multipart_contents)
            .with_context(|| format!("building email for {user:?}"))?;

        mailer
            .send(&email)
            .context("Sending e-mail notifications via SMTP")?;
    }

    Ok(())
}
