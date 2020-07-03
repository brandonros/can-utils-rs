extern crate tungstenite;

use std::sync::{Arc, Mutex};
use can_utils_rs::{devices};

/*
const run = async () => {
  const socket = new WebSocket('ws://127.0.0.1:9001')
  await new Promise(resolve => socket.onopen = resolve)
  socket.send(new Uint8Array([
    0x00,
    0x00,
    0x07,
    0xE5,
    0x02,
    0x10,
    0x03,
    0x55,
    0x55,
    0x55,
    0x55
  ]))
}

run()
*/

fn main() {
    // init device
    let device_handle_main = Arc::new(devices::tactrix_openport::new());
    let device_handle1 = device_handle_main.clone();
    let device_handle2 = device_handle_main.clone();
    // init server
    let server = std::net::TcpListener::bind("127.0.0.1:9001").unwrap();
    // listen for connections
    let mut websockets = Arc::new(Mutex::new(vec![]));
    std::thread::spawn (move || {
        for stream in server.incoming() {
            let websocket = Arc::new(Mutex::new(tungstenite::server::accept(stream.unwrap()).unwrap()));
            websockets.lock().unwrap().push(websocket.clone());
            // read from socket, send to device
            std::thread::spawn (move || {
                loop {
                    let msg = websocket.lock().unwrap().read_message().unwrap().into_data();
                    let arbitration_id = u32::from_be_bytes([
                        msg[0],
                        msg[1],
                        msg[2],
                        msg[3]
                    ]);
                    let data = &msg[4..];
                    devices::tactrix_openport::send_can_frame(&device_handle1, arbitration_id, data);
                }
            });
        }
    });
    // read from device, send to sockets
    std::thread::spawn (move || {
        let mut handler = move |frame: Vec<u8>| {
            for websocket in websockets.lock().unwrap().iter() {
                let binary_frame = tungstenite::Message::Binary(frame.clone());
                websocket.lock().unwrap().write_message(binary_frame).unwrap();
            }
        };
        devices::tactrix_openport::recv(&device_handle2, &mut handler);
    });
}
