use crate::db;
use crate::{AppContext, SeriesDetailsChanges};
use anyhow::Context;
use lettre::message::{Attachment, SinglePart};

#[derive(Debug)]
pub struct SeriesEntry<'a> {
    pub series: &'a db::Series,
    pub changes: &'a SeriesDetailsChanges,
    pub url: String,
    pub poster: db::Poster,
}

impl SeriesEntry<'_> {
    pub fn poster_attachment_id(&self) -> String {
        let series_id = self.series.details.id;
        let poster_file_ext = self.series.details.poster_extension().unwrap();
        format!("{series_id}.{poster_file_ext}")
    }

    pub fn poster_attachment_uri(&self) -> String {
        String::from("cid:") + self.poster_attachment_id().as_ref()
    }

    pub fn create_poster_attachment(&self) -> SinglePart {
        Attachment::new_inline(self.poster_attachment_id()).body(
            Vec::from(self.poster.img_data.clone()),
            self.poster.mime_type.clone().into(),
        )
    }
}

pub fn series_changes_to_entries<'a>(
    ctx: &mut AppContext,
    changes: &'a [(db::Series, SeriesDetailsChanges)],
) -> anyhow::Result<Box<[SeriesEntry<'a>]>> {
    let mut entries = Vec::new();

    for (series, changes) in changes.iter() {
        let Some(poster) = ctx
            .db
            .get_poster_by_id(series.poster_id)
            .with_context(|| format!("Querying poster for {}", series.details.identify()))?
        else {
            anyhow::bail!(
                "Could not find poster with ID {} for series {}",
                series.poster_id,
                series.details.identify()
            );
        };

        let series_id = changes.id;
        let new_entry = SeriesEntry {
            series,
            changes,
            url: ctx.tmdb.make_series_url(series_id),
            poster,
        };

        entries.push(new_entry);
    }

    Ok(entries.into_boxed_slice())
}
