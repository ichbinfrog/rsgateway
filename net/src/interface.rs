use std::{
    array::TryFromSliceError,
    ffi::CStr,
    mem,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use libc::{ifaddrs, sockaddr_in, sockaddr_in6, sockaddr_ll, AF_INET, AF_INET6, AF_PACKET};

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

    address: Option<IpAddr>,
    netmask: Option<IpAddr>,
    broadcast: Option<IpAddr>,
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

fn if_addr_to_mac_addr(interface: &libc::ifaddrs) -> Result<Option<MacAddr>, Error> {
    let socket_addr = interface.ifa_addr as *mut sockaddr_ll;
    let mac_array = unsafe { (*socket_addr).sll_addr };
    let addr_len = std::cmp::min(
        unsafe { (*socket_addr).sll_halen } as usize,
        mac_array.len(),
    );
    let mac_slice = unsafe { std::slice::from_raw_parts(mac_array.as_ptr(), addr_len) };
    if mac_slice.len() != 6 {
        return Ok(None);
    }

    Ok(Some(MacAddr(mac_slice.try_into()?)))
}

fn sockaddr_in_to_ipv6(addr: *mut libc::sockaddr) -> Option<IpAddr> {
    if addr.is_null() {
        return None;
    }

    let addr = addr as *mut sockaddr_in6;
    Some(IpAddr::V6(Ipv6Addr::from(
        unsafe { (*addr).sin6_addr }.s6_addr,
    )))
}

fn sockaddr_in_to_ipv4(addr: *mut libc::sockaddr) -> Option<IpAddr> {
    if addr.is_null() {
        return None;
    }

    let addr = addr as *mut sockaddr_in;
    Some(IpAddr::V4(Ipv4Addr::from(
        unsafe { (*addr).sin_addr }.s_addr,
    )))
}

pub fn interfaces() -> Result<Vec<NetworkInterface>, Error> {
    let mut res: Vec<NetworkInterface> = Vec::new();
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
                    AF_INET => res.push(NetworkInterface {
                        name: if_name.to_string(),
                        index: if_index,
                        address: None,
                        mac: if_addr_to_mac_addr(&interface)?,
                        netmask: sockaddr_in_to_ipv4(interface.ifa_addr),
                        broadcast: sockaddr_in_to_ipv4(interface.ifa_ifu),
                    }),
                    AF_INET6 => res.push(NetworkInterface {
                        name: if_name.to_string(),
                        index: if_index,
                        address: None,
                        mac: if_addr_to_mac_addr(&interface)?,
                        netmask: sockaddr_in_to_ipv6(interface.ifa_addr),
                        broadcast: sockaddr_in_to_ipv6(interface.ifa_ifu),
                    }),
                    AF_PACKET => res.push(NetworkInterface {
                        name: if_name.to_string(),
                        index: if_index,
                        address: None,
                        mac: if_addr_to_mac_addr(&interface)?,
                        netmask: None,
                        broadcast: None,
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
    use super::*;

    #[test]
    fn test_list_interfaces() {
        println!("{:?}", interfaces());
    }
}
