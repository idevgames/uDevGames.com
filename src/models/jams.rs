use crate::db::DbConn;
use crate::models::{
    last_insert_rowid, ApprovalState, Attachment, ModelError, RichText,
};
use chrono::NaiveDateTime;

use super::r_to_opt;

/// Models a game jam.
#[derive(Debug, Queryable)]
pub struct Jam {
    pub id: i32,
    pub title: String,
    pub slug: String,
    pub summary: String,
    pub summary_attachment_id: Option<i32>,
    pub rich_text_id: i32,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub approval_state: ApprovalState,
}

impl Jam {
    pub fn create(conn: &DbConn) -> Result<Jam, ModelError> {
        use crate::schema::jams::dsl::{
            approval_state, end_date, id, jams, rich_text_id, slug, start_date,
            summary, summary_attachment_id, title,
        };
        use diesel::prelude::*;

        let jam = conn.transaction::<Jam, ModelError, _>(|| {
            let rich_text = RichText::create(conn)?;

            diesel::insert_into(jams)
                .values((
                    title.eq("My Jam"),
                    slug.eq("my-jam"),
                    summary.eq("My really cool game jam"),
                    summary_attachment_id.eq::<Option<i32>>(None),
                    rich_text_id.eq(rich_text.id),
                    start_date.eq(chrono::offset::Utc::now().naive_utc()),
                    end_date.eq(chrono::offset::Utc::now().naive_utc()),
                    approval_state.eq(ApprovalState::Draft),
                ))
                .execute(conn)?;

            let rowid =
                diesel::select(last_insert_rowid).get_result::<i32>(conn)?;

            Ok(jams.filter(id.eq(rowid)).limit(1).first::<Jam>(conn)?)
        })?;

        Ok(jam)
    }

    /// Finds a Jam by its id.
    pub fn find_by_id(
        conn: &DbConn,
        jam_id: i32,
    ) -> Result<Option<Jam>, ModelError> {
        use crate::schema::jams::dsl::*;
        use diesel::prelude::*;

        let jam = jams.filter(id.eq(jam_id)).limit(1).first::<Jam>(conn);

        r_to_opt(jam)
    }

    /// Finds all Jams, paging them.
    ///
    /// * `approved_only` when `true` returns only [`crate::models::jams::Jam`]s
    ///   which have the the `approval_state`
    ///   [`crate::models::ApprovalState::Approved`].
    /// * `page` is the page in the results to retrieve.
    /// * `page_size` is the length of each page.
    pub fn find_all(
        conn: &DbConn,
        approved_only: bool,
        page: i64,
        page_size: i64,
    ) -> Result<Vec<Jam>, ModelError> {
        use crate::schema::jams::dsl::*;
        use diesel::prelude::*;

        let q = jams
            .order(start_date.desc())
            .limit(page_size)
            .offset(page * page_size);

        let r = if approved_only {
            q.filter(approval_state.eq(ApprovalState::Approved))
                .load::<Jam>(conn)?
        } else {
            q.load::<Jam>(conn)?
        };

        Ok(r)
    }

    /// Updates a Jam by making what's in the database look like what's on the
    /// model.
    pub fn update(&self, conn: &DbConn) -> Result<(), ModelError> {
        use crate::schema::jams::dsl::{
            approval_state, end_date, jams, rich_text_id, slug, start_date,
            summary, summary_attachment_id, title,
        };
        use diesel::prelude::*;

        diesel::update(jams.find(self.id))
            .set((
                title.eq(&self.title),
                slug.eq(&self.slug),
                summary.eq(&self.summary),
                summary_attachment_id.eq(self.summary_attachment_id),
                rich_text_id.eq(self.rich_text_id),
                start_date.eq(self.start_date),
                end_date.eq(self.end_date),
                approval_state.eq(self.approval_state),
            ))
            .execute(conn)?;

        Ok(())
    }

    /// Loads the summary attachment, if present.
    pub fn load_attachment(
        &self,
        conn: &DbConn,
    ) -> Result<Option<Attachment>, ModelError> {
        match self.summary_attachment_id {
            Some(id) => Ok(Attachment::find_by_id(conn, id)?),
            None => Ok(None),
        }
    }

    /// Loads the rich text summary of this Jam. Since every Jam has a rich text
    /// summary, the absence of this is a schema violation and is returned as an
    /// [`crate::models::ModelError::NotFound`].
    pub fn load_rich_text(
        &self,
        conn: &DbConn,
    ) -> Result<RichText, ModelError> {
        RichText::find_by_id(conn, self.rich_text_id)?
            .ok_or(ModelError::NotFound)
    }
}
