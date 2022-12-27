use std::net::{Ipv6Addr, SocketAddrV6};

use socket2::{Domain, Protocol, Socket, Type};

const ICMP_PACKET: [u8; 8] = [0x45, 0x00, 0x00, 0x54, 0xee, 0x96, 0x40, 0x00];

pub fn ping_v6(addr: Ipv6Addr) {
    Socket::new(Domain::IPV6, Type::RAW, Some(Protocol::ICMPV6))
        .expect("should be able to create a raw ICMPV6 socket")
        .send_to(&ICMP_PACKET, &SocketAddrV6::new(addr, 0, 0, 0).into())
        .expect("should be able to send ICMPV6 packet");
}
