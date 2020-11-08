use rocket::response::{Redirect, Responder};
use rocket::{post, uri, State};

use crate::db::DbPool;

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

#[derive(Responder)]
enum CreateJamError {
    #[response(status = 500)]
    PoolError(String),
}

#[post("/jams")]
pub async fn create_jam(
    db_pool: State<'_, DbPool>,
    /* TODO: admin only */
) -> Result<Redirect, CreateJamError> {
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return Err(CreateJamError::PoolError(
                "Couldn't get out of the pool. Send a lifeguard.".to_owned(),
            ))
        }
    };

    Ok(Redirect::to(uri!(crate::controllers::homepage::homepage)))
}
