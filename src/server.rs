extern crate tungstenite;

use can_utils_rs::devices;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

type SocketAddr = std::net::SocketAddr;
type TcpStream = std::net::TcpStream;
type WebSocket = tungstenite::WebSocket<std::net::TcpStream>;

fn main() {
    // init device
    let device_handle = devices::tactrix_openport::new();
    let device_handle_arc = Arc::new(device_handle);
    // init server
    let server = std::net::TcpListener::bind("127.0.0.1:9001").unwrap();
    let websockets_read_map: HashMap<SocketAddr, Arc<Mutex<TcpStream>>> = HashMap::new();
    let websockets_write_map: HashMap<SocketAddr, Arc<Mutex<TcpStream>>> = HashMap::new();
    let websockets_read_map_arc = Arc::new(Mutex::new(websockets_read_map));
    let websockets_write_map_arc = Arc::new(Mutex::new(websockets_write_map));
    // read from device, send to sockets
    let websockets_write_map_ref = websockets_write_map_arc.clone();
    let device_handle_ref = device_handle_arc.clone();
    let device_thread = std::thread::spawn(move || {
        let mut handler = move |frame: Vec<u8>| {
            println!("got frame = {:?}", frame);
            let mut websockets_write_map = websockets_write_map_ref.lock().unwrap();
            println!("unlocked");
            for (peer_addr, stream) in (*websockets_write_map).iter_mut() {
                println!("writing to {:?}", peer_addr);
                let binary_frame = tungstenite::Message::Binary(frame.clone());
                let mut websocket = tungstenite::WebSocket::from_raw_socket(stream.lock().unwrap(), tungstenite::protocol::Role::Client, None);
                websocket
                    .write_message(binary_frame)
                    .unwrap();
            }
        };
        devices::tactrix_openport::recv(&device_handle_ref, &mut handler);
    });
    // listen for connections
    let websockets_read_map_ref = websockets_read_map_arc.clone();
    let server_thread = std::thread::spawn(move || {
        for stream in server.incoming() {
            // accept websocket and store in hashmaps
            let mut websocket = tungstenite::server::accept(stream.unwrap()).unwrap();
            let peer_addr = websocket.get_mut().peer_addr().unwrap();
            let read_stream = websocket.get_ref().try_clone().unwrap();
            let write_stream = websocket.get_ref().try_clone().unwrap();
            let mut websockets_read_map = websockets_read_map_ref.lock().unwrap();
            let mut websockets_write_map = websockets_write_map_ref.lock().unwrap();
            (*websockets_read_map).insert(peer_addr, Arc::new(Mutex::new(read_stream)));
            (*websockets_write_map).insert(peer_addr, Arc::new(Mutex::new(write_stream)));
            // read from socket, send to device
            let websockets_read_map_ref = websockets_read_map_arc.clone();
            let device_handle_ref = device_handle_arc.clone();
            std::thread::spawn(move || {
                let mut websockets_read_map = websockets_read_map_ref.lock().unwrap();
                let stream = websockets_read_map.get_mut(&peer_addr).unwrap();
                let mut websocket = tungstenite::WebSocket::from_raw_socket(stream.lock().unwrap(), tungstenite::protocol::Role::Client, None);
                loop {
                    println!("waiting for read...");
                    let msg = websocket
                        .read_message()
                        .unwrap()
                        .into_data();
                    if msg.len() == 0 {
                        continue;
                    }
                    println!("read {:?}", msg);
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
