extern crate tungstenite;

use can_utils_rs::devices;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

type SocketAddr = std::net::SocketAddr;
type WebSocket = tungstenite::WebSocket<std::net::TcpStream>;

fn main() {
    // init device
    let device_handle = devices::tactrix_openport::new();
    let device_handle_arc = Arc::new(device_handle);
    // init server
    let server = std::net::TcpListener::bind("127.0.0.1:9001").unwrap();
    let websockets_map: HashMap<SocketAddr, WebSocket> = HashMap::new();
    let websockets_map_arc = Arc::new(Mutex::new(websockets_map));
    // read from device, send to sockets
    let websockets_map_ref = websockets_map_arc.clone();
    let device_handle_ref = device_handle_arc.clone();
    let device_thread = std::thread::spawn(move || {
        let mut handler = move |frame: Vec<u8>| {
            println!("got frame = {:?}", frame);
            let mut websockets_map = websockets_map_ref.lock().unwrap();
            println!("unlocked");
            for (peer_addr, websocket) in (*websockets_map).iter_mut() {
                println!("writing to {:?}", peer_addr);
                let binary_frame = tungstenite::Message::Binary(frame.clone());
                (*websocket)
                    .write_message(binary_frame)
                    .unwrap();
            }
        };
        devices::tactrix_openport::recv(&device_handle_ref, &mut handler);
    });
    // listen for connections
    let _device_handle_ref = device_handle_arc.clone();
    let websockets_map_ref = websockets_map_arc.clone();
    let server_thread = std::thread::spawn(move || {
        for stream in server.incoming() {
            let mut websocket = tungstenite::server::accept(stream.unwrap()).unwrap();
            let peer_addr = websocket.get_mut().peer_addr().unwrap();
            let mut websockets_map = websockets_map_ref.lock().unwrap();
            (*websockets_map).insert(peer_addr, websocket);
            let websockets_map_ref = websockets_map_arc.clone();
            let device_handle_ref = device_handle_arc.clone();
            // read from socket, send to device
            std::thread::spawn(move || {
                let mut websockets_map = websockets_map_ref.lock().unwrap();
                let mut websocket = websockets_map.get_mut(&peer_addr).unwrap();
                loop {
                    let msg = websocket
                        .read_message()
                        .unwrap()
                        .into_data();
                    if msg.len() == 0 {
                        continue;
                    }
                    println!("{:?}", msg);
                    let arbitration_id = u32::from_be_bytes([msg[0], msg[1], msg[2], msg[3]]);
                    let data = &msg[4..];
                    devices::tactrix_openport::send_can_frame(
                        &device_handle_ref,
                        arbitration_id,
                        data,
                    );
                }
            });
        }
    });
    device_thread.join().unwrap();
    server_thread.join().unwrap();
}
