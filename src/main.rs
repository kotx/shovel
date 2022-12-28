#![feature(iter_array_chunks)]

mod icmp;
mod place;

use std::{sync::Arc, time::Duration};

use clap::Parser;
use icmp::ping_v6;
use image::{imageops::FilterType, GenericImageView};
use place::canvas::CanvasClient;
use rayon::{
    prelude::{IntoParallelRefIterator, ParallelBridge, ParallelIterator},
    slice::ParallelSliceMut,
};

use crate::place::{util::pixel_to_addr, Pixel};

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

    println!("Connecting to WebSocket...");
    let mut client = CanvasClient::new();
    client.setup();

    let canvas = Arc::clone(&client.canvas);

    std::thread::spawn(move || {
        loop {
            client.recv();
        }
    });

    println!("Sending pings...");

    let mut pixels: Vec<Pixel> = img
        .pixels()
        .par_bridge()
        .filter(|px| px.2[3] != 0 /* alpha 0 */)
        .collect();

    loop {
        let canvas = canvas.read().unwrap();

        if let Some(canvas) = canvas.as_ref() {
            pixels.par_sort_unstable_by_key(|px| {
                let canvas_color = &canvas.get_pixel(px.0, px.1).0.map(u16::from);
                let px_color = &px.2.0.map(u16::from);

                // Simple sum of absolute distance of R, G, B
                let total_dist = 255 * 3
                    - [0, 1, 2]
                        .iter()
                        .map(|&idx| canvas_color[idx].abs_diff(px_color[idx]))
                        .sum::<u16>();

                total_dist
            });
        }

        pixels
            .iter()
            .map(|px| pixel_to_addr(px, 1))
            .array_chunks::<8000>()
            .for_each(|addrs| {
                addrs.par_iter().for_each(|&addr| ping_v6(addr));
                std::thread::sleep(Duration::from_millis(50));
            });
    }
}
