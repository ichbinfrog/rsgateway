use std::str::FromStr;

use super::{
    header::{HeaderKind, HeaderMap},
    method::Method,
    request::Request,
    uri::{
        path::Query,
        url::Url,
    },
};

#[derive(Debug)]
pub struct Builder<T> {
    request: Request<T>,
}

impl<T> Builder<T> {
    pub fn new() -> Self {
        Self {
            request: Request::default(),
        }
    }

    fn update_url(&mut self, url: String) {
        let url = Url::from_str(&url).unwrap();
        self.request.parts.url = url.clone();
        let _ = self
            .request
            .parts
            .headers
            .put("host", HeaderKind::Host(url.authority));
    }

    pub fn get(mut self, url: String) -> Self {
        self.request.parts.method = Method::GET;
        self.update_url(url);
        self
    }

    pub fn head(mut self, url: String) -> Self {
        self.request.parts.method = Method::HEAD;
        self.update_url(url);
        self
    }

    pub fn query(mut self, q: Query) -> Self {
        self.request.parts.url.path.query = Some(q);
        self
    }

    pub fn headers(mut self, h: HeaderMap) -> Self {
        for (k, v) in h.raw.iter() {
            self.request
                .parts
                .headers
                .raw
                .insert(k.to_string(), v.to_string());
        }
        self
    }

    pub fn build(self) -> Request<T> {
        self.request
    }
}
