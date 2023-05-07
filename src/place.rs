use image::Rgba;
pub type Pixel = (u32, u32, Rgba<u8>);

pub mod util {
    use std::{net::Ipv6Addr, str::FromStr};

    use super::Pixel;

    const IP_FORMAT: &str = "2602:fa9b:202:SXXX:YYY:RR:GG:BB";

    pub fn pixel_to_addr(px: &Pixel, size: usize) -> Ipv6Addr {
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
}

pub mod canvas {
    use std::{
        net::TcpStream,
        sync::{Arc, RwLock},
    };

    use image::{imageops::overlay, ImageFormat, RgbaImage};
    use tungstenite::{connect, stream::MaybeTlsStream, WebSocket};

    const WEBSOCKET_ADDRESS: &str = "wss://v6staging.sys42.net/ws";

    pub struct CanvasClient {
        pub canvas: Arc<RwLock<Option<RgbaImage>>>,
        ws: WebSocket<MaybeTlsStream<TcpStream>>,
    }

    impl CanvasClient {
        pub fn new() -> CanvasClient {
            CanvasClient {
                canvas: Arc::new(RwLock::new(None)),
                ws: connect(WEBSOCKET_ADDRESS).unwrap().0,
            }
        }

        pub fn setup(&mut self) {
            let init = self.ws.read_message().unwrap().into_data();
            *self.canvas.write().unwrap() = Some(
                image::load_from_memory_with_format(&init, ImageFormat::Png)
                    .unwrap()
                    .into_rgba8(),
            );
        }

        pub fn recv(&mut self) {
            let msg = self.ws.read_message().unwrap();
            if msg.is_binary() {
                let delta = msg.into_data();
                let delta_img = image::load_from_memory_with_format(&delta, ImageFormat::Png)
                    .expect("delta should convert to PNG image");

                if let Some(canvas) = self.canvas.write().unwrap().as_mut() {
                    overlay(canvas, &delta_img, 0, 0);
                } else {
                    panic!("Canvas must be initialized");
                }
            } else if msg.is_text() {
                // let text = msg.into_text().unwrap();
                // println!("{}", text);
            }
        }
    }
}
