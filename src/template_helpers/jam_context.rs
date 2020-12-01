use crate::db::DbConn;
use crate::models::{Jam, ModelError};
use crate::template_helpers::attachment_context::AttachmentContext;
use serde::Serialize;

/// Describes what a [`crate::models::jams::Jam`] is to a Tera Template context.
#[derive(Debug, Serialize)]
pub struct JamContext {
    id: i32,
    title: String,
    slug: String,
    summary: String,
    summary_attachment: Option<AttachmentContext>,
    rich_text_content: String,
    rendered_rich_text_content: Option<String>,
    start_date: String,
    end_date: String,
    approval_state: String,
}

impl JamContext {
    /// Populates a [`JamContext`] from a database model, pulling other related
    /// values from the database at `conn`. Optionally renders Markdown, so it
    /// can be omitted if not used in the page.
    pub fn from_model(
        conn: &DbConn,
        jam: &Jam, render_markdown: bool
    ) -> Result<Self, ModelError> {
        let attachment = jam.load_attachment(conn)?;
        let rich_text = jam.load_rich_text(conn)?;
        let rendered_rich_text_content = if render_markdown {
            Some(rich_text.render())
        } else {
            None
        };

        Ok(JamContext {
            id: jam.id,
            title: jam.title.clone(),
            slug: jam.slug.clone(),
            summary: jam.summary.clone(),
            summary_attachment: attachment.map(|a| AttachmentContext::from_model(&a)),
            rich_text_content: rich_text.content.clone(),
            rendered_rich_text_content: rendered_rich_text_content,
            start_date: jam.start_date.to_string(),
            end_date: jam.end_date.to_string(),
            approval_state: jam.approval_state.to_human_str(),
        })
    }
}
