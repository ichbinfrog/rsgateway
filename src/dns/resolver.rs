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

pub const DNS_IP_LOCAL: &[Ipv4Addr] = &[Ipv4Addr::new(192, 168, 1, 254)];
pub const DNS_IP_CLOUDFLARE: &[Ipv4Addr] = &[Ipv4Addr::new(1, 1, 1, 1)];
pub const DNS_IP_GOOGLE: &[Ipv4Addr] = &[Ipv4Addr::new(8, 8, 8, 8), Ipv4Addr::new(8, 8, 4, 4)];

pub struct Resolver {}

impl Resolver {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn lookup(
        self,
        domain: &str,
        qtype: QuestionKind,
        resolver: &[Ipv4Addr],
    ) -> Result<Packet, Box<dyn Error + Send + Sync>> {
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
        let server = fastrand::choice(resolver).unwrap();
        packet.lookup(&mut socket, &server.to_string()).await
    }

    pub async fn lookup_a(
        self,
        domain: &str,
        resolver: &[Ipv4Addr],
    ) -> Result<Vec<Ipv4Addr>, Box<dyn Error + Send + Sync>> {
        let packet = self.lookup(domain, QuestionKind::A, resolver).await?;
        Ok(packet
            .answers
            .iter()
            .flatten()
            .filter_map(|record| match record {
                Record::A { addr, .. } => Some(*addr),
                _ => None,
            })
            .collect())
    }

    pub async fn lookup_aaaa(
        self,
        domain: &str,
        resolver: &[Ipv4Addr],
    ) -> Result<Vec<Ipv6Addr>, Box<dyn Error + Send + Sync>> {
        let packet = self.lookup(domain, QuestionKind::A, resolver).await?;
        Ok(packet
            .answers
            .iter()
            .flatten()
            .filter_map(|record| match record {
                Record::AAAA { addr, .. } => Some(*addr),
                _ => None,
            })
            .collect())
    }

    pub async fn lookup_ns(
        self,
        domain: &str,
        resolver: &[Ipv4Addr],
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let packet = self.lookup(domain, QuestionKind::A, resolver).await?;
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

    pub async fn lookup_cname(
        self,
        domain: &str,
        resolver: &[Ipv4Addr],
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let packet = self.lookup(domain, QuestionKind::A, resolver).await?;
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
}
