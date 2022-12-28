mod icmp;
mod place;

use std::net::Ipv6Addr;

use clap::Parser;
use icmp::ping_v6;
use image::{imageops::FilterType, GenericImageView};
use place::canvas::CanvasClient;
use rayon::prelude::{ParallelBridge, ParallelIterator};

use crate::place::util::pixel_to_addr;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(help = "Image file to shovel")]
    file: std::path::PathBuf,
}

fn main() {
    let args = Args::parse();

    let mut img = image::open(args.file).unwrap();
    println!("Resizing image...");
    img = img.resize_exact(512, 512, FilterType::Lanczos3);

    println!("Converting image to IPV6 addresses...");
    let pixels = img.pixels().par_bridge().filter(|_px| /* cond */ true);
    let addrs: Vec<Ipv6Addr> = pixels.map(|px| pixel_to_addr(px, 1)).collect();

    println!("Connecting to WebSocket...");
    let mut client = CanvasClient::new();
    client.setup();

    let canvas = client.canvas.clone();

    std::thread::spawn(move || loop {
        client.recv();
    });

    println!("Sending pings...");
    loop {
        addrs.iter().for_each(|addr| ping_v6(*addr));
        println!("{:?}", canvas)
    }
}
