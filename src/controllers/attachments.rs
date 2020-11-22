use crate::{
    attachments::AttachmentStorage,
    db::DbPool,
    models::Attachment,
};
use rocket::{get, response::Stream, tokio::fs::File, State};

// match stuff like udevgames.com/attachments/1/my_file.jpeg
#[get("/attachments/<id>/<_name>")]
pub async fn get_attachment(
    pool: State<'_, DbPool>,
    attachment_storage: State<'_, AttachmentStorage>,
    id: i32,
    _name: String,
) -> Result<Stream<File>, super::HandlerError> {
    let conn = pool.get()?;
    let attachment = match Attachment::find_published_by_id(&conn, id)? {
        Some(attachment) => attachment,
        None => return Err(super::HandlerError::NotFound),
    };
    let f = attachment_storage.load(attachment.id)?;
    Ok(Stream::from(File::from_std(f)))
}
