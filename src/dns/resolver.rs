use tokio::net::UdpSocket;

use super::{
    buffer::Header,
    packet::{Packet, Record},
    question::{Question, QuestionClass, QuestionKind},
};
use std::{
    error::Error,
    net::{Ipv4Addr, Ipv6Addr},
};

pub trait Resolver {
    fn server() -> Vec<Ipv4Addr>;
}

pub async fn lookup<R>(
    domain: &str,
    qtype: QuestionKind,
) -> Result<Packet, Box<dyn Error + Send + Sync>>
where
    R: Resolver,
{
    let packet = Packet {
        header: Header {
            id: 30000,
            query_count: 1,
            recursion_desired: true,
            ..Default::default()
        },
        questions: Some(vec![Question {
            name: domain.to_string(),
            kind: qtype,
            class: QuestionClass::IN,
        }]),
        answers: None,
        authorities: None,
        resources: None,
    };

    let mut socket = UdpSocket::bind(("0.0.0.0", 0)).await?;

    let servers = R::server();

    let server = fastrand::choice(servers).unwrap();
    Ok(packet.lookup(&mut socket, &server.to_string()).await?)
}

pub async fn lookup_a<R>(domain: &str) -> Result<Vec<Ipv4Addr>, Box<dyn Error + Send + Sync>>
where
    R: Resolver,
{
    let packet = lookup::<R>(domain, QuestionKind::A).await?;
    Ok(packet
        .answers
        .iter()
        .flatten()
        .filter_map(|record| match record {
            Record::A { addr, .. } => Some(addr.clone()),
            _ => None,
        })
        .collect())
}

pub async fn lookup_aaaa<R>(domain: &str) -> Result<Vec<Ipv6Addr>, Box<dyn Error + Send + Sync>>
where
    R: Resolver,
{
    let packet = lookup::<R>(domain, QuestionKind::A).await?;
    Ok(packet
        .answers
        .iter()
        .flatten()
        .filter_map(|record| match record {
            Record::AAAA { addr, .. } => Some(addr.clone()),
            _ => None,
        })
        .collect())
}

pub async fn lookup_ns<R>(domain: &str) -> Result<Vec<String>, Box<dyn Error + Send + Sync>>
where
    R: Resolver,
{
    let packet = lookup::<R>(domain, QuestionKind::A).await?;
    Ok(packet
        .answers
        .iter()
        .flatten()
        .filter_map(|record| match record {
            Record::NS { host, .. } => Some(host.clone()),
            _ => None,
        })
        .collect())
}

pub async fn lookup_cname<R>(domain: &str) -> Result<Vec<String>, Box<dyn Error + Send + Sync>>
where
    R: Resolver,
{
    let packet = lookup::<R>(domain, QuestionKind::A).await?;
    Ok(packet
        .authorities
        .iter()
        .flatten()
        .filter_map(|record| match record {
            Record::CNAME { host, .. } => Some(host.clone()),
            _ => None,
        })
        .collect())
}

pub struct Local {}

impl Resolver for Local {
    fn server() -> Vec<Ipv4Addr> {
        vec![Ipv4Addr::new(192, 168, 1, 254)]
    }
}

pub struct Google {}

impl Resolver for Google {
    fn server() -> Vec<Ipv4Addr> {
        vec![Ipv4Addr::new(8, 8, 8, 8), Ipv4Addr::new(8, 8, 4, 4)]
    }
}

pub struct CloudFlare {}
impl Resolver for CloudFlare {
    fn server() -> Vec<Ipv4Addr> {
        vec![Ipv4Addr::new(1, 1, 1, 1)]
    }
}
