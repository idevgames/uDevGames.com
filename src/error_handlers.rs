use rocket::catch;
use rocket_contrib::templates::Template;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct ErrorContext {
  message: String,
  suppress_auth_controls: bool,
}

impl ErrorContext {
  pub fn new(code: i32, message: &str) -> Self {
    Self {
      message: format!("{}: {}", code, message),
      suppress_auth_controls: true,
    } 
  }
}

#[catch(404)]
pub fn not_found() -> Template {
  Template::render("error_page", &ErrorContext::new(404, "Page not found."))
}

#[catch(401)]
pub fn not_authorized() -> Template {
  Template::render("error_page", &ErrorContext::new(401, "You must be logged in to view this page."))
}

#[catch(403)]
pub fn forbidden() -> Template {
  Template::render("error_page", &ErrorContext::new(403, "You cannot view this page."))
}

#[catch(500)]
pub fn server_error() -> Template {
  Template::render("error_page", &ErrorContext::new(500, "Internal Server Error."))
}

