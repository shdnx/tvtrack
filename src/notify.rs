use crate::{CmdContext, Result, SeriesDetailsChanges};
use lettre::{message::header::ContentType, Message, SmtpTransport, Transport};

pub fn send_email_notifications(ctx: &CmdContext, all_changes: Vec<SeriesDetailsChanges>) -> Result<()> {
    
    Ok(())
}
