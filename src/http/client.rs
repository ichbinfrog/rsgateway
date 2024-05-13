use std::{error::Error, net::TcpStream};

use super::{
    builder::Builder, error::parse::ParseError, header::HeaderMap, request::Request,
    response::Response, uri::authority::Authority,
};
use crate::dns::resolver::{self, Resolver};

pub struct Client {}

impl Client {
    pub fn get<R>(url: String, headers: HeaderMap) -> Result<Response<TcpStream>, Box<dyn Error>>
    where
        R: Resolver,
    {
        let mut stream: TcpStream;
        let request: Request<TcpStream> = Builder::new().get(url).headers(headers).build();

        println!("{:?}", request);
        match request.parts.url.authority {
            Authority::Domain { ref host, port } => {
                let hosts = resolver::lookup_a::<R>(host)?;
                let host = hosts.get(0).unwrap();
                stream = TcpStream::connect((host.clone(), port as u16))?;
            }
            Authority::IPv4 { ip, port } => {
                stream = TcpStream::connect((ip, port as u16))?;
            }
            Authority::IPv6 { ip, port } => {
                stream = TcpStream::connect((ip, port as u16))?;
            }
            _ => {
                return Err(ParseError::InvalidURI.into());
            }
        }
        Ok(request.call(&mut stream)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::http::{header::HeaderKind, mimetype::MimeType, statuscode::StatusCode};

    use self::resolver::Google;

    use super::*;

    #[test]
    fn test_client() {
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

        let mut resp =
            Client::get::<Google>("http://httpbin.org/robots.txt".to_string(), headers).unwrap();
        assert_eq!(resp.status, StatusCode::Ok);
        assert!(resp.read_body().unwrap() > 0);
    }
}
