use crate::{
    result::AnyError,
    state::{ApplicationState, SeriesState},
    tmdb, CmdContext, Result, SeriesDetailsChanges,
};
use chrono::Datelike;
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};

fn make_email_html(
    client: &tmdb::Client,
    all_changes: &[(&SeriesState, SeriesDetailsChanges)],
) -> String {
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

    for (series_state, series_changes) in all_changes.iter() {
        let template = r###"
                    <tr>
                        <td class="series-poster" style="font-family: sans-serif; font-size: 14px; vertical-align: top; width: 110px;" width="110" valign="top">
                        <img src="{{poster_url}}" alt="{{title}} poster" style="border: none; -ms-interpolation-mode: bicubic; max-width: 100%; width: 92px;" width="92">
                        </td>
                        <td style="font-family: sans-serif; font-size: 14px; vertical-align: top;" valign="top">
                        <h3 class="series-title" style="color: #06090f; font-family: sans-serif; font-weight: 400; line-height: 1.4; margin: 0; margin-bottom: 7px;">
                            <a href="{{url}}" style="color: #ec0867; text-decoration: underline;">{{title}} ({{release_year}})</a>
                        </h3>
                        <ul class="series-changes" style="font-family: sans-serif; font-size: 14px; font-weight: normal; margin: 0; padding: 0; margin-bottom: 30px;">
                            <li style="list-style-position: inside; margin-left: 5px;">{{in_production}}</li>
                            <li style="list-style-position: inside; margin-left: 5px;">{{status}}</li>
                            <li style="list-style-position: inside; margin-left: 5px;">Last: {{last_episode}}</li>
                            <li style="list-style-position: inside; margin-left: 5px;">Next: {{next_episode}</li>
                        </ul>
                        </td>
                    </tr>"###;

        let series_html = template
            .to_string()
            .replace("{{title}}", &series_state.details.name)
            .replace(
                "{{release_year}}",
                &series_state
                    .details
                    .first_air_date
                    .map(|dt| dt.year().to_string())
                    .unwrap_or("unreleased".to_owned()),
            )
            .replace("{{url}}", &client.make_series_url(series_state.details.id))
            .replace(
                "{{poster_url}}",
                &client.make_poster_url(&series_state.details.poster_path),
            )
            .replace(
                "{{in_production}}",
                match series_changes.in_production_change {
                    None => {
                        if series_state.details.in_production {
                            "In production"
                        } else {
                            "Not in production"
                        }
                    }
                    Some((_, false)) => "No longer in production",
                    Some((_, true)) => "Now in production",
                },
            )
            .replace(
                "{{status}}",
                &match series_changes.status_change {
                    None => series_state.details.status.to_string(),
                    Some((old_status, new_status)) => format!("{old_status} &#8658; {new_status}"),
                },
            )
            .replace(
                "{{last_episode}}",
                &series_state
                    .details
                    .last_episode_to_air
                    .as_ref()
                    .map(|last_ep| last_ep.identify())
                    .unwrap_or("none".to_owned()),
            )
            .replace(
                "{{next_episode}}",
                &series_state
                    .details
                    .next_episode_to_air
                    .as_ref()
                    .map(|next_ep| next_ep.identify())
                    .unwrap_or("none".to_owned()),
            );
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

fn try_get_env_mailbox(env_var: &str) -> Result<Mailbox> {
    std::env::var(env_var)
        .map_err(AnyError::from)
        .and_then(|s| s.parse().map_err(AnyError::from))
        .inspect_err(|e| {
            eprintln!("Invalid or missing env var {env_var}: {e:?}");
        })
}

fn try_get_env_var(env_var: &str) -> Result<String> {
    std::env::var(env_var)
        .inspect_err(|e| {
            eprintln!("Invalid or missing env var {env_var}: {e:?}");
        })
        .map_err(AnyError::from)
}

pub fn send_email_notifications(
    ctx: &CmdContext,
    app_state: &ApplicationState,
    changes: Vec<SeriesDetailsChanges>,
) -> Result<()> {
    let changes = changes
        .into_iter()
        .map(|c| (app_state.tracked_series.get(&c.id).unwrap(), c))
        .collect::<Vec<_>>();

    let email = Message::builder()
        .from(try_get_env_mailbox("EMAILS_FROM")?)
        .to(try_get_env_mailbox("EMAILS_TO")?)
        .subject(format!("TVTrack updates {}", ctx.now.date_naive()))
        .header(ContentType::TEXT_HTML)
        .body(make_email_html(&ctx.tmdb_client, &changes))?;

    let credentials = Credentials::new(
        try_get_env_var("SMTP_USER")?,
        try_get_env_var("SMTP_PASSWORD")?,
    );

    let mailer = SmtpTransport::relay(&try_get_env_var("SMTP_HOST")?)
        .inspect_err(|e| {
            eprintln!("Failed to set up TLS for SMTP: {e:?}");
        })?
        .port(
            try_get_env_var("SMTP_PORT")
                .and_then(|port| port.parse().map_err(AnyError::from))
                .inspect_err(|e| {
                    eprintln!("Missing or invalid env var SMTP_PORT: {e:?}");
                })?,
        )
        .credentials(credentials)
        .build();

    mailer.send(&email).inspect_err(|e| {
        eprintln!("Failed to send e-mail via SMTP: {e:?}");
    })?;
    Ok(())
}
