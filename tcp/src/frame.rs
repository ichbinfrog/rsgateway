use bitarray::{buffer, serialize};
use bitarray_derive::{Deserialize, Serialize};

// A TCP Header as defined in [RFC-9293](https://datatracker.ietf.org/doc/html/rfc9293#section-3.1)
#[derive(Serialize, Deserialize, Default, Debug)]
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
}

#[cfg(test)]
pub mod tests {
    use bitarray::{
        buffer,
        serialize::{Deserialize, Serialize},
    };

    use super::Header;

    #[test]
    fn test_serialization() {
        let header = Header {
            cwr: true,
            ece: true,
            ack: true,
            checksum: 10,
            ..Default::default()
        };
        let mut buf = buffer::Buffer::new(258);
        println!("{:?}", header.write(&mut buf));
        println!("{:?}", buf);
        buf.reset();

        let res = Header::deserialize(&mut buf);
        println!("{:?}", res);
    }
}
