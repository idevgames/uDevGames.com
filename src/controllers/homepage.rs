use crate::template_helpers::{UserOptional, UserOptionalContext};
use crate::{db::DbPool, models::Jam};
use rocket::{get, State};
use rocket_contrib::templates::Template;
use serde::Serialize;

#[get("/")]
pub fn homepage(
    pool: State<'_, DbPool>,
    user: UserOptional,
) -> Result<Template, super::HandlerError> {
    let conn = pool.get()?;
    // load the firs three approved jams
    let jams = Jam::find_all(&conn, true, 0, 3)?;

    #[derive(Debug, Serialize)]
    struct Context {
        auth: UserOptionalContext,
        jams: Vec<Jam>,
    }

    let context = Context {
        auth: user.to_context(),
        jams: jams,
    };

    Ok(Template::render("homepage", &context))
}
