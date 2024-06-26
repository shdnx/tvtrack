use super::MimeType;

#[derive(Debug)]
pub struct Poster {
    pub img_data: Box<[u8]>,
    pub mime_type: MimeType,
    pub source_url: String,
}
