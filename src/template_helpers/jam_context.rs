use crate::db::DbConn;
use crate::models::{Attachment, Jam, ModelError, RichText};
use crate::template_helpers::attachment_context::AttachmentContext;
use serde::Serialize;

/// Describes what a [`crate::models::jams::Jam`] is to a Tera Template context.
#[derive(Debug, Serialize)]
struct JamContext {
    id: i32,
    title: String,
    slug: String,
    summary: String,
    summary_attachment: Option<AttachmentContext>,
    rich_text_content: Option<String>,
    start_date: String,
    end_date: String,
    approval_state: String,
}

impl JamContext {
    /// Populates a [`JamContext`] from a database model, pulling other related
    /// values from the database at `conn`. Optionally renders Markdown, so it
    /// can be omitted if not used in the page.
    pub fn from_model(conn: &DbConn, jam: &Jam, render_markdown: bool) -> Result<Self, ModelError> {
        let attachment =
            match jam.summary_attachment_id {
                Some(id) => Some(Attachment::find_by_id(conn, id)?.ok_or(ModelError::NotFound)?),
                None => None,
            };
        

        let rich_text = match RichText::find_by_id(conn, jam.rich_text_id)? {
            Some(rich_text) => rich_text,
            None => Err(ModelError::NotFound),
        };

        let rendered_rich_text_content = if render_markdown {
            Some(rich_text.render())
        } else {
            None
        };


    }
}
