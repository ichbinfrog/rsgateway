use std::{
    array::TryFromSliceError, collections::HashMap, ffi::CStr, mem, net::{IpAddr, Ipv4Addr, Ipv6Addr}, ptr::null,
};

use libc::{if_freenameindex, if_nameindex, ifaddrs, sockaddr_in, sockaddr_in6, sockaddr_ll, AF_INET, AF_INET6, AF_PACKET};

#[derive(Debug)]
pub enum Error {
    Utf8Error(std::str::Utf8Error),
    GetIfAddrsError(i32),
    TryFromSliceError(TryFromSliceError),
}

impl From<std::str::Utf8Error> for Error {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::Utf8Error(value)
    }
}

impl From<TryFromSliceError> for Error {
    fn from(value: TryFromSliceError) -> Self {
        Self::TryFromSliceError(value)
    }
}

#[derive(Debug)]
pub struct IPNetwork {
    addr: Ipv4Addr,
    prefix: u8,
}

#[derive(Debug)]
pub struct MacAddr([u8; 6]);

#[derive(Debug)]
pub struct NetworkInterface {
    name: String,
    index: u32,
    mac: Option<MacAddr>,
}

pub struct IfNameIndexIterator {
    base: *mut libc::if_nameindex,
    next: *mut libc::if_nameindex,
}

impl Iterator for IfNameIndexIterator {
    type Item = libc::if_nameindex;

    fn next(&mut self) -> Option<Self::Item> {
        match unsafe { self.next.as_ref() } {
            Some(if_ni) => {
                self.next = unsafe { self.next.add(1) };
                Some(if_ni.to_owned())
            }
            None => None,
        }
    }
}

pub struct IfAddrIterator {
    base: *mut libc::ifaddrs,
    next: *mut libc::ifaddrs,
}

impl Iterator for IfAddrIterator {
    type Item = libc::ifaddrs;

    fn next(&mut self) -> Option<Self::Item> {
        match unsafe { self.next.as_ref() } {
            Some(ifaddrs) => {
                self.next = ifaddrs.ifa_next;
                Some(ifaddrs.to_owned())
            }
            None => None,
        }
    }
}

impl Drop for IfAddrIterator {
    fn drop(&mut self) {
        unsafe { libc::freeifaddrs(self.base) }
    }
}

impl TryFrom<&libc::ifaddrs> for MacAddr {
    type Error = Error;

    fn try_from(interface: &libc::ifaddrs) -> Result<Self, Self::Error> {
        let socket_addr = interface.ifa_addr as *mut sockaddr_ll;
        let mac_array = unsafe { (*socket_addr).sll_addr };
        let addr_len = std::cmp::min(
            unsafe { (*socket_addr).sll_halen } as usize,
            mac_array.len(),
        );
        let mac_slice = unsafe { std::slice::from_raw_parts(mac_array.as_ptr(), addr_len) };
        Ok(MacAddr(mac_slice.try_into()?))
    }
}

fn sockaddr_in_to_ipv6(interface: &libc::ifaddrs) -> Option<Ipv6Addr> {
    let addr = interface.ifa_netmask;
    if addr.is_null() {
        return None
    }

    let addr = addr as *mut sockaddr_in6;
    Some(Ipv6Addr::from( unsafe { (*addr).sin6_addr }.s6_addr ))
}

fn sockaddr_in_to_ipv4(interface: &libc::ifaddrs) -> Option<Ipv4Addr> {
    let addr = interface.ifa_netmask;
    if addr.is_null() {
        return None
    }

    let addr = addr as *mut sockaddr_in;
    Some(Ipv4Addr::from( unsafe { (*addr).sin_addr }.s_addr ))
}

pub fn interfaces() -> Result<Vec<NetworkInterface>, Error> {
    let mut res: Vec<NetworkInterface> = Vec::new();
    // let mut lookup = HashMap::<String, NetworkInterface>::new();

    // // Fetch interfaces potentially without any addresses
    // let if_ni = unsafe { libc::if_nameindex() };
    // let indexes = IfNameIndexIterator {
    //     base: if_ni,
    //     next: if_ni,
    // };
    // for index in indexes {
    //     if index.if_index == 0 && index.if_name.is_null() {
    //         break;
    //     }
    //     let name = unsafe { CStr::from_ptr(index.if_name) }
    //         .to_str()?
    //         .to_string();
    //     let interface = NetworkInterface {
    //         name: name.clone(),
    //         index: index.if_index,
    //     };
    //     lookup.insert(name, interface);
    // }
    // println!("{:?}", lookup);

    // Enrich hashmap with addresses if possible
    let mut addr: mem::MaybeUninit<*mut ifaddrs> = mem::MaybeUninit::<*mut libc::ifaddrs>::uninit();
    match unsafe { libc::getifaddrs(addr.as_mut_ptr()) } {
        0 => {
            let interfaces = IfAddrIterator {
                base: unsafe { addr.assume_init() },
                next: unsafe { addr.assume_init() },
            };

            for interface in interfaces {
                let if_socket = interface.ifa_addr;
                if if_socket.is_null() {
                    break;
                }

                let if_name = unsafe { CStr::from_ptr(interface.ifa_name) }.to_str()?;
                let if_index =
                    unsafe { libc::if_nametoindex(interface.ifa_name as *const libc::c_char) };
                match unsafe { (*if_socket).sa_family } as i32 {
                    AF_INET => {
                        let sockaddr = if_socket  as *mut sockaddr_in;
                        let if_addr = unsafe { (*sockaddr).sin_addr };
                        let netmask = sockaddr_in_to_ipv4(&interface);
                        // let if_netmask = (*if_socket).sin_zero;
                    }
                    AF_INET6 => {
                        println!("AF_INET6 {:?}", if_name);
                        let if_addr = if_socket  as *mut sockaddr_in6;
                    }
                    AF_PACKET => res.push(NetworkInterface {
                        name: if_name.to_string(),
                        index: if_index,
                        mac: Some(MacAddr::try_from(&interface)?),
                    }),
                    x => unimplemented!("socket family {} not implemented", x),
                    }
            }
        }
        err_no => return Err(Error::GetIfAddrsError(err_no)),
    }

    Ok(res)
}

#[cfg(test)]
pub mod tests {
    use crate::interfaces;

    #[test]
    fn test_list_interfaces() {
        println!("{:?}", interfaces());
    }
}
