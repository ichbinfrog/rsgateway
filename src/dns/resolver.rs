use std::{error::Error, net::Ipv4Addr};

trait Resolver {
    fn lookup(domain: String) -> Result<Ipv4Addr, Box<dyn Error>>;
}

pub struct Default {}
