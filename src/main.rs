use std::{
    net::{IpAddr, Ipv6Addr},
    str::FromStr,
    time::Duration,
};

use async_recursion::async_recursion;
use clap::Parser;
use image::{imageops::FilterType, DynamicImage, GenericImageView, Rgba};
use surge_ping::{Client, Config, PingIdentifier, PingSequence, ICMP};
use tokio::task::JoinSet;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(help = "Image file to shovel")]
    file: std::path::PathBuf,
    #[arg(short, long, help = "Whether to print the addresses to ping")]
    addrs: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut img = image::open(args.file).unwrap();
    img = img.resize_exact(512, 512, FilterType::Lanczos3);
    run("2a06:a003:d040:1XXX:YYY:RR:GG:BB", &img, args.addrs).await
}

fn px_to_addr(px: (u32, u32, Rgba<u8>), ip_fmt: &str) -> Ipv6Addr {
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

    let ip_str = ip_fmt
        .replace("XXX", &format!("{x:03x}"))
        .replace("YYY", &format!("{y:03x}"))
        .replace("RR", &format!("{r:02x}"))
        .replace("GG", &format!("{g:02x}"))
        .replace("BB", &format!("{b:02x}"));

    Ipv6Addr::from_str(&ip_str).expect("given ip format should be correct")
}

async fn run(ip_fmt: &str, img: &DynamicImage, print_addrs: bool) {
    let config = Config {
        kind: ICMP::V6,
        ..Default::default()
    };

    let client = Client::new(&config).expect("ICMP client config should be valid");

    // let mut px_set = JoinSet::new();
    let addrs: Vec<Ipv6Addr> = img.pixels().map(|px| px_to_addr(px, ip_fmt)).collect();
    if print_addrs {
        println!("{:?}", addrs);
    }

    loop {
        for addr in addrs.iter() {
            tokio::spawn(ping(client.clone(), *addr));
        }
    }

    // while (px_set.join_next().await).is_some() {}
}

async fn ping(client: Client, addr: Ipv6Addr) {
    let mut pinger = client.pinger(addr.into(), PingIdentifier(0)).await;
    pinger.ping(PingSequence(0), &[]).await.ok();
}
