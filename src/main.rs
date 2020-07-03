extern crate tungstenite;

use can_utils_rs::{devices};
use std::cell::RefCell;
use std::rc::Rc;

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
    let mut websockets = vec![];
    for stream in server.incoming() {
        let websocket = tungstenite::server::accept(stream.unwrap()).unwrap();
        let websocket = Rc::new(RefCell::new(websocket));
        websockets.push(websocket.clone());
        // read from socket, send to device
        loop {
            let msg = websocket.borrow_mut().read_message().unwrap().into_data();
            let arbitration_id = u32::from_be_bytes([
                msg[0],
                msg[1],
                msg[2],
                msg[3]
            ]);
            let data = &msg[4..];
            devices::tactrix_openport::send_can_frame(&device_handle, arbitration_id, data);
        }
    }
    // read from device, send to sockets
    let mut handler = move |frame: Vec<u8>| {
        for websocket in websockets.iter() {
            let binary_frame = tungstenite::Message::Binary(frame.clone());
            websocket.try_borrow_mut().unwrap().write_message(binary_frame).unwrap();
        }
    };
    devices::tactrix_openport::recv(&device_handle, &mut handler);
}
