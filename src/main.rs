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
    let device_handle = devices::tactrix_openport::new();
    // init server
    let server = std::net::TcpListener::bind("127.0.0.1:9001").unwrap();
    // listen for connections
    let websockets = Arc::new(Mutex::new(vec![]));
    for stream in server.incoming() {
        let mut websocket = tungstenite::server::accept(stream.unwrap()).unwrap();
        websockets.lock().unwrap().push(websocket);
        // read from socket, send to evice
        std::thread::spawn (move || {
            loop {
                let msg = websocket.read_message().unwrap().into_data();
                let arbitration_id = u32::from_be_bytes([
                    msg[0],
                    msg[1],
                    msg[2],
                    msg[3]
                ]);
                let data = &msg[4..];
                devices::tactrix_openport::send_can_frame(&device_handle, arbitration_id, data);
            }
        });
    }
    // read from device, send to sockets
    std::thread::spawn (move || {
        let mut handler = move |frame: Vec<u8>| {
            for websocket in websockets.lock().unwrap().iter() {
                let binary_frame = tungstenite::Message::Binary(frame);
                websocket.write_message(binary_frame).unwrap();
            }
        };
        devices::tactrix_openport::recv(&device_handle, &mut handler);
    });
}
