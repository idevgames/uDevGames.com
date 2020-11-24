use chrono::NaiveDateTime;
use diesel::Connection;
use rocket::{get, post, uri, State};
use rocket::{
    request::{Form, FromForm},
    response::{Redirect, Responder},
};
use rocket_contrib::templates::Template;
use serde::Serialize;

use crate::{db::DbPool, models::ApprovalState, template_helpers::AdminOnly};
use crate::{
    models::{Jam, RichText},
    template_helpers::AdminOnlyContext,
};

// CREATE   /jams                   -> jam_id           ADMIN ONLY
// GET      /jams/:jam_id/edit      -> Jam              ADMIN ONLY
// UPDATE   /jams/:jam_id           -> Result<()>       ADMIN ONLY
// GET      /jams                   -> Vec<Jam>         All jams when admin,
// GET      /jams/:jam_id/:jam_slug -> Jam              otherwise only published
// DELETE   /jams/:jam_id           -> Result<()>       ADMIN ONLY
// GET      /jams/:jam_id/attachments                   find all attachments for
// GET      /jams/:jam_id/:jam_slug/attachments         a jam... probably ignorable
//                                  -> Vec<Attachment>
// CREATE   /jams/:jam_id/attachments                   create an attachment for this jam
//                                  -> Result<Attachment>

/// Creates a new blank jam and immediately redirects to its edit page.
#[post("/jams")]
pub async fn create_jam(
    pool: State<'_, DbPool>,
    _admin_only: AdminOnly,
) -> Result<Redirect, super::HandlerError> {
    let conn = pool.get()?;
    let jam = Jam::create(&conn)?;
    Ok(Redirect::to(uri!(edit_jam: jam.id)))
}

// this provides a barrier between the data model and what's rendered out to
// html, so we won't accidentally leak private data should it be added. it
// also makes it serializable so it can actually go into a template context.
// in addition to these three important functions, it flattens the RichText
// onto the base Jam model, which is more logical for the domain of template
// rendering.
#[derive(Debug, Serialize)]
struct JamContext {
    id: i32,
    title: String,
    slug: String,
    summary: String,
    summary_attachment_id: Option<i32>,
    rich_text_id: i32,
    rich_text_content: String,
    start_date: String,
    end_date: String,
    approval_state: String,
}

#[derive(Debug, Serialize)]
struct EditJamContext {
    auth: AdminOnlyContext,
    jam: JamContext,
}

/// Renders out a lovely form that you can use to edit the jam.
#[get("/jams/<jam_id>/edit")]
pub async fn edit_jam(
    pool: State<'_, DbPool>,
    admin_only: AdminOnly,
    jam_id: i32,
) -> Result<Template, super::HandlerError> {
    let conn = pool.get()?;
    let jam = match Jam::find_by_id(&conn, jam_id)? {
        Some(jam) => jam,
        None => return Err(super::HandlerError::NotFound),
    };

    let rich_text = match RichText::find_by_id(&conn, jam.rich_text_id)? {
        Some(rich_text) => rich_text,
        None => return Err(super::HandlerError::NotFound),
    };

    let context = EditJamContext {
        auth: admin_only.to_context(),
        jam: JamContext {
            id: jam.id,
            title: jam.title.clone(),
            slug: jam.slug.clone(),
            summary: jam.summary.clone(),
            summary_attachment_id: jam.summary_attachment_id,
            rich_text_id: jam.rich_text_id,
            rich_text_content: rich_text.content.clone(),
            start_date: format!("{}", jam.start_date),
            end_date: format!("{}", jam.end_date),
            approval_state: jam.approval_state.to_human_str(),
        },
    };

    Ok(Template::render("edit_jam", &context))
}

#[derive(Debug, FromForm)]
pub struct JamFormData {
    id: i32,
    title: String,
    slug: String,
    summary: String,
    // summary_attachment_id to be set by ajax
    // rich_text_id is already set, not changing that through web calls
    rich_text_content: String,
    start_date: String,
    end_date: String,
    approval_state: String,
}

#[post("/jams/<jam_id>", data = "<jam_form_data>")]
pub async fn update_jam(
    pool: State<'_, DbPool>,
    admin_only: AdminOnly,
    jam_id: i32,
    jam_form_data: Form<JamFormData>,
) -> Result<Template, super::HandlerError> {
    let conn = pool.get()?;

    // do operations in a transaction so that all the updates roll back on
    // failure
    let txr = conn.transaction::<(Jam, RichText), super::HandlerError, _>(|| {
        let mut jam = match Jam::find_by_id(&conn, jam_id)? {
            Some(jam) => jam,
            None => return Err(super::HandlerError::NotFound),
        };
        let mut rich_text = match RichText::find_by_id(&conn, jam.rich_text_id)? {
            Some(rich_text) => rich_text,
            None => return Err(super::HandlerError::NotFound),
        };

        jam.title = jam_form_data.title.clone();
        jam.slug = jam_form_data.slug.clone();
        jam.summary = jam_form_data.summary.clone();
        jam.start_date = jam_form_data.start_date.parse::<NaiveDateTime>()?;
        jam.end_date = jam_form_data.end_date.parse::<NaiveDateTime>()?;
        jam.approval_state = ApprovalState::from_human_str(&jam_form_data.approval_state)?;
        rich_text.content = jam_form_data.rich_text_content.clone();

        jam.update(&conn)?;
        rich_text.update(&conn)?;
        Ok((jam, rich_text))
    });

    let (jam, rich_text) = match txr {
        Ok((jam, rich_text)) => (jam, rich_text),
        Err(e) => return Err(e),
    };

    let context = EditJamContext {
        auth: admin_only.to_context(),
        jam: JamContext {
            id: jam.id,
            title: jam.title.clone(),
            slug: jam.slug.clone(),
            summary: jam.summary.clone(),
            summary_attachment_id: jam.summary_attachment_id,
            rich_text_id: jam.rich_text_id,
            rich_text_content: rich_text.content.clone(),
            start_date: format!("{}", jam.start_date),
            end_date: format!("{}", jam.end_date),
            approval_state: jam.approval_state.to_human_str(),
        },
    };

    Ok(Template::render("edit_jam", &context))
}
