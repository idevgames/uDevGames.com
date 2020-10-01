///! The name is a little sloppy but it's just generally stuff that has to do
///! with the template system.
use crate::{ db::DbPool, models::{ GhUserRecord, ModelError } };
use rocket::{ http::Status, request::{ FromRequest, Outcome, Request } };
use serde::Serialize;
use std::num::ParseIntError;
use thiserror::Error;


/// Request guard for which there may or may not be a logged in user. This is
/// for pages which can be viewed by anyone but which may change their controls
/// when viewed by someone who is logged in.
pub struct UserOptional {
    user: Option<GhUserRecord>
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
    avatar_url: String
}

impl UserOptional {
    /// Produces a serializable context that can be passed to a template.
    pub fn to_context(&self) -> UserOptionalContext {
        return UserOptionalContext {
            user: match &self.user {
                Some(u) => Some(UserOptionalContextUser {
                    id: u.id, login: u.login.clone(),
                    html_url: u.html_url.clone(),
                    avatar_url: u.avatar_url.clone()
                }),
                None => None
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum UserOptionalError {
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
        let pool = 
            req.managed_state::<DbPool>().unwrap();

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
                    match GhUserRecord::find_by_id_p(&pool, uid) {
                        Ok(u) => u,
                        Err(e) =>
                            return Outcome::Failure((
                                Status::BadRequest,
                                UserOptionalError::DbQueryError(e)
                            ))
                    };

                UserOptional { user }
            },
            None => UserOptional { user: None }
        };

        Outcome::Success(u)
    }
}

#[derive(Serialize)]
pub struct TemplateContext {

}