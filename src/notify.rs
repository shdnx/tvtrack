use std::io::Read;

use crate::{
    state::{ApplicationState, SeriesState},
    tmdb::{self, SeriesDetails},
    CmdContext, SeriesDetailsChanges,
};
use anyhow::{bail, Context};
use chrono::Datelike;
use lettre::{
    message::{header::ContentType, Attachment, Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};

fn fetch_poster_image(
    ctx: &mut CmdContext,
    series: &SeriesDetails,
) -> anyhow::Result<(Box<[u8]>, ContentType)> {
    // TODO: once we switch to SQLite, store the images inside
    let file_ext = series
        .poster_extension()
        .expect("Poster path without valid extension?");

    let cache_dir_path = "posters-cache";
    let cache_file_path = format!("{cache_dir_path}/{}.{file_ext}", series.id);

    match std::fs::create_dir(cache_dir_path) {
        Ok(()) => (),
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => (),
        // TODO: is this a good way?
        Err(err) => bail!("Posters cache directory could not be created: {err}"),
    };

    let (data, mime_type) = match std::fs::File::open(&cache_file_path) {
        Ok(mut file) => {
            let mut data: Vec<u8> = vec![];
            file.read_to_end(&mut data)
                .with_context(|| format!("Reading poster cache file {cache_file_path}"))?;

            let mime_type = tmdb::Client::try_determine_mime_type(&cache_file_path)?;
            (data.into_boxed_slice(), mime_type)
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            let (data, mime_type) = ctx.tmdb_client.get_poster(&series.poster_path)?;
            std::fs::write(&cache_file_path, &data)
                .with_context(|| format!("Writing poster cache file {cache_file_path}"))?;
            (data, mime_type)
        }
        Err(err) => return Err(err.into()),
    };

    let content_type = ContentType::parse(mime_type).expect("Invalid MIME type");
    Ok((data, content_type))
}

struct SeriesEntry<'a> {
    pub state: &'a SeriesState,
    pub changes: SeriesDetailsChanges,
    pub url: String,
    pub poster_url: String,
}

