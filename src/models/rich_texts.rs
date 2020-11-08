use crate::{
    db::DbConn,
    models::{LastInsertRowid, ModelError},
};
use diesel::result::Error as DieselError;
use pulldown_cmark::{html, Options, Parser};

#[derive(Debug, Queryable)]
pub struct RichText {
    id: i32,
    content: String,
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

    /// Updates an existing RichText.
    pub fn update(
        conn: &DbConn,
        rich_text: &RichText,
    ) -> Result<(), ModelError> {
        use crate::schema::rich_texts::dsl::*;
        use diesel::prelude::*;

        diesel::update(rich_texts.find(rich_text.id))
            .set(content.eq(&rich_text.content))
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
