use bitarray::buffer::Buffer;
use net::ip;
use tun_tap::Iface;
use bitarray::serialize::Deserialize;

fn main() {
    let mut iface = Iface::new("tun0", tun_tap::Mode::Tun).unwrap();
    let name = iface.name();

    loop {
        // Configure the device ‒ set IP address on it, bring it up.
        let mut raw = vec![0; 128]; // MTU + 4 for the header
        iface.recv(&mut raw).unwrap();
        let mut buf = Buffer::from_vec(512, raw);
        let (res, m) = ip::Packet::deserialize(&mut buf).unwrap();
        println!("{:?}", res);
    }
}
