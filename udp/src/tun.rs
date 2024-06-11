use std::{
    fs::{File, OpenOptions},
    io::{self, Read},
    os::fd::AsRawFd,
};

use libc::ioctl;

const IFNAMSIZ: usize = 16;
const IFF_NO_PI: i32 = 0x1000;
const IFF_TUN: i32 = 0x0001;
const IFF_TAP: i32 = 0x0002;

#[derive(Debug)]
pub struct IfReq {
    name: [u8; IFNAMSIZ],
    flags: i16,
}

pub struct TunSocket {
    fd: File,
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl TunSocket {
    const TUNSETIFF: u64 = 0x400454CA;

    fn new(name: &str) -> Result<TunSocket, Error> {
        // based on https://docs.kernel.org/networking/tuntap.html
        let fd = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/net/tun")?;

        let mut ifr = IfReq {
            name: [0; IFNAMSIZ],
            flags: (IFF_TUN | IFF_NO_PI) as _,
        };

        let name = name.as_bytes();
        ifr.name[..name.len()].copy_from_slice(name);

        // https://www.man7.org/linux/man-pages/man2/ioctl.2.html
        if unsafe { ioctl(fd.as_raw_fd(), Self::TUNSETIFF, &ifr) } < 0 {
            return Err(Error::IoError(io::Error::last_os_error()));
        }

        Ok(Self { fd })
    }
}

#[cfg(test)]
pub mod tests {
    use std::io::{Read, Write};

    use super::TunSocket;

    #[test]
    fn test_tun_device() {
        let mut socket = TunSocket::new("holla").unwrap();
        socket.fd.write(&[b'h', b'e', b'l', b'l', b'o']);

        let mut res = vec![0u8; 5];
        socket.fd.read_to_end(&mut res).unwrap();

        println!("{:?}", res);
        // let fd = OpenOptions::new().read(true).write(true).open("/dev/net/tun").unwrap();
        // println!("{:?}", fd);
    }
}
