use std::{net::Ipv6Addr, str::FromStr};

use image::Rgba;

const IP_FORMAT: &str = "2a06:a003:d040:SXXX:YYY:RR:GG:BB";

pub fn pixel_to_addr(px: (u32, u32, Rgba<u8>), size: usize) -> Ipv6Addr {
    let x = px.0;
    let y = px.1;

    let a = px.2[3];
    let mut r = 0xFF;
    let mut g = 0xFF;
    let mut b = 0xFF;

    if a != 0 {
        r = px.2[0];
        g = px.2[1];
        b = px.2[2];
    }

    if x > 512 || y > 512 {
        panic!("Pixel out of bounds");
    }

    let addr_str = IP_FORMAT
        .replace('S', &format!("{size:x}"))
        .replace("XXX", &format!("{x:03x}"))
        .replace("YYY", &format!("{y:03x}"))
        .replace("RR", &format!("{r:02x}"))
        .replace("GG", &format!("{g:02x}"))
        .replace("BB", &format!("{b:02x}"));

    Ipv6Addr::from_str(&addr_str).expect("formatted address should convert")
}
