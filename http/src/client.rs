use std::{error::Error, net::Ipv4Addr};

use tokio::net::TcpStream;

use super::{
    error::parse::ParseError, request::Request, response::Response, uri::authority::Authority,
};
use dns::resolver::Resolver;

pub struct Client {}

impl Client {
    pub async fn perform(
        request: Request,
        dns_ip: &[Ipv4Addr],
    ) -> Result<Response, Box<dyn Error + Send + Sync>> {
        let mut stream: TcpStream;
        let resolver = Resolver::new();

        match request.parts.url.authority {
            Authority::Domain { ref host, port } => {
                let hosts: Vec<Ipv4Addr> = resolver.lookup_a(host, dns_ip).await?;
                let host = hosts.first().unwrap();
                stream = TcpStream::connect((*host, port as u16)).await?;
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

        request.call(&mut stream).await
    }
}

#[cfg(test)]
mod tests {
    use dns::resolver::DNS_IP_LOCAL;
    use crate::{
        builder::Builder,
        header::{HeaderKind, HeaderMap},
        method::Method,
        mimetype::MimeType,
        statuscode::StatusCode,
    };

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
        let resp = Client::perform(request, DNS_IP_LOCAL).await.unwrap();
        assert_eq!(resp.status, StatusCode::Ok);
    }
}
