///! The name is a little sloppy but it's just generally stuff that has to do
///! with the template system.
use serde::Serialize;

/// Drives the navbar's breadcrumbs to show hierarchy and stuff.
pub struct Breadcrumbs(Vec<Breadcrumb>);

/// A piece in the navbar. You can order these arbitrarily to create nonsensical
/// navbars, but please don't do that.
pub enum Breadcrumb {
    /// Will link to the homepage.
    Home,
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
        Breadcrumbs(if crumbs.is_empty() {
            vec![Breadcrumb::Home]
        } else {
            crumbs
        })
    }

    /// Convert these Breadcrumbs to something that can be put into a template
    /// context.
    pub fn to_context(&self) -> BreadcrumbsContext {
        BreadcrumbsContext(
            self.0
                .iter()
                .map(|crumb| crumb.to_breadcrumb_context())
                .collect(),
        )
    }
}

impl Breadcrumb {
    fn to_breadcrumb_context(&self) -> BreadcrumbContext {
        match self {
            Breadcrumb::Home => BreadcrumbContext::new("Home", "/"),
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
