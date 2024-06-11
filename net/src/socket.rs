use bitflags::bitflags;
use libc::sockaddr_in;
use num_traits::{PrimInt, Signed};
use std::{
    io, mem,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, ToSocketAddrs},
    ops::BitOr,
};

#[derive(Debug, Clone, Copy)]
pub enum Domain {
    Unix = libc::AF_UNIX as isize,
    Inet = libc::AF_INET as isize,
    Packet = libc::AF_BLUETOOTH as isize,
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct Semantic: i32 {
        const Stream = libc::SOCK_STREAM;
        const Dgram = libc::SOCK_DGRAM;
        const Raw = libc::SOCK_RAW;
        const NonBlock = libc::SOCK_NONBLOCK;
        const CloExec = libc::SOCK_CLOEXEC;
    }
}

#[derive(Debug)]
pub struct Socket {
    domain: Domain,
    semantic: Semantic,
    protocol: libc::c_int,

    fd: libc::c_int,
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}

fn cvt<T: PrimInt + Signed>(t: T) -> Result<T, std::io::Error> {
    if t.is_negative() {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(t)
    }
}

impl Socket {
    pub fn new(
        domain: Domain,
        semantic: Semantic,
        protocol: libc::c_int,
    ) -> Result<Self, std::io::Error> {
        let fd = cvt(unsafe { libc::socket(domain as i32, semantic.bits(), protocol) })?;
        Ok(Self {
            domain,
            semantic,
            protocol,
            fd,
        })
    }

    pub fn bind(&self, addr: SocketAddr) -> Result<u16, std::io::Error> {
        match addr {
            SocketAddr::V4(addr) => {
                let mut addr_in = libc::sockaddr_in {
                    sin_family: self.domain as u16,
                    sin_port: addr.port(),
                    sin_addr: libc::in_addr {
                        s_addr: libc::INADDR_ANY,
                    },
                    sin_zero: [0u8; 8],
                };
                let _ = cvt(unsafe {
                    libc::bind(
                        self.fd,
                        (&mut addr_in as *const libc::sockaddr_in).cast(),
                        mem::size_of::<libc::sockaddr_in>().try_into().unwrap(),
                    )
                })?;
                let mut bound = std::mem::MaybeUninit::<libc::sockaddr_in>::uninit();
                let mut bound_len = 0u32;

                let _ = cvt(unsafe {
                    libc::getsockname(self.fd, bound.as_mut_ptr().cast(), &mut bound_len)
                })?;
                return Ok(unsafe { bound.assume_init().sin_port });
            }
            SocketAddr::V6(_) => unimplemented!("bind not implemented for ipv6 yet"),
        }
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), std::io::Error> {
        let mut storage: libc::sockaddr_storage = unsafe { mem::zeroed() };
        let mut addrlen = mem::size_of_val(&storage) as libc::socklen_t;

        let n = cvt(unsafe {
            libc::recvfrom(
                self.fd, 
                buf.as_mut_ptr() as *mut libc::c_void, 
                buf.len(), 
                0, 
                std::ptr::addr_of_mut!(storage) as *mut _,
                &mut addrlen)
        })?;
        Ok((n as usize, ))

    }
}

#[derive(Debug)]
pub struct UdpSocket {
    inner: Socket,
}

#[repr(C)]
pub union SocketAddrCRepr {
    v4: libc::sockaddr_in,
    v6: libc::sockaddr_in6,
}

impl SocketAddrCRepr {
    pub fn as_ptr(&self) -> *const libc::sockaddr {
        self as *const _ as *const libc::sockaddr
    }
}

impl UdpSocket {
    pub fn bind(addr: SocketAddr) -> Result<Self, std::io::Error> {
        let socket = Socket::new(Domain::Inet, Semantic::Dgram | Semantic::CloExec, 0)?;
        match addr {
            SocketAddr::V4(addr) => {
                let c_addr = SocketAddrCRepr {
                    v4: {
                        libc::sockaddr_in {
                            sin_family: libc::AF_INET as libc::sa_family_t,
                            sin_port: addr.port().to_be(),
                            sin_addr: libc::in_addr {
                                s_addr: u32::from_ne_bytes(addr.ip().octets()),
                            },
                            sin_zero: unsafe { mem::zeroed() },
                        }
                    },
                };

                cvt(unsafe {
                    libc::bind(
                        socket.fd,
                        c_addr.as_ptr(),
                        mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
                    )
                })?;
                return Ok(Self { inner: socket });
            }
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::net::{Ipv4Addr, SocketAddrV4};

    use super::*;

    #[test]
    fn test_new_socket() {
        let s = Socket::new(Domain::Inet, Semantic::Raw, 0).unwrap();
        let res = s.bind(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(127, 0, 0, 1),
            0,
        )));
        println!("{:?}", res);
        println!("{:?}", s.fd);
    }

    #[test]
    fn test_new_udpsocket() {
        let s = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(127, 0, 0, 1),
            0,
        )));
        println!("{:?}", s);
    }
}
