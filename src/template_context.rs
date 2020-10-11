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
    user: Option<UserOptionalContextUser>,
}

/// The fields that describe a user in a template context.
#[derive(Debug, Serialize)]
struct UserOptionalContextUser {
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

/// Various kinds of error getting a `UserOptional` from a `Request`.
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

/// Drives the navbar's breadcrumbs to show hierarchy and stuff.
pub struct Breadcrumbs(Vec<Breadcrumb>);

/// A piece in the navbar. You can order these arbitrarily to create nonsensical
/// navbars, but please don't do that.
pub enum Breadcrumb {
    /// Will link to the homepage.
    Home
}

/// Unwraps the concept of a breadcrumb from a higher-level abstraction into a
/// form compatible with the template engine. This should always be in the
/// `breadcrumbs` field of a template context. To suppress the navbar completely
/// simply do not supply a `BreadcrumbsContext` at all.
#[derive(Debug, Serialize)]
pub struct BreadcrumbsContext(Vec<BreadcrumbContext>);

/// The individual piece of a breadcrumb handed off to a template.
#[derive(Debug, Serialize)]
pub struct BreadcrumbContext {
    content: String,
    href: String,
}

impl Breadcrumbs {
    /// Creates a new Breadcrumbs, with the home page added if the crumbs are
    /// empty.
    pub fn from_crumbs(crumbs: Vec<Breadcrumb>) -> Breadcrumbs {
        Breadcrumbs(
            if crumbs.is_empty() {
                vec![Breadcrumb::Home]
            } else {
                crumbs
            }
        )
    }

    /// Convert these Breadcrumbs to something that can be put into a template
    /// context.
    pub fn to_context(&self) -> BreadcrumbsContext {
        BreadcrumbsContext(
            self.0.iter()
                .map(|crumb| crumb.to_breadcrumb_context())
                .collect()
        )
    }
}

impl Breadcrumb {
    fn to_breadcrumb_context(&self) -> BreadcrumbContext {
        match self {
            Breadcrumb::Home => BreadcrumbContext::new("Home", "/")
        }
    }
}

impl BreadcrumbContext {
    fn new(content: &str, href: &str) -> BreadcrumbContext {
        BreadcrumbContext {
            content: content.to_string(),
            href: href.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::template_context::*;
    use rocket_contrib::templates::tera::{ Context, Tera, };

    /// Validates the detection of a logged in user in a template. If this
    /// breaks (highly unlikely) then a number of templates also need to be
    /// updated.
    #[test]
    fn test_user_optional_template_context() {
        let none_context = UserOptionalContext { user: None };
        let some_context = UserOptionalContext {
            user: Some(UserOptionalContextUser {
                id: 1, login: "ed".to_string(), html_url: "".to_string(),
                avatar_url: "".to_string(),
                permissions: vec!["admin".to_string()]
            })
        };
        let mut tera = Tera::default();
        tera.add_raw_template("example.html", "
            {% if user is object %}
            The user is logged in!
            {% else %}
            There is no user logged in.
            {% endif %}
        ").unwrap();
        let none_result = tera.render(
            "example.html",
            &Context::from_serialize(&none_context).unwrap()
        ).unwrap();
        let some_result = tera.render(
            "example.html",
            &Context::from_serialize(&some_context).unwrap()
        ).unwrap();
        assert_eq!("There is no user logged in.", none_result.trim());
        assert_eq!("The user is logged in!", some_result.trim());
    }
}
