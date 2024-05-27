use std::net::Ipv4Addr;

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
    use std::str::FromStr;

    use super::*;
    use crate::{
        builder::Builder, header::HeaderMap, method::Method, statuscode::StatusCode, uri::url::Url,
    };
    use dns::resolver::DNS_IP_LOCAL;
    use rstest::*;

    #[ignore]
    #[tokio::test]
    async fn test_client() {
        let request = Builder::new()
            .method(Method::GET)
            .url(Url::from_str("http://httpbin.org/robots.txt").unwrap())
            .headers(HeaderMap::default())
            .build();
        let resp = Client::perform(request, DNS_IP_LOCAL).await.unwrap();
        assert_eq!(resp.status, StatusCode::Ok);
    }

    #[rstest]
    #[case::is_ignored(
        "basic-auth/user/correct-password",
        "user",
        "correct-password",
        StatusCode::Ok
    )]
    #[case::is_ignored(
        "basic-auth/user/correct-password",
        "user",
        "bad-password",
        StatusCode::Unauthorized
    )]
    #[case::is_ignored(
        "hidden-basic-auth/user/correct-password",
        "user",
        "correct-password",
        StatusCode::Ok
    )]
    #[case::is_ignored(
        "hidden-basic-auth/user/correct-password",
        "user",
        "bad-password",
        StatusCode::NotFound
    )]
    #[ignore]
    #[tokio::test]
    async fn test_client_authorization(
        #[case] endpoint: &str,
        #[case] user: &str,
        #[case] password: &str,
        #[case] expected: StatusCode,
    ) {
        let mut url = "http://httpbin.org/".to_string();
        url.push_str(endpoint);
        let url = Url::from_str(&url).unwrap();

        let request = Builder::new()
            .method(Method::GET)
            .url(url)
            .headers(HeaderMap::default())
            .basic_auth(user, password)
            .build();
        let resp = Client::perform(request, DNS_IP_LOCAL).await.unwrap();
        assert_eq!(resp.status, expected);
    }
}
