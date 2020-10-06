///! The name is a little sloppy but it's just generally stuff that has to do
///! with the template system.
use crate::{ db::DbPool, models::{ GhUserRecord, ModelError, Permission } };
use rocket::{ http::Status, request::{ FromRequest, Outcome, Request } };
use serde::Serialize;
use std::num::ParseIntError;
use thiserror::Error;


/// Request guard for which there may or may not be a logged in user. This is
/// for pages which can be viewed by anyone but which may change their controls
/// when viewed by someone who is logged in.
pub struct UserOptional {
    user: Option<GhUserRecord>,
    permissions: Vec<String>
}

#[derive(Debug, Serialize)]
pub struct UserOptionalContext {
    user: Option<UserOptionalContextUser>
}

#[derive(Debug, Serialize)]
struct UserOptionalContextUser {
    id: i64,
    login: String,
    html_url: String,
    avatar_url: String,
    permissions: Vec<String>,
}

impl UserOptional {
    /// Produces a serializable context that can be passed to a template.
    pub fn to_context(&self) -> UserOptionalContext {
        return UserOptionalContext {
            user: match &self.user {
                Some(u) => Some(UserOptionalContextUser {
                    id: u.id, login: u.login.clone(),
                    html_url: u.html_url.clone(),
                    avatar_url: u.avatar_url.clone(),
                    permissions: self.permissions.clone()
                }),
                None => None
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum UserOptionalError {
    #[error("Could not get a connection from the pool with error {0}")]
    DbPoolError(#[from] diesel::r2d2::PoolError),

    #[error("Could not parse uid from cookie with error {0}")]
    UserIdDecodeError(#[from] ParseIntError),

    #[error("Could not query the database with error {0}")]
    DbQueryError(#[from] ModelError)
}

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for UserOptional {
    type Error = UserOptionalError;

    async fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        // unwrap is okay here, if there's no pool then the entire application
        // bootstrap was wrong
        let pool = req.managed_state::<DbPool>().unwrap();
        let conn = match pool.get() {
            Ok(conn) => conn,
            Err(e) => return Outcome::Failure((
                Status::InternalServerError,
                UserOptionalError::DbPoolError(e)
            ))
        };

        // pull the user out of the cookie, if it's there
        let mut cookies = req.cookies();
        let user_id = cookies.get_private("gh_user_id");

        let u = match user_id {
            Some(cookie) => {
                let value = cookie.value();
                let uid = match str::parse::<i64>(value) {
                    Ok(uid) => uid,
                    Err(e) =>
                        return Outcome::Failure((
                            Status::BadRequest,
                            UserOptionalError::UserIdDecodeError(e)
                        ))
                };

                let user = 
                    match GhUserRecord::find_by_id_c(&conn, uid) {
                        Ok(u) => u,
                        Err(e) =>
                            return Outcome::Failure((
                                Status::BadRequest,
                                UserOptionalError::DbQueryError(e)
                            ))
                    };

                let permissions =
                    match Permission::find_by_gh_user_id_c(&conn, uid) {
                        Ok(perms) => perms,
                        Err(e) =>
                            return Outcome::Failure((
                                Status::BadRequest,
                                UserOptionalError::DbQueryError(e)
                            ))
                    };

                UserOptional {
                    user,
                    permissions: permissions.iter()
                        // TODO: would Cow work here?
                        .map(|perm| perm.name.clone()).collect()
                }
            },
            None => UserOptional { user: None, permissions: vec![] }
        };

        Outcome::Success(u)
    }
}
