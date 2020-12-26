use crate::template_helpers::{JamContext, UserOptional, UserOptionalContext};
use crate::{db::DbPool, models::Jam};
use rocket::{get, State};
use rocket_contrib::templates::Template;
use serde::Serialize;

#[get("/?<show_all_jams>")]
pub fn homepage(
    pool: State<'_, DbPool>,
    user: UserOptional,
    show_all_jams: Option<bool>,
) -> Result<Template, super::HandlerError> {
    let conn = pool.get()?;
    let should_show_all_jams =
        user.is_admin() && show_all_jams.unwrap_or(false);
    // load the first three approved jams
    let mut jams = Vec::new();
    for j in Jam::find_all(&conn, !should_show_all_jams, 0, 3)? {
        jams.push(JamContext::from_model(&conn, &j, false)?);
    }

    println!("{:?}", jams);

    #[derive(Debug, Serialize)]
    struct Context {
        auth: UserOptionalContext,
        jams: Vec<JamContext>,
        showing_all_jams: bool,
    }

    let context = Context {
        jams,
        auth: user.to_context(),
        showing_all_jams: should_show_all_jams,
    };

    Ok(Template::render("homepage", &context))
}
