pub mod util {
    use image::Rgba;
    use std::{net::Ipv6Addr, str::FromStr};

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
}

pub mod canvas {
    use std::{
        net::TcpStream,
        sync::{Arc, Mutex},
    };

    use image::{imageops::overlay, DynamicImage, ImageFormat};
    use tungstenite::{connect, stream::MaybeTlsStream, WebSocket};

    const WEBSOCKET_ADDRESS: &str = "wss://v6.sys42.net/ws";

    pub type Canvas = Option<DynamicImage>;

    pub struct CanvasClient {
        pub canvas: Arc<Mutex<Canvas>>,
        ws: WebSocket<MaybeTlsStream<TcpStream>>,
    }

    impl CanvasClient {
        pub fn new() -> CanvasClient {
            CanvasClient {
                canvas: Arc::new(Mutex::new(None)),
                ws: connect(WEBSOCKET_ADDRESS).unwrap().0,
            }
        }

        pub fn setup(&mut self) {
            let init = self.ws.read_message().unwrap().into_data();
            *self.canvas.lock().unwrap() =
                Some(image::load_from_memory_with_format(&init, ImageFormat::Png).unwrap());
        }

        pub fn recv(&mut self) {
            let msg = self.ws.read_message().unwrap();
            if msg.is_binary() {
                let delta = msg.into_data();
                let delta_img = image::load_from_memory_with_format(&delta, ImageFormat::Png)
                    .expect("delta should convert to PNG image");

                let mut canvas = self.canvas.lock().unwrap();

                if canvas.is_none() {
                    panic!("Canvas must be initialized");
                }

                let canvas = canvas.as_mut().unwrap();
                overlay(canvas, &delta_img, 0, 0);
            } else if msg.is_text() {
                let text = msg.into_text().unwrap();
                println!("{}", text);
            }
        }
    }
}
