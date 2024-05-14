use std::error::Error;

use tokio::net::TcpStream;

use super::{
    error::parse::ParseError,
    request::Request, response::Response, uri::authority::Authority,
};
use crate::dns::resolver::{self, Resolver};

pub struct Client {}

impl Client {
    pub async fn perform<R>(request: Request) -> Result<Response, Box<dyn Error + Send + Sync>>
    where
        R: Resolver,
    {
        let mut stream: TcpStream;

        match request.parts.url.authority {
            Authority::Domain { ref host, port } => {
                let hosts = resolver::lookup_a::<R>(host).await?;
                let host = hosts.get(0).unwrap();
                stream = TcpStream::connect((host.clone(), port as u16)).await?;
            }
            Authority::IPv4 { ip, port } => {
                stream = TcpStream::connect((ip, port as u16)).await?;
            }
            Authority::IPv6 { ip, port } => {
                stream = TcpStream::connect((ip, port as u16)).await?;
            }
            _ => {
                return Err(ParseError::InvalidURI.into());
            }
        }

        Ok(request.call(&mut stream).await?)
    }
}

#[cfg(test)]
mod tests {
    use crate::http::{builder::Builder, header::{HeaderKind, HeaderMap}, method::Method, mimetype::MimeType, statuscode::StatusCode};

    use self::resolver::Google;

    use super::*;

    #[tokio::test]
    async fn test_client() {
        let mut headers = HeaderMap::default();
        headers
            .put(
                "accept",
                HeaderKind::Accept(Some(vec![MimeType {
                    kind: "*".to_string(),
                    sub: "*".to_string(),
                    param: None,
                }])),
            )
            .unwrap();

        let request = Builder::new()
            .method(Method::GET)
            .url("http://httpbin.org/robots.txt")
            .headers(headers)
            .build();
        let resp = Client::perform::<Google>(request).await.unwrap();
        assert_eq!(resp.status, StatusCode::Ok);
    }
}
