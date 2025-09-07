use std::net::Ipv6Addr;

const ICMP_PACKET: [u8; 8] = [8, 0, 0, 0, 0, 0, 0, 0];

pub async fn ping_v6(addr: Ipv6Addr) {
    surge_ping::ping(addr.into(), &ICMP_PACKET).await.ok();
}
