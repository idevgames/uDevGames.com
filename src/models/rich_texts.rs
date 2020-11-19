use crate::{
    db::DbConn,
    models::{LastInsertRowid, ModelError},
};
use diesel::result::Error as DieselError;
use pulldown_cmark::{html, Options, Parser};

use super::r_to_opt;

#[derive(Debug, Queryable)]
pub struct RichText {
    pub id: i32,
    pub content: String,
}

impl RichText {
    /// Creates a new blank RichText.
    pub fn create(conn: &DbConn) -> Result<RichText, ModelError> {
        use crate::schema::rich_texts::dsl::*;
        use diesel::prelude::*;

        let rich_text = conn.transaction::<RichText, DieselError, _>(|| {
            diesel::insert_into(rich_texts)
                .values(content.eq(""))
                .execute(conn)?;

            let last_row_id =
                diesel::select(LastInsertRowid).get_result::<i32>(conn)?;

            rich_texts
                .filter(id.eq(last_row_id))
                .limit(1)
                .first::<RichText>(conn)
        })?;

        Ok(rich_text)
    }

    /// Find a rich text by id.
    pub fn find_by_id(
        conn: &DbConn,
        rich_text_id: i32,
    ) -> Result<Option<RichText>, ModelError> {
        use crate::schema::rich_texts::dsl::*;
        use diesel::prelude::*;

        let rich_text = rich_texts
            .filter(id.eq(rich_text_id))
            .limit(1)
            .first::<RichText>(conn);

        r_to_opt(rich_text)
    }

    /// Updates an existing RichText.
    pub fn update(&self, conn: &DbConn) -> Result<(), ModelError> {
        use crate::schema::rich_texts::dsl::{content, rich_texts};
        use diesel::prelude::*;

        diesel::update(rich_texts.find(self.id))
            .set(content.eq(&self.content))
            .execute(conn)?;

        Ok(())
    }

    /// Renders the rich text's markdown to HTML.
    pub fn render(&self) -> String {
        let parser = Parser::new_ext(&self.content, Options::all());
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    }
}

#[derive(Debug, Queryable)]
pub struct RichTextAttachment {
    id: i32,
    rich_text_id: i32,
    attachment_id: i32,
}
