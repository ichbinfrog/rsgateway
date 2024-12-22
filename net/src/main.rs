use bitarray::buffer::Buffer;
use bitarray::decode::Decoder;
use net::{ip, udp};
use tun_tap::Iface;

fn main() {
    let iface = Iface::new("tun0", tun_tap::Mode::Tun).unwrap();
    let name = iface.name();

    loop {
        // Configure the device ‒ set IP address on it, bring it up.
        let mut raw = vec![0; 128]; // MTU + 4 for the header
        iface.recv(&mut raw).unwrap();

        let mut buf = Buffer::from_vec(raw);
        buf.reset();

        let (ip_p, ip_l) = ip::Packet::decode(&mut buf).unwrap();

        if ip_l != 0 {
            let mut data = Buffer::from_vec(ip_p.data);
            data.reset();
            let (udp_p, udp_l) = udp::Datagram::decode(&mut data).unwrap();
            println!("{:?}", udp_p);
        }
    }
}
