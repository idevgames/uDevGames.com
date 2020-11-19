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

/// Errors that may be encountered when creating a new jam.
#[derive(Responder)]
pub enum CreateJamError {
    /// Couldn't get out of the pool. Send a lifeguard.
    #[response(status = 500)]
    PoolError(String),

    /// Couldn't use the database. Send a DBA.
    #[response(status = 500)]
    DatabaseError(String),
}

/// Creates a new blank jam and immediately redirects to its edit page.
#[post("/jams")]
pub async fn create_jam(
    db_pool: State<'_, DbPool>,
    _admin_only: AdminOnly,
) -> Result<Redirect, CreateJamError> {
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return Err(CreateJamError::PoolError(format!(
            "Couldn't get out of the pool with error {:?}. Send a lifeguard.",
            e
        )))
        }
    };

    let jam = match Jam::create(&conn) {
        Ok(jam) => jam,
        Err(e) => {
            return Err(CreateJamError::DatabaseError(format!("{:?}", e)))
        }
    };

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
/// Errors that may  be encountered when editing a jam.
#[derive(Responder)]
pub enum EditJamError {
    /// Couldn't get out of the pool. Send a lifeguard.
    #[response(status = 500)]
    PoolError(String),

    /// Couldn't use the database. Send a DBA.
    #[response(status = 500)]
    DatabaseError(String),

    /// The user requested a jam that does not exist.
    #[response(status = 404)]
    NoSuchJam(String),

    /// The Jam is found but references a RichText that doesn't exist.
    #[response(status = 404)]
    NoAttachedText(String),
}

/// Renders out a lovely form that you can use to edit the jam.
#[get("/jams/<jam_id>/edit")]
pub async fn edit_jam(
    db_pool: State<'_, DbPool>,
    admin_only: AdminOnly,
    jam_id: i32,
) -> Result<Template, EditJamError> {
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return Err(EditJamError::PoolError(format!(
            "Couldn't get out of the pool with error {:?}. Send a lifeguard.",
            e)))
        }
    };

    let jam = match Jam::find_by_id(&conn, jam_id) {
        Ok(Some(jam)) => jam,
        Ok(None) => {
            return Err(EditJamError::NoSuchJam(format!(
                "No Jam with ID {}",
                jam_id
            )))
        }
        Err(e) => {
            return Err(EditJamError::DatabaseError(format!(
                "Could not retrieve jam with error {:?}",
                e
            )))
        }
    };

    let rich_text = match RichText::find_by_id(&conn, jam.rich_text_id) {
        Ok(Some(rich_text)) => rich_text,
        Ok(None) => {
            return Err(EditJamError::NoAttachedText(format!(
                "No RichText entry with ID {}",
                jam.rich_text_id
            )))
        }
        Err(e) => {
            return Err(EditJamError::DatabaseError(format!(
                "Could not retrieve rich text with error {:?}",
                e
            )))
        }
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

#[derive(Debug, Responder)]
pub enum UpdateJamError {
    /// Couldn't get out of the pool. Send a lifeguard.
    #[response(status = 500)]
    PoolError(String),

    /// Couldn't use the database. Send a DBA.
    #[response(status = 500)]
    DatabaseError(String),

    /// The user requested a jam that does not exist.
    #[response(status = 404)]
    NoSuchJam(String),

    /// The Jam is found but references a RichText that doesn't exist.
    #[response(status = 404)]
    NoAttachedText(String),

    /// The date format is not ISO-8601.
    #[response(status = 400)]
    InvalidDateFormat(String),

    /// The approval state does not match the list.
    #[response(status = 400)]
    InvalidApprovalState(String),
}

impl From<diesel::result::Error> for UpdateJamError {
    fn from(e: diesel::result::Error) -> Self {
        return UpdateJamError::DatabaseError(format!(
            "Could not access the database with error {:?}",
            e
        ));
    }
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
    db_pool: State<'_, DbPool>,
    admin_only: AdminOnly,
    jam_id: i32,
    jam_form_data: Form<JamFormData>,
) -> Result<Template, UpdateJamError> {
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return Err(UpdateJamError::PoolError(format!(
                "Couldn't get out of the pool with error {:?}",
                e
            )))
        }
    };

    // do operations in a transaction so that all the updates roll back on
    // failure
    let txr = conn.transaction::<(Jam, RichText), UpdateJamError, _>(|| {
        let mut jam = match Jam::find_by_id(&conn, jam_id) {
            Ok(Some(jam)) => jam,
            Ok(None) => {
                return Err(UpdateJamError::NoSuchJam(format!(
                    "No Jam with ID {}",
                    jam_id
                )))
            }
            Err(e) => {
                return Err(UpdateJamError::DatabaseError(format!(
                    "Could not retrieve from database with error {:?}",
                    e
                )))
            }
        };
        let mut rich_text = match RichText::find_by_id(&conn, jam.rich_text_id)
        {
            Ok(Some(rich_text)) => rich_text,
            Ok(None) => {
                return Err(UpdateJamError::NoAttachedText(format!(
                    "No rich text entry with ID {}",
                    jam.rich_text_id
                )))
            }
            Err(e) => {
                return Err(UpdateJamError::DatabaseError(format!(
                    "Could not retrieve from database with error {:?}",
                    e
                )))
            }
        };

        jam.title = jam_form_data.title.clone();
        jam.slug = jam_form_data.slug.clone();
        jam.summary = jam_form_data.summary.clone();
        jam.start_date = match jam_form_data.start_date.parse::<NaiveDateTime>()
        {
            Ok(t) => t,
            Err(_e) => {
                return Err(UpdateJamError::InvalidDateFormat(format!(
                    "Date {} not a valid date",
                    jam_form_data.start_date
                )))
            }
        };
        jam.end_date = match jam_form_data.end_date.parse::<NaiveDateTime>() {
            Ok(t) => t,
            Err(_e) => {
                return Err(UpdateJamError::InvalidDateFormat(format!(
                    "Date {} not a valid date",
                    jam_form_data.end_date
                )))
            }
        };
        jam.approval_state = match ApprovalState::from_human_str(
            &jam_form_data.approval_state,
        ) {
            Ok(approval_state) => approval_state,
            Err(_e) => {
                return Err(UpdateJamError::InvalidApprovalState(format!(
                    "Supplied approval state {} is invalid",
                    jam_form_data.approval_state
                )))
            }
        };
        rich_text.content = jam_form_data.rich_text_content.clone();

        match jam.update(&conn) {
            Ok(()) => {}
            Err(e) => {
                return Err(UpdateJamError::DatabaseError(format!(
                    "Could not update database with error {:?}",
                    e
                )))
            }
        }

        match rich_text.update(&conn) {
            Ok(()) => {}
            Err(e) => {
                return Err(UpdateJamError::DatabaseError(format!(
                    "Could not update database with error {:?}",
                    e
                )))
            }
        }

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
