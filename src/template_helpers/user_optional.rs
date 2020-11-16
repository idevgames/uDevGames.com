//! This request guard is also a template helper because it provides the user
//! and permissions to a template context.
use crate::{models::GhUserRecord, template_helpers::TemplateContextUser};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
};
use serde::Serialize;

use super::{auth_from_request, AuthFromRequestError};

/// Request guard for which there may or may not be a logged in user. This is
/// for pages which can be viewed by anyone but which may change their controls
/// when viewed by someone who is logged in.
pub struct UserOptional {
    /// The current user, or is it?
    user: Option<GhUserRecord>,

    /// The permissions the current user has, if any.
    permissions: Vec<String>,
}

/// This is the context that goes to the template itself. To check for the
/// presence of a user, use the `is object` test. This should always be in the
/// `auth` field of a template context.
#[derive(Debug, Serialize)]
pub struct UserOptionalContext {
    /// The user, or is it?
    user: Option<TemplateContextUser>,
}

impl UserOptional {
    /// Produces a serializable context that can be passed to a template.
    pub fn to_context(&self) -> UserOptionalContext {
        return UserOptionalContext {
            user: match &self.user {
                Some(u) => Some(TemplateContextUser {
                    id: u.id,
                    login: u.login.clone(),
                    html_url: u.html_url.clone(),
                    avatar_url: u.avatar_url.clone(),
                    permissions: self.permissions.clone(),
                }),
                None => None,
            },
        };
    }
}

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for UserOptional {
    type Error = AuthFromRequestError;

    async fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match auth_from_request(req) {
            Ok(Some((user, permissions))) => Outcome::Success(UserOptional {
                user: Some(user),
                permissions,
            }),
            Ok(None) => Outcome::Success(UserOptional {
                user: None,
                permissions: vec![],
            }),
            Err(e) => match e {
                AuthFromRequestError::DbPoolError(_) => {
                    Outcome::Failure((Status::InternalServerError, e))
                }
                AuthFromRequestError::UserIdDecodeError(_) => {
                    Outcome::Failure((Status::BadRequest, e))
                }
                AuthFromRequestError::DbQueryError(_) => {
                    Outcome::Failure((Status::BadRequest, e))
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::template_helpers::user_optional::*;
    use rocket_contrib::templates::tera::{Context, Tera};

    /// Validates the detection of a logged in user in a template. If this
    /// breaks (highly unlikely) then a number of templates also need to be
    /// updated.
    #[test]
    fn test_user_optional_template_context() {
        let none_context = UserOptionalContext { user: None };
        let some_context = UserOptionalContext {
            user: Some(TemplateContextUser {
                id: 1,
                login: "ed".to_string(),
                html_url: "".to_string(),
                avatar_url: "".to_string(),
                permissions: vec!["admin".to_string()],
            }),
        };
        let mut tera = Tera::default();
        tera.add_raw_template(
            "example.html",
            "
            {% if user is object %}
            The user is logged in!
            {% else %}
            There is no user logged in.
            {% endif %}
        ",
        )
        .unwrap();
        let none_result = tera
            .render(
                "example.html",
                &Context::from_serialize(&none_context).unwrap(),
            )
            .unwrap();
        let some_result = tera
            .render(
                "example.html",
                &Context::from_serialize(&some_context).unwrap(),
            )
            .unwrap();
        assert_eq!("There is no user logged in.", none_result.trim());
        assert_eq!("The user is logged in!", some_result.trim());
    }
}
