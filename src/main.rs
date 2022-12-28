mod icmp;
mod place;

use clap::Parser;
use icmp::ping_v6;
use image::{imageops::FilterType, GenericImageView};

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
    img = img.resize_exact(512, 512, FilterType::Lanczos3);

    let pixels = img.pixels().filter(|_px| /* cond */ true);
    let addrs = pixels.map(|px| pixel_to_addr(px, 1));

    loop {
        addrs.clone().for_each(ping_v6);
    }
}
