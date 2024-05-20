use std::{net::Ipv4Addr};

use tokio::net::TcpStream;

use super::{
    error::frame::FrameError, request::Request, response::Response, uri::authority::Authority,
};
use dns::resolver::Resolver;

pub struct Client {}

impl Client {
    pub async fn perform(request: Request, dns_ip: &[Ipv4Addr]) -> Result<Response, FrameError> {
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
                return Err(FrameError::Invalid {
                    reason: "unable to resolve authority",
                    subject: "authority",
                });
            }
        }

        request.call(&mut stream).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{builder::Builder, header::HeaderMap, method::Method, statuscode::StatusCode};
    use dns::resolver::DNS_IP_LOCAL;
    use rstest::*;

    #[tokio::test]
    async fn test_client() {
        let request = Builder::new()
            .method(Method::GET)
            .url("http://httpbin.org/robots.txt")
            .headers(HeaderMap::default())
            .build();
        let resp = Client::perform(request, DNS_IP_LOCAL).await.unwrap();
        assert_eq!(resp.status, StatusCode::Ok);
    }

    #[rstest]
    #[case(
        "basic-auth/user/correct-password",
        "user",
        "correct-password",
        StatusCode::Ok
    )]
    #[case(
        "basic-auth/user/correct-password",
        "user",
        "bad-password",
        StatusCode::Unauthorized
    )]
    #[case(
        "hidden-basic-auth/user/correct-password",
        "user",
        "correct-password",
        StatusCode::Ok
    )]
    #[case(
        "hidden-basic-auth/user/correct-password",
        "user",
        "bad-password",
        StatusCode::NotFound
    )]
    #[tokio::test]
    async fn test_client_authorization(
        #[case] endpoint: &str,
        #[case] user: &str,
        #[case] password: &str,
        #[case] expected: StatusCode,
    ) {
        let mut url = "http://httpbin.org/".to_string();
        url.push_str(endpoint);

        let request = Builder::new()
            .method(Method::GET)
            .url(&url)
            .headers(HeaderMap::default())
            .basic_auth(user, password)
            .build();
        let resp = Client::perform(request, DNS_IP_LOCAL).await.unwrap();
        assert_eq!(resp.status, expected);
    }
}
