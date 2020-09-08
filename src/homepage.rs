use crate::template_context::TemplateContext;
use rocket::get;
use rocket_contrib::templates::Template;


#[get("/")]
pub fn homepage() -> Template {
    let context = TemplateContext { };
    Template::render("homepage", &context)
}
