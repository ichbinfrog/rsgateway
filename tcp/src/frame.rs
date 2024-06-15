use bitarray::buffer::{Buffer, Error};

use bitarray::decode::Decoder;
use bitarray::encode::Encoder;
use bitarray_derive::Encode;

// A TCP Header as defined in [RFC-9293](https://datatracker.ietf.org/doc/html/rfc9293#section-3.1)
#[derive(Default, Encode, PartialEq, Debug)]
pub struct Header {
    src: u16,
    dst: u16,
    seq_num: u32,
    ack_num: u32,
    offset: u8,
    reserved: u8,

    cwr: bool,
    ece: bool,
    urg: bool,
    ack: bool,
    psh: bool,
    rst: bool,
    syn: bool,
    fin: bool,

    window: u16,
    checksum: u16,
    urgent: u16,

    options: Option<TcpOptions>,
}

impl Decoder for Header {
    fn decode(buf: &mut Buffer) -> Result<(Self, usize), Error> {
        let mut i = 0;
        let (src, n) = u16::decode(buf)?;
        i += n;
        let (dst, n) = u16::decode(buf)?;
        i += n;
        let (seq_num, n) = u32::decode(buf)?;
        i += n;
        let (ack_num, n) = u32::decode(buf)?;
        i += n;
        let (offset, n) = u8::decode(buf)?;
        i += n;
        let (reserved, n) = u8::decode(buf)?;
        i += n;
        let (cwr, n) = bool::decode(buf)?;
        i += n;
        let (ece, n) = bool::decode(buf)?;
        i += n;
        let (urg, n) = bool::decode(buf)?;
        i += n;
        let (ack, n) = bool::decode(buf)?;
        i += n;
        let (psh, n) = bool::decode(buf)?;
        i += n;
        let (rst, n) = bool::decode(buf)?;
        i += n;
        let (syn, n) = bool::decode(buf)?;
        i += n;
        let (fin, n) = bool::decode(buf)?;
        i += n;
        let (window, n) = u16::decode(buf)?;
        i += n;
        let (checksum, n) = u16::decode(buf)?;
        i += n;
        let (urgent, n) = u16::decode(buf)?;
        i += n;

        let mut options: Option<TcpOptions> = None;
        if offset > 5 {
            let (opts, n) = TcpOptions::decode(buf)?;
            i += n;
            options = Some(opts);
        }

        Ok((
            Self {
                src,
                dst,
                seq_num,
                ack_num,
                offset,
                reserved,
                cwr,
                ece,
                urg,
                ack,
                psh,
                rst,
                syn,
                fin,
                window,
                checksum,
                urgent,
                options,
            },
            i,
        ))
    }
}

#[derive(Debug, Default, PartialEq)]
pub enum TcpOption {
    #[default]
    End,
    NoOp,
    MSS {
        length: u8,
        mss: u16,
    },
}

impl Encoder for TcpOption {
    fn encode(&self, buf: &mut Buffer) -> Result<usize, Error> {
        match self {
            Self::End => 0u8.encode(buf),
            Self::NoOp => 1u8.encode(buf),
            Self::MSS { length, mss } => {
                let mut n = 0;
                n += 2u8.encode(buf)?;
                n += length.encode(buf)?;
                n += mss.encode(buf)?;
                Ok(n)
            }
        }
    }
}

#[derive(Debug, Default, PartialEq)]
struct TcpOptions(Vec<TcpOption>);

impl Encoder for TcpOptions {
    fn encode(&self, buf: &mut Buffer) -> Result<usize, Error> {
        let mut n = 0;
        for opt in self.0.iter() {
            n += opt.encode(buf)?;
        }
        Ok(n)
    }
}

impl Decoder for TcpOptions {
    fn decode(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        let mut n = 0;
        let mut options: Vec<TcpOption> = Vec::new();

        loop {
            let (op_code, i) = u8::decode(buf)?;
            n += i;

            match op_code {
                0 => {
                    options.push(TcpOption::End);
                    return Ok((TcpOptions(options), n));
                }
                1 => {
                    options.push(TcpOption::NoOp);
                }
                2 => {
                    let (length, i) = u8::decode(buf)?;
                    n += i;
                    let (mss, i) = u16::decode(buf)?;
                    n += i;
                    options.push(TcpOption::MSS { length, mss });
                }
                _ => unimplemented!("tcp_option code unimplemented"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{TcpOption, TcpOptions};

    #[test]
    fn test_serialization() {
        let header = Header {
            offset: 7,
            cwr: true,
            ece: true,
            ack: true,
            checksum: 10,
            options: Some(TcpOptions(vec![
                TcpOption::MSS {
                    length: 5,
                    mss: 352,
                },
                TcpOption::End,
            ])),
            ..Default::default()
        };
        let mut buf = Buffer::new(258);
        let n = header.encode(&mut buf).unwrap();
        buf.reset();

        let (res, m) = Header::decode(&mut buf).unwrap();
        assert_eq!(n, m);
        assert_eq!(res, header)
    }
}
