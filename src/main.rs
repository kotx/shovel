use std::{
    net::{IpAddr, Ipv6Addr},
    str::FromStr,
    time::Duration,
};

use async_recursion::async_recursion;
use clap::Parser;
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use surge_ping::{Client, Config, PingIdentifier, PingSequence, ICMP};
use tokio::task::JoinSet;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    file: std::path::PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut img = image::open(args.file).unwrap();
    img = img.resize_exact(512, 512, FilterType::Lanczos3);
    run("2a06:a003:d040:1XXX:YYY:RR:GG:BB", &img).await
}

async fn run(ip_fmt: &str, img: &DynamicImage) {
    let config = Config {
        kind: ICMP::V6,
        ..Default::default()
    };

    let client = Client::new(&config).expect("ICMP client config should be valid");

    let mut px_set = JoinSet::new();

    for px in img.pixels() {
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

        let addr = {
            let ip_str = ip_fmt
                .replace("XXX", &format!("{x:03x}"))
                .replace("YYY", &format!("{y:03x}"))
                .replace("RR", &format!("{r:02x}"))
                .replace("GG", &format!("{g:02x}"))
                .replace("BB", &format!("{b:02x}"));

            Ipv6Addr::from_str(&ip_str).expect("given ip format should be correct")
        };

        px_set.spawn(ping(client.clone(), IpAddr::V6(addr)));
    }

    while (px_set.join_next().await).is_some() {}
}

#[async_recursion]
async fn ping(client: Client, addr: IpAddr) {
    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut pinger = client.pinger(addr, PingIdentifier(0)).await;
    let pong = pinger.ping(PingSequence(0), &[]).await;
    if pong.is_err() {
        println!("{:?} {}", pong, addr);
        ping(client, addr).await;
    }
}
