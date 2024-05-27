use http::{error::frame::FrameError, uri::url::Url};

#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    Exact,
    Prefix,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    pub url: Url,
    pub match_type: MatchType,
}

impl TryFrom<Route> for String {
    type Error = FrameError;
    fn try_from(route: Route) -> Result<Self, Self::Error> {
        String::try_from(route.url)
    }
}
