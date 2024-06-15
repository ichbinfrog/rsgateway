use bitarray::{
    buffer,
    serialize::{self, Deserialize, Serialize},
};
use bitarray_derive::Serialize;

// A TCP Header as defined in [RFC-9293](https://datatracker.ietf.org/doc/html/rfc9293#section-3.1)
#[derive(Default, Serialize, PartialEq, Debug)]
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

impl serialize::Deserialize for Header {
    fn deserialize(buf: &mut buffer::Buffer) -> Result<(Self, usize), buffer::Error> {
        let mut i = 0;
        let (src, n) = u16::deserialize(buf)?;
        i += n;
        let (dst, n) = u16::deserialize(buf)?;
        i += n;
        let (seq_num, n) = u32::deserialize(buf)?;
        i += n;
        let (ack_num, n) = u32::deserialize(buf)?;
        i += n;
        let (offset, n) = u8::deserialize(buf)?;
        i += n;
        let (reserved, n) = u8::deserialize(buf)?;
        i += n;
        let (cwr, n) = bool::deserialize(buf)?;
        i += n;
        let (ece, n) = bool::deserialize(buf)?;
        i += n;
        let (urg, n) = bool::deserialize(buf)?;
        i += n;
        let (ack, n) = bool::deserialize(buf)?;
        i += n;
        let (psh, n) = bool::deserialize(buf)?;
        i += n;
        let (rst, n) = bool::deserialize(buf)?;
        i += n;
        let (syn, n) = bool::deserialize(buf)?;
        i += n;
        let (fin, n) = bool::deserialize(buf)?;
        i += n;
        let (window, n) = u16::deserialize(buf)?;
        i += n;
        let (checksum, n) = u16::deserialize(buf)?;
        i += n;
        let (urgent, n) = u16::deserialize(buf)?;
        i += n;

        let mut options: Option<TcpOptions> = None;
        if offset > 5 {
            let (opts, n) = TcpOptions::deserialize(buf)?;
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

impl Serialize for TcpOption {
    fn serialize(&self, buf: &mut buffer::Buffer) -> Result<usize, buffer::Error> {
        match self {
            Self::End => 0u8.serialize(buf),
            Self::NoOp => 1u8.serialize(buf),
            Self::MSS { length, mss } => {
                let mut n = 0;
                n += 2u8.serialize(buf)?;
                n += length.serialize(buf)?;
                n += mss.serialize(buf)?;
                Ok(n)
            }
        }
    }
}

#[derive(Debug, Default, PartialEq)]
struct TcpOptions(Vec<TcpOption>);

impl Serialize for TcpOptions {
    fn serialize(&self, buf: &mut buffer::Buffer) -> Result<usize, buffer::Error> {
        let mut n = 0;
        for opt in self.0.iter() {
            n += opt.serialize(buf)?;
        }
        Ok(n)
    }
}

impl Deserialize for TcpOptions {
    fn deserialize(buf: &mut buffer::Buffer) -> Result<(Self, usize), buffer::Error>
    where
        Self: Sized,
    {
        let mut n = 0;
        let mut options: Vec<TcpOption> = Vec::new();

        loop {
            let (op_code, i) = u8::deserialize(buf)?;
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
                    let (length, i) = u8::deserialize(buf)?;
                    n += i;
                    let (mss, i) = u16::deserialize(buf)?;
                    n += i;
                    options.push(TcpOption::MSS { length, mss });
                }
                _ => unimplemented!("tcp_option code unimplemented"),
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use bitarray::{
        buffer,
        serialize::{Deserialize, Serialize},
    };

    use crate::frame::{TcpOption, TcpOptions};

    use super::Header;

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
        let mut buf = buffer::Buffer::new(258);
        let n = header.serialize(&mut buf).unwrap();
        buf.reset();

        let (res, m) = Header::deserialize(&mut buf).unwrap();
        assert_eq!(n, m);
        assert_eq!(res, header)
    }
}
