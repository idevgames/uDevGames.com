mod admin_only;
mod breadcrumbs;
mod user_optional;

use std::num::ParseIntError;

use rocket::Request;
use serde::Serialize;

pub use crate::template_helpers::{
    admin_only::*, breadcrumbs::*, user_optional::*,
};
use crate::{
    db::DbPool,
    models::{GhUserRecord, ModelError, Permission},
};
use thiserror::Error;

#[derive(Debug, Serialize)]
struct TemplateContextUser {
    /// The user's numeric id.
    id: i64,

    /// The user's Github login.
    login: String,

    /// The user's Github profile link.
    html_url: String,

    /// The user's avatar.
    avatar_url: String,

    /// List of the user's permissions.
    permissions: Vec<String>,
}

#[derive(Debug, Error)]
pub enum AuthFromRequestError {
    #[error("Could not get a connection from the pool with error {0}")]
    DbPoolError(#[from] diesel::r2d2::PoolError),

    #[error("Could not parse uid from cookie with error {0}")]
    UserIdDecodeError(#[from] ParseIntError),

    #[error("Could not query the database with error {0}")]
    DbQueryError(#[from] ModelError),
}

fn auth_from_request<'a, 'r>(
    req: &'a Request<'r>,
) -> Result<Option<(GhUserRecord, Vec<String>)>, AuthFromRequestError> {
    // unwrap is okay here, if there's no pool then the entire application
    // bootstrap was wrong
    let pool = req.managed_state::<DbPool>().unwrap();
    let conn = pool.get()?;

    // pull the user out of the cookie, if it's there
    let cookies = req.cookies();
    let user_id = cookies.get_private("gh_user_id");

    match user_id {
        Some(cookie) => {
            let value = cookie.value();
            let uid = str::parse::<i64>(value)?;
            let user = match GhUserRecord::find_by_id(&conn, uid)? {
                Some(user) => user,
                None => {
                    // remove the nonexistent user from the cookie, effectively
                    // logging out the user
                    cookies.remove_private(cookie);
                    return Ok(None);
                }
            };

            let permissions = Permission::find_by_gh_user_id(&conn, uid)?
                .iter()
                .map(|p| p.name.clone())
                .collect();

            return Ok(Some((user, permissions)));
        }
        None => return Ok(None),
    };
}
