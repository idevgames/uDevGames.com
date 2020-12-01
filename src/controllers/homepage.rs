use crate::template_helpers::{UserOptional, JamContext, UserOptionalContext};
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
    // load the first three approved jams
    let mut jams = Vec::new();
    for j in Jam::find_all(&conn, true, 0, 3)? {
        jams.push(JamContext::from_model(&conn, &j, false)?);
    }

    #[derive(Debug, Serialize)]
    struct Context {
        auth: UserOptionalContext,
        jams: Vec<JamContext>,
    }

    let context = Context {
        auth: user.to_context(),
        jams: jams,
    };

    Ok(Template::render("homepage", &context))
}
