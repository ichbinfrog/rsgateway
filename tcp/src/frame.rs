// A TCP Header as defined in [RFC-9293](https://datatracker.ietf.org/doc/html/rfc9293#section-3.1)
#[derive(Debug)]
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
