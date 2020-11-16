use crate::{
    models::{GhUserRecord, ModelError},
    template_helpers::TemplateContextUser,
};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
};
use serde::Serialize;
use std::num::ParseIntError;
use thiserror::Error;

use super::{auth_from_request, AuthFromRequestError};

/// Request guard for which there must be a logged in user with the admin role.
/// This is for pages which cannot be accessed except by an admin.
pub struct AdminOnly {
    /// The admin user.
    user: GhUserRecord,
    /// The permissions of the admin user.
    permissions: Vec<String>,
}

/// This is the context that goes to the template itself. This should always be
/// in the `auth` field of a template context.
#[derive(Debug, Serialize)]
pub struct AdminOnlyContext {
    /// The user.
    user: TemplateContextUser,
}

impl AdminOnly {
    /// Produces a serializable context that can be passed to a template.
    pub fn to_context(&self) -> AdminOnlyContext {
        AdminOnlyContext {
            user: TemplateContextUser {
                id: self.user.id,
                login: self.user.login.clone(),
                html_url: self.user.html_url.clone(),
                avatar_url: self.user.avatar_url.clone(),
                permissions: self.permissions.clone(),
            },
        }
    }
}

#[derive(Debug, Error)]
pub enum AdminOnlyError {
    #[error("The user is not an admin")]
    NotAdmin,

    #[error("No user is logged in")]
    NotLoggedIn,

    #[error("Could not get a connection from the pool with error {0}")]
    DbPoolError(#[from] diesel::r2d2::PoolError),

    #[error("Could not parse uid from cookie with error {0}")]
    UserIdDecodeError(#[from] ParseIntError),

    #[error("Could not query the database with error {0}")]
    DbQueryError(#[from] ModelError),
}

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for AdminOnly {
    type Error = AdminOnlyError;

    async fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match auth_from_request(req) {
            Ok(Some((user, permissions))) => {
                if permissions.contains(&"admin".to_string()) {
                    Outcome::Success(AdminOnly { user, permissions })
                } else {
                    Outcome::Failure((
                        Status::Forbidden,
                        AdminOnlyError::NotAdmin,
                    ))
                }
            }
            Ok(None) => Outcome::Failure((
                Status::Forbidden,
                AdminOnlyError::NotLoggedIn,
            )),
            Err(e) => match e {
                AuthFromRequestError::DbPoolError(e) => Outcome::Failure((
                    Status::InternalServerError,
                    AdminOnlyError::DbPoolError(e),
                )),
                AuthFromRequestError::UserIdDecodeError(e) => Outcome::Failure(
                    (Status::BadRequest, AdminOnlyError::UserIdDecodeError(e)),
                ),
                AuthFromRequestError::DbQueryError(e) => Outcome::Failure((
                    Status::BadRequest,
                    AdminOnlyError::DbQueryError(e),
                )),
            },
        }
    }
}
