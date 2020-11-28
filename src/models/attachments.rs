use crate::{
    attachments::AttachmentStorage,
    db::DbConn,
    models::{r_to_opt, last_insert_rowid, ModelError},
};
use diesel::result::Error as DieselError;
use std::path::PathBuf;

/// An attachment, which is a file on disk.
#[derive(Debug, Queryable)]
pub struct Attachment {
    /// Unique id of this attachment.
    pub id: i32,

    /// The name of this attachment, which is the human or friendly name of it.
    pub name: String,

    /// Whether or not this attachment is visible to the world.
    pub published: bool,

    /// The MIME type, such as `image/png`, which is stored so that they can be
    /// served up idiomatically.
    pub mime_type: String,

    /// The MD5 of the file. If this differs from what is on disk, we may be
    /// experiencing bitrot or an attack.
    pub md5: Vec<u8>,
}

impl Attachment {
    /// Create a new attachment from a temporary file. Copies it to a permanent
    /// storage location and md5's it.
    pub fn create(
        conn: &DbConn,
        attachment_storage: &AttachmentStorage,
        the_file: impl AsRef<PathBuf>,
        the_name: &str,
        the_mime_type: &str,
    ) -> Result<Attachment, ModelError> {
        use crate::schema::attachments::dsl::{
            attachments, id, md5, mime_type, name,
        };
        use diesel::prelude::*;

        let the_file = the_file.as_ref();

        if !the_file.exists() {
            return Err(ModelError::FileNotFoundError(the_file.clone()));
        }

        let attachment =
            // transaction so last_insert_rowid doesn't do anything untoward
            conn.transaction::<Attachment, DieselError, _>(|| {
                diesel::insert_into(attachments)
                    .values((
                        name.eq(the_name), mime_type.eq(the_mime_type),
                        md5.eq(vec![]) // how to insert binary data?
                    ))
                    .execute(conn)?;
                let rowid = diesel::select(last_insert_rowid)
                    .get_result::<i32>(conn)?;
                Ok(
                    attachments
                        .filter(id.eq(rowid))
                        .limit(1)
                        .first::<Attachment>(conn)?
                )
            })?;

        let mut stored_attachment =
            attachment_storage.store(&the_file, attachment.id)?;

        diesel::update(attachments)
            .set(md5.eq(stored_attachment.get_or_compute_md5()?.to_vec()))
            .execute(conn)?;

        Ok(attachment)
    }

    /// Finds an attachment by its id, if it exists.
    pub fn find_by_id(
        conn: &DbConn,
        attachment_id: i32,
    ) -> Result<Option<Attachment>, ModelError> {
        use crate::schema::attachments::dsl::{attachments, id};
        use diesel::prelude::*;

        let attachment = attachments
            .filter(id.eq(attachment_id))
            .limit(1)
            .first::<Attachment>(conn);

        r_to_opt(attachment)
    }

    /// Finds an attachment by its id, if it exists, and only if it is
    /// published.
    pub fn find_published_by_id(
        conn: &DbConn,
        attachment_id: i32,
    ) -> Result<Option<Attachment>, ModelError> {
        use crate::schema::attachments::dsl::{attachments, id, published};
        use diesel::prelude::*;

        let attachment = attachments
            .filter(id.eq(attachment_id))
            .filter(published.eq(true))
            .limit(1)
            .first::<Attachment>(conn);

        r_to_opt(attachment)
    }
}
