use std::{collections::HashMap};

use http::{
    error::frame::FrameError,
    uri::{authority::Authority, url::Url},
};

#[derive(Debug)]
pub enum MatchType {
    Exact,
    Prefix,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    pub url: Url,
}

impl TryFrom<Route> for String {
    type Error = FrameError;
    fn try_from(route: Route) -> Result<Self, Self::Error> {
        String::try_from(route.url)
    }
}

#[derive(Debug)]
pub struct Router {
    lookup: HashMap<Authority, Route>,
}
