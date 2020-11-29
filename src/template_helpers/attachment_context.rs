use crate::models::Attachment;
use hex::encode as hex_encode;
use serde::Serialize;

/// Describes what a [`crate::models::attachments::Attachment`] is to a Tera
/// template context.
#[derive(Debug, Serialize)]
pub struct AttachmentContext {
    id: i32,
    name: String,
    published: bool,
    mime_type: String,
    md5: String,
    url: String,
}

impl AttachmentContext {
    pub fn from_model(attachment: &Attachment) -> Self {
        AttachmentContext {
            id: attachment.id,
            name: attachment.name.clone(),
            published: attachment.published,
            mime_type: attachment.mime_type,
            md5: hex_encode(attachment.md5),
            url: attachment.url(),
        }
    }
}
