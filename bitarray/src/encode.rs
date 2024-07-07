use std::net::{Ipv4Addr, Ipv6Addr};

use num_traits::Pow;

use crate::buffer::{Buffer, Error, SizedString};

pub trait Encoder {
    fn encode(&self, buf: &mut Buffer) -> Result<usize, Error>;
}
impl Encoder for Ipv4Addr {
    fn encode(&self, buf: &mut Buffer) -> Result<usize, Error> {
        self.to_bits().encode(buf)
    }
}

impl Encoder for Ipv6Addr {
    fn encode(&self, buf: &mut Buffer) -> Result<usize, Error> {
        self.to_bits().encode(buf)
    }
}

impl<T: Encoder> Encoder for Vec<T> {
    fn encode(&self, buf: &mut Buffer) -> Result<usize, Error> {
        let mut n = 0;
        for v in self.iter() {
            n += v.encode(buf)?;
        }
        Ok(n)
    }
}

impl<T: Encoder> Encoder for Option<T> {
    fn encode(&self, buf: &mut Buffer) -> Result<usize, Error> {
        match self {
            Some(x) => Ok(x.encode(buf)?),
            None => Ok(0),
        }
    }
}

impl Encoder for String {
    fn encode(&self, buf: &mut Buffer) -> Result<usize, Error> {
        let mut i: usize = 0;

        for b in self.bytes() {
            buf.push_primitive(b)?;
            i += 1;
        }

        // TODO: chunk to write as u64
        // for chunk in self.as_bytes().chunks(4) {
        // }

        Ok(i)
    }
}

impl<const N: usize> Encoder for SizedString<N> {
    fn encode(&self, buf: &mut Buffer) -> Result<usize, Error> {
        let n = self.0.len();
        if n >= 2.pow(4 * N) as usize {
            return Err(Error::Overflow {
                size: n,
                max: 2.pow(N) as usize,
            });
        }

        match N {
            1 => {
                buf.push_primitive(n as u8)?;
            }
            2 => {
                buf.push_primitive(n as u16)?;
            }
            3..=4 => {
                buf.push_primitive(n as u32)?;
            }
            5..=8 => {
                buf.push_primitive(n as u64)?;
            }
            _ => {
                unimplemented!("Serialization for string lengths over u64 has not been implemented")
            }
        }

        self.0.encode(buf)?;
        Ok(n + N)
    }
}

impl Encoder for u8 {
    fn encode(&self, buf: &mut Buffer) -> Result<usize, Error> {
        buf.push_primitive(*self)
    }
}
impl Encoder for u16 {
    fn encode(&self, buf: &mut Buffer) -> Result<usize, Error> {
        buf.push_primitive(*self)
    }
}

impl Encoder for u32 {
    fn encode(&self, buf: &mut Buffer) -> Result<usize, Error> {
        buf.push_primitive(*self)
    }
}

impl Encoder for u64 {
    fn encode(&self, buf: &mut Buffer) -> Result<usize, Error> {
        buf.push_primitive(*self)
    }
}

impl Encoder for u128 {
    fn encode(&self, buf: &mut Buffer) -> Result<usize, Error> {
        buf.push_primitive(*self)
    }
}

impl Encoder for bool {
    fn encode(&self, buf: &mut Buffer) -> Result<usize, Error> {
        buf.push_bool(*self)
    }
}