fn make_email_html(entries: &[SeriesEntry]) -> String {
    // TODO: use some kind of templating?
    let mut html: String = r###"<!doctype html>
<html>
    <head>
    <meta name="viewport" content="width=device-width">
    <meta http-equiv="Content-Type" content="text/html; charset=UTF-8">
    <title>TVTrack updates</title>
    <style>
@media only screen and (max-width: 620px) {
    table[class=body] h1 {
    font-size: 28px !important;
    margin-bottom: 10px !important;
    }

    table[class=body] p,
table[class=body] ul,
table[class=body] ol,
table[class=body] td,
table[class=body] span,
table[class=body] a {
    font-size: 16px !important;
    }

    table[class=body] .wrapper,
table[class=body] .article {
    padding: 10px !important;
    }

    table[class=body] .content {
    padding: 0 !important;
    }

    table[class=body] .container {
    padding: 0 !important;
    width: 100% !important;
    }

    table[class=body] .main {
    border-left-width: 0 !important;
    border-radius: 0 !important;
    border-right-width: 0 !important;
    }

    table[class=body] .btn table {
    width: 100% !important;
    }

    table[class=body] .btn a {
    width: 100% !important;
    }

    table[class=body] .img-responsive {
    height: auto !important;
    max-width: 100% !important;
    width: auto !important;
    }
}
@media all {
    .ExternalClass {
    width: 100%;
    }

    .ExternalClass,
.ExternalClass p,
.ExternalClass span,
.ExternalClass font,
.ExternalClass td,
.ExternalClass div {
    line-height: 100%;
    }

    .apple-link a {
    color: inherit !important;
    font-family: inherit !important;
    font-size: inherit !important;
    font-weight: inherit !important;
    line-height: inherit !important;
    text-decoration: none !important;
    }

    .btn-primary table td:hover {
    background-color: #d5075d !important;
    }

    .btn-primary a:hover {
    background-color: #d5075d !important;
    border-color: #d5075d !important;
    }
}
</style></head>
<body class style="background-color: #eaebed; font-family: sans-serif; -webkit-font-smoothing: antialiased; font-size: 14px; line-height: 1.4; margin: 0; padding: 0; -ms-text-size-adjust: 100%; -webkit-text-size-adjust: 100%;">
<table role="presentation" border="0" cellpadding="0" cellspacing="0" class="body" style="border-collapse: separate; mso-table-lspace: 0pt; mso-table-rspace: 0pt; min-width: 100%; background-color: #eaebed; width: 100%;" width="100%" bgcolor="#eaebed">
    <tr>
    <td style="font-family: sans-serif; font-size: 14px; vertical-align: top;" valign="top">&nbsp;</td>
    <td class="container" style="font-family: sans-serif; font-size: 14px; vertical-align: top; display: block; max-width: 580px; padding: 10px; width: 580px; Margin: 0 auto;" width="580" valign="top">
        <div class="header" style="padding: 20px 0;">
        <table role="presentation" border="0" cellpadding="0" cellspacing="0" style="border-collapse: separate; mso-table-lspace: 0pt; mso-table-rspace: 0pt; min-width: 100%; width: 100%;" width="100%">
            <tr>
            <td class="align-center" style="font-family: sans-serif; font-size: 14px; vertical-align: top; text-align: center;" valign="top" align="center">
                <h2 style="color: #06090f; font-family: sans-serif; font-weight: 400; line-height: 1.4; margin-bottom: 30px; margin: 0; padding: 0;">TVTrack updates</h2>
            </td>
            </tr>
        </table>
        </div>
        <div class="content" style="box-sizing: border-box; display: block; Margin: 0 auto; max-width: 580px; padding: 10px;">

        <!-- START CENTERED WHITE CONTAINER -->
        <span class="preheader" style="color: transparent; display: none; height: 0; max-height: 0; max-width: 0; opacity: 0; overflow: hidden; mso-hide: all; visibility: hidden; width: 0;">Updates to series you are tracking</span>

        <table role="presentation" class="main" style="border-collapse: separate; mso-table-lspace: 0pt; mso-table-rspace: 0pt; min-width: 100%; background: #ffffff; border-radius: 3px; width: 100%;" width="100%">

            <!-- START MAIN CONTENT AREA -->
            <tr>
            <td class="wrapper" style="font-family: sans-serif; font-size: 14px; vertical-align: top; box-sizing: border-box; padding: 20px;" valign="top">
                <table role="presentation" border="0" cellpadding="0" cellspacing="0" style="border-collapse: separate; mso-table-lspace: 0pt; mso-table-rspace: 0pt; min-width: 100%; width: 100%;" width="100%">
"###.into();

    for i in 0..entries.len() {
        let SeriesEntry {
            state: series_state,
            changes: series_changes,
            url: series_url,
            poster_url,
        } = &entries[i];
        let is_last = i == entries.len() - 1;

        let template = r###"
                    <tr>
                        <td class="series-poster" style="font-family: sans-serif; font-size: 14px; vertical-align: top; width: 110px;" width="110" valign="top">
                        <img src="{{poster_url}}" alt="{{title}} poster" style="border: none; -ms-interpolation-mode: bicubic; max-width: 100%; width: 92px;" width="92">
                        </td>
                        <td style="font-family: sans-serif; font-size: 14px; vertical-align: top;" valign="top">
                        <h3 class="series-title" style="color: #06090f; font-family: sans-serif; font-weight: 400; line-height: 1.4; margin: 0; margin-bottom: 7px;">
                            <a href="{{url}}" style="color: #ec0867; text-decoration: underline;">{{title}} ({{release_year}})</a>
                        </h3>
                        <ul class="series-changes" style="font-family: sans-serif; font-size: 14px; font-weight: normal; margin: 0; padding: 0; margin-bottom: {{margin_bottom}}px;">
                            <li style="list-style-position: inside; margin-left: 5px;">{{in_production}}</li>
                            <li style="list-style-position: inside; margin-left: 5px;">{{status}}</li>
                            <li style="list-style-position: inside; margin-left: 5px;">Last: {{last_episode}}</li>
                            <li style="list-style-position: inside; margin-left: 5px;">Next: {{next_episode}}</li>
                        </ul>
                        </td>
                    </tr>"###;

        fn wrap_changed(text: &str) -> String {
            format!(r#"<span style="color: #b6004c;">{}</span>"#, text)
        }

        let series_html = template
            .to_string()
            .replace("{{margin_bottom}}", if is_last { "0" } else { "30" })
            .replace("{{title}}", &series_state.details.name)
            .replace(
                "{{release_year}}",
                &series_state
                    .details
                    .first_air_date
                    .map(|dt| dt.year().to_string())
                    .unwrap_or("unreleased".to_owned()),
            )
            .replace("{{url}}", &series_url)
            .replace("{{poster_url}}", &poster_url)
            .replace(
                "{{in_production}}",
                &match series_changes.in_production_change {
                    None => {
                        if series_state.details.in_production {
                            "In production".to_owned()
                        } else {
                            "Not in production".to_owned()
                        }
                    }
                    Some((_, false)) => wrap_changed("No longer in production"),
                    Some((_, true)) => wrap_changed("Now in production"),
                },
            )
            .replace(
                "{{status}}",
                &match series_changes.status_change {
                    None => series_state.details.status.to_string(),
                    Some((old_status, new_status)) => {
                        wrap_changed(&format!("{old_status} &#8658; {new_status}"))
                    }
                },
            )
            .replace(
                "{{last_episode}}",
                &series_state
                    .details
                    .last_episode_to_air
                    .as_ref()
                    .map(|ep| ep.identify())
                    .unwrap_or("none".to_owned()),
            )
            .replace("{{next_episode}}", &{
                let ep_info = series_state
                    .details
                    .next_episode_to_air
                    .as_ref()
                    .map(|ep| ep.identify())
                    .unwrap_or("unknown".to_owned());

                if series_changes.next_episode_change.is_some() {
                    wrap_changed(&ep_info)
                } else {
                    ep_info
                }
            });
        html += &series_html;
    }

    html += r###"
                    </table>
                </td>
                </tr>

            <!-- END MAIN CONTENT AREA -->
            </table>

            <!-- END CENTERED WHITE CONTAINER -->
            </div>
        </td>
        <td style="font-family: sans-serif; font-size: 14px; vertical-align: top;" valign="top">&nbsp;</td>
        </tr>
    </table>
    </body>
</html>"###;

    html
}

pub fn send_email_notifications(
    ctx: &mut CmdContext,
    app_state: &ApplicationState,
    changes: Vec<SeriesDetailsChanges>,
) -> anyhow::Result<()> {
    // NOTE: we are using CIDs to attach the poster image data inline with the e-mail
    // this is because we don't have a simple GET url for them without leaking our TMDB API key
    // however, some e-mail clients don't like CIDs and prefer external images
    // that is only feasible if we have hosting and a CDN set up though
    // reading on CIDs:
    // - https://mailtrap.io/blog/embedding-images-in-html-email-have-the-rules-changed/
    // - https://stackoverflow.com/a/40420648/128240
    // - https://users.rust-lang.org/t/add-attachment-to-message-builder-in-lettre-email-sender/68471

    let entries = changes
        .into_iter()
        .map(|changes| {
            let id = changes.id;
            let state = app_state.tracked_series.get(&id).unwrap();
            SeriesEntry {
                state,
                changes,
                url: ctx.tmdb_client.make_series_url(id),
                poster_url: format!("cid:{id}.{}", state.details.poster_extension().unwrap()),
            }
        })
        .collect::<Vec<_>>();

    let email = Message::builder()
        .from(Mailbox::new(
            ctx.config.emails.from_name.clone(),
            ctx.config.emails.from_address.parse()?,
        ))
        .to(Mailbox::new(
            ctx.config.emails.to_name.clone(),
            ctx.config.emails.to_address.parse()?,
        ))
        .subject(format!("TVTrack updates {}", ctx.now.date_naive()))
        .multipart(MultiPart::mixed().multipart({
            let mut multipart = MultiPart::related().singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_HTML)
                    .body(make_email_html(&entries)),
            );

            for SeriesEntry { state, .. } in entries.iter() {
                let (poster_data, poster_content_type) = fetch_poster_image(ctx, &state.details)?;
                let cid_id = format!(
                    "{}.{}",
                    state.details.id,
                    state.details.poster_extension().unwrap()
                ); // TODO: duplication between this and entry.poster_url

                let attachment = Attachment::new_inline(cid_id)
                    .body(Vec::from(poster_data), poster_content_type);
                multipart = multipart.singlepart(attachment);
            }

            multipart
        }))?;

    let credentials = Credentials::new(
        ctx.config.smtp.user.clone(),
        ctx.config.smtp.password.clone(),
    );

    let mailer = SmtpTransport::starttls_relay(&ctx.config.smtp.host)
        .context("Setting up STARTTLS for SMTP")?
        .port(ctx.config.smtp.port)
        .credentials(credentials)
        .build();

    mailer
        .send(&email)
        .context("Sending e-mail notifications via SMTP")?;
    Ok(())
}
