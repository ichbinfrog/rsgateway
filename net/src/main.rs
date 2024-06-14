use bitarray::buffer::Buffer;
use bitarray::serialize::Deserialize;
use net::{ip, udp};
use tun_tap::Iface;

fn main() {
    let iface = Iface::new("tun0", tun_tap::Mode::Tun).unwrap();
    let name = iface.name();

    loop {
        // Configure the device ‒ set IP address on it, bring it up.
        let mut raw = vec![0; 128]; // MTU + 4 for the header
        iface.recv(&mut raw).unwrap();
        
        let mut buf = Buffer::from_vec(512, raw);
        buf.reset();

        let (ip_p, ip_l) = ip::Packet::deserialize(&mut buf).unwrap();

        if ip_l != 0 {
            let mut data = Buffer::from_vec(512, ip_p.data);
            data.reset();
            let (udp_p, udp_l) = udp::Datagram::deserialize(&mut data).unwrap();
            println!("{:?}", udp_p);   
        }
    }
}
