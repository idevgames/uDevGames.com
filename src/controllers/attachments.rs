use crate::{
    attachments::{AttachmentStorage, AttachmentStorageError},
    db::DbPool,
    models::Attachment,
};
use rocket::{get, response::Stream, tokio::fs::File, Responder, State};

/// Errors getting an attachment.
#[derive(Debug, Responder)]
pub enum GetAttachmentError {
    /// No such attachment exists in record, or isn't public, at any rate.
    #[response(status = 404)]
    AttachmentNotFound(String),

    /// Could not query the database for an attachment.
    #[response(status = 500)]
    DbError(String),

    /// The attachment is more or less valid, but the file on disk isn't there.
    #[response(status = 500)]
    FileNotFound(String),

    /// The attachment could not be read from disk. Weird.
    IoError(String),

    /// Could not get a connection to the database out of the pool.
    #[response(status = 500)]
    PoolError(String),
}

// match stuff like udevgames.com/attachments/1/my_file.jpeg
#[get("/attachments/<id>/<_name>")]
pub async fn get_attachment(
    pool: State<'_, DbPool>,
    attachment_storage: State<'_, AttachmentStorage>,
    id: i32,
    _name: String,
) -> Result<Stream<File>, GetAttachmentError> {
    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return Err(GetAttachmentError::PoolError(format!("{:?}", e)))
        }
    };
    let attachment = match Attachment::find_published_by_id(&conn, id) {
        Ok(attachment) => match attachment {
            Some(attachment) => attachment,
            None => {
                return Err(GetAttachmentError::AttachmentNotFound(
                    "Not found. Send in Search and Rescue.".to_string(),
                ))
            }
        },
        Err(e) => return Err(GetAttachmentError::DbError(format!("{:?}", e))),
    };
    let f = match attachment_storage.load(attachment.id) {
        Ok(f) => f,
        Err(AttachmentStorageError::NotFound(path)) => {
            return Err(GetAttachmentError::FileNotFound(format!("{:?}", path)))
        }
        Err(AttachmentStorageError::IoError(e)) => {
            return Err(GetAttachmentError::IoError(format!("{:?}", e)))
        }
    };
    Ok(Stream::from(File::from_std(f)))
}
