use std::str::FromStr;

use super::{
    header::{HeaderKind, HeaderMap},
    method::Method,
    request::Request,
    uri::{path::Query, url::Url},
};

#[derive(Debug)]
pub struct Builder {
    request: Request,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            request: Request::default(),
        }
    }

    fn update_url(&mut self, url: &str) {
        let url = Url::from_str(url).unwrap();
        self.request.parts.url = url.clone();
        let _ = self
            .request
            .parts
            .headers
            .put("host", HeaderKind::Host(url.authority));
    }

    pub fn method(mut self, method: Method) -> Self {
        self.request.parts.method = method;
        self
    }

    pub fn url(mut self, url: &str) -> Self {
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

    pub fn body(mut self, buf: Option<Vec<u8>>) -> Self {
        self.request.body = buf;
        self
    }

    pub fn build(self) -> Request {
        self.request
    }
}
