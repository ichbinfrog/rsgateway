use crate::buffer::{Buffer, Error, SizedString};
use std::net::Ipv4Addr;

pub trait Decoder {
    fn decode(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized;
}

impl Decoder for Ipv4Addr {
    fn decode(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        let (bits, n) = buf.read_primitive::<u32, 4>()?;
        Ok((Ipv4Addr::from_bits(bits), n))
    }
}

impl<T: Decoder> Decoder for Option<T> {
    fn decode(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        let (res, n) = T::decode(buf)?;
        Ok((Some(res), n))
    }
}

impl<const N: usize> Decoder for SizedString<N> {
    fn decode(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        let (length, _) = match N {
            1 => {
                let (length, length_size) = buf.read_primitive::<u8, 1>()?;
                (length as usize, length_size)
            }
            2 => {
                let (length, length_size) = buf.read_primitive::<u16, 2>()?;
                (length as usize, length_size)
            }
            3..=4 => {
                let (length, length_size) = buf.read_primitive::<u32, 4>()?;
                (length as usize, length_size)
            }
            5..=8 => {
                let (length, length_size) = buf.read_primitive::<u64, 8>()?;
                (length as usize, length_size)
            }
            _ => {
                unimplemented!("Serialization for string lengths over u64 has not been implemented")
            }
        };

        let mut value = Vec::<u8>::with_capacity(length);
        for _ in 0..length {
            let (b, _) = buf.read_primitive::<u8, 1>()?;
            value.push(b);
        }
        let res = SizedString::<N>(String::from_utf8(value)?);
        Ok((res, N + length))
    }
}

impl Decoder for u8 {
    fn decode(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        buf.read_primitive::<u8, 1>()
    }
}

impl Decoder for u16 {
    fn decode(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        buf.read_primitive::<u16, 2>()
    }
}

impl Decoder for u32 {
    fn decode(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        buf.read_primitive::<u32, 4>()
    }
}

impl Decoder for u64 {
    fn decode(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        buf.read_primitive::<u64, 8>()
    }
}

impl Decoder for bool {
    fn decode(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        buf.read_bool()
    }
}
