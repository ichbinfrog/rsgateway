use super::{header::HeaderMap, statuscode::StatusCode, version::Version};

#[derive(Debug)]
pub struct Response<T> {
    pub status: StatusCode,
    pub version: Version,
    pub headers: HeaderMap,

    pub body: T,
}

impl<T> Response<T> {}
