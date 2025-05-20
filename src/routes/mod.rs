pub mod api;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter)]
pub enum Route {
    Health,
    Metrics,
    Docs,
    LinkRedirect,
    Links,
    LinkGet,
}

impl Route {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Health => "/health",
            Self::Metrics => "/metrics",
            Self::Docs => "/docs",
            Self::LinkRedirect => "/{link_id}",
            Self::Links => "/links",
            Self::LinkGet => "/links/{link_id}",
        }
    }
}
impl From<Route> for &str {
    fn from(value: Route) -> Self {
        value.as_str()
    }
}
impl AsRef<str> for Route {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
