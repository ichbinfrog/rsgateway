
use crate::{auth::authorization::Authorization, uri::path::Path};

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

    pub fn method(mut self, method: Method) -> Self {
        self.request.parts.method = method;
        self
    }

    pub fn basic_auth(mut self, user: &str, password: &str) -> Self {
        self.request
            .parts
            .headers
            .put(
                "authorization",
                HeaderKind::Authorization(Authorization::Basic {
                    user: user.to_string(),
                    password: password.to_string(),
                }),
            )
            .unwrap();
        self
    }

    pub fn url(mut self, url: Url) -> Self {
        self.request.parts.url = url.clone();
        let _ = self
            .request
            .parts
            .headers
            .put("host", HeaderKind::Host(url.authority));
        self
    }

    pub fn path(mut self, path: Path) -> Self {
        self.request.parts.url.path = path;
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
