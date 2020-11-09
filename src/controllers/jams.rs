use rocket::response::{Redirect, Responder};
use rocket::{get, post, uri, State};

use crate::db::DbPool;
use crate::models::Jam;

// CREATE   /jams                   -> jam_id           ADMIN ONLY
// GET      /jams/:jam_id/edit      -> Jam              ADMIN ONLY
// UPDATE   /jams/:jam_id           -> Result<()>       ADMIN ONLY
// GET      /jams                   -> Vec<Jam>         All jams when admin,
// GET      /jams/:jam_id/:jam_slug -> Jam              otherwise only published
// DELETE   /jams/:jam_id           -> Result<()>       ADMIN ONLY
// GET      /jams/:jam_id/attachmens                    find all attachments for
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
    /* TODO: admin only */
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

/// Errors that may  be encountered when editing a jam.
#[derive(Responder)]
pub enum EditJamError {}

/// Renders out a lovely form that you can use to edit the jam.
#[get("/jams/<jam_id>/edit")]
pub async fn edit_jam(
    db_pool: State<'_, DbPool>,
    /* TODO: admin only */
    jam_id: i32,
) -> Result<Redirect, EditJamError> {
    Ok(Redirect::to(uri!(crate::controllers::homepage::homepage)))
}
