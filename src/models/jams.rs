use chrono::NaiveDateTime;
use crate::models::ApprovalState;

/// Models a game jam.
#[derive(Queryable)]
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

}
