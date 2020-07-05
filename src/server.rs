extern crate tungstenite;

use can_utils_rs::devices;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

type SocketAddr = std::net::SocketAddr;
type TcpStream = std::net::TcpStream;
type Role = tungstenite::protocol::Role;

fn main() {
    // init device
    let device_handle = devices::tactrix_openport::new();
    let device_handle_arc = Arc::new(device_handle);
    // init server
    let server = std::net::TcpListener::bind("127.0.0.1:9001").unwrap();
    let read_streams_map: HashMap<SocketAddr, TcpStream> = HashMap::new();
    let write_streams_map: HashMap<SocketAddr, TcpStream> = HashMap::new();
    let read_streams_map_arc = Arc::new(Mutex::new(read_streams_map));
    let write_streams_map_arc = Arc::new(Mutex::new(write_streams_map));
    // read from device, send to sockets
    let write_streams_map_ref = write_streams_map_arc.clone();
    let device_handle_ref = device_handle_arc.clone();
    let device_thread = std::thread::spawn(move || {
        let mut handler = move |frame: Vec<u8>| {
            println!("got frame = {:?}", frame);
            let mut write_streams_map = write_streams_map_ref.lock().unwrap();
            println!("unlocked");
            for (peer_addr, stream) in write_streams_map.iter_mut() {
                let stream = stream.try_clone().unwrap();
                let mut websocket = tungstenite::WebSocket::<std::net::TcpStream>::from_raw_socket(stream, Role::Client, None);
                println!("writing to {:?}", peer_addr);
                let binary_frame = tungstenite::Message::Binary(frame.clone());
                websocket
                    .write_message(binary_frame)
                    .unwrap();
            }
        };
        devices::tactrix_openport::recv(&device_handle_ref, &mut handler);
    });
    // listen for connections
    let read_streams_map_ref = read_streams_map_arc.clone();
    let write_streams_map_ref = write_streams_map_arc.clone();
    let server_thread = std::thread::spawn(move || {
        for stream in server.incoming() {
            let mut websocket = tungstenite::server::accept(stream.unwrap()).unwrap();
            let peer_addr = websocket.get_mut().peer_addr().unwrap();
            let mut read_streams_map = read_streams_map_ref.lock().unwrap();
            read_streams_map.insert(peer_addr, websocket.get_mut().try_clone().unwrap());
            let mut write_streams_map = write_streams_map_ref.lock().unwrap();
            write_streams_map.insert(peer_addr, websocket.get_mut().try_clone().unwrap());
            let read_streams_map_ref = read_streams_map_arc.clone();
            let device_handle_ref = device_handle_arc.clone();
            // read from socket, send to device
            std::thread::spawn(move || {
                let read_streams_map = read_streams_map_ref.lock().unwrap();
                let stream = read_streams_map.get(&peer_addr).unwrap().try_clone().unwrap();
                let mut websocket = tungstenite::WebSocket::<std::net::TcpStream>::from_raw_socket(stream, Role::Client, None);
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
