use std::error::Error;

use tokio::net::TcpStream;

use super::{
    builder::Builder, error::parse::ParseError, header::HeaderMap, request::Request,
    response::Response, uri::authority::Authority,
};
use crate::dns::resolver::{self, Resolver};

pub struct Client {}

impl Client {
    pub async fn get<R>(
        url: String,
        headers: HeaderMap,
    ) -> Result<Response, Box<dyn Error + Send + Sync>>
    where
        R: Resolver,
    {
        let mut stream: TcpStream;
        let request: Request = Builder::new().get(url).headers(headers).build();

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
    use crate::http::{header::HeaderKind, mimetype::MimeType, statuscode::StatusCode};

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

        let mut resp = Client::get::<Google>("http://httpbin.org/robots.txt".to_string(), headers)
            .await
            .unwrap();
        assert_eq!(resp.status, StatusCode::Ok);
        // assert!(resp.read_body().await.unwrap() > 0);
    }
}
