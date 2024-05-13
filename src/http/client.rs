use std::{
    collections::HashMap,
    error::Error,
    io::{Read, Write},
    net::TcpStream,
};

use super::{
    error::parse::ParseError,
    header::HeaderMap,
    method::Method,
    request::{Parts, Request, Standard},
    response::Response,
    uri::{authority::Authority, path::Path, url::Url},
    version::Version,
};
use crate::dns::resolver::{self, Resolver};

pub struct Client {}

impl Client {
    pub fn get<R>(
        url: Url,
        method: Method,
        headers: HeaderMap,
    ) -> Result<Response<TcpStream>, Box<dyn Error>>
    where
        R: Resolver,
    {
        let mut stream: TcpStream;

        match url.authority {
            Some(Authority::Domain { host, port }) => {
                let hosts = resolver::lookup_a::<R>(&host)?;
                let host = hosts.get(0).unwrap();
                stream = TcpStream::connect((host.clone(), port.unwrap() as u16))?;
            }
            Some(Authority::IPv4 { ip, port }) => {
                stream = TcpStream::connect((ip, port.unwrap() as u16))?;
            }
            Some(Authority::IPv6 { ip, port }) => {
                stream = TcpStream::connect((ip, port.unwrap() as u16))?;
            }
            None => {
                return Err(ParseError::InvalidURI.into());
            }
        }
        let request: Request<TcpStream> = Request {
            parts: Parts {
                method,
                headers,
                standard: Standard {
                    name: "HTTP".to_string(),
                    version: Version {
                        major: 1,
                        minor: Some(1),
                        patch: None,
                    },
                },
                path: Path {
                    raw_path: url.path.unwrap().raw_path,
                    ..Default::default()
                },
            },
            body: None,
            stream: None,
        };
        println!("{:?}", request);

        let resp = request.call(&mut stream)?;
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use self::resolver::Google;

    use super::*;

    #[test]
    fn test_client() {
        let url = Url {
            scheme: "http".to_string(),
            authority: Some(Authority::Domain {
                host: "httpbin.org".to_string(),
                port: Some(80),
            }),
            path: Some(Path {
                raw_path: "/status/200".to_string(),
                raw_fragment: None,
                query: None,
            }),
        };

        let mut headers = HeaderMap::default();
        headers
            .raw
            .insert("host".to_string(), "httpbin.org".to_string());
        headers.raw.insert("accept".to_string(), "*/*".to_string());

        let resp = Client::get::<Google>(url, Method::GET, headers).unwrap();
        println!("{:?}", resp);
    }
}
