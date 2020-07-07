extern crate clap;
extern crate hex;
extern crate tungstenite;
extern crate url;
extern crate native_tls;

use clap::{App, Arg};

type WebSocket = tungstenite::WebSocket<tungstenite::stream::Stream<std::net::TcpStream, native_tls::TlsStream<std::net::TcpStream>>>;

struct IsoTpReader {
    pub first_frame: Vec<u8>,
    pub consecutive_frames: Vec<Vec<u8>>,
    pub sequence_number: u8,
    pub expected_size: u16,
}

fn high_nibble(b: u8) -> u8 {
    return ((b) >> 4) & 0x0F;
}

fn low_nibble(b: u8) -> u8 {
    return b & 0x0F;
}

fn on_error(isotp_reader: &mut Option<IsoTpReader>, reason: String) {
    println!("error: {}", reason);
    *isotp_reader = None;
}

fn on_pdu(isotp_reader: &mut Option<IsoTpReader>, service_id: u8, pdu: Vec<u8>) {
    let mut output = format!("{:02x}", service_id);
    for byte in pdu {
        output = format!("{} {:02x}", output, byte);
    }
    println!("{}", output.trim());
    *isotp_reader = None;
}

fn on_flow_control(socket: &mut WebSocket, st_min: u64, source_arbitration_id: u32) {
    let flow_control_frame: Vec<u8> = vec![
        0x30,
        0x00,
        (st_min / 1000000) as u8,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
    ];
    let mut buffer: Vec<u8> = vec![];
    buffer.extend_from_slice(&source_arbitration_id.to_be_bytes());
    buffer.extend_from_slice(&flow_control_frame);
    socket.write_message(tungstenite::Message::Binary(buffer)).unwrap();
}

fn record_single_frame(isotp_reader: &mut Option<IsoTpReader>, data: &Vec<u8>) {
    let length = data[0];
    let service_id = data[1];
    let payload = &data[2..((length as usize) + 1)];
    on_pdu(isotp_reader, service_id, payload.to_vec())
}

fn record_first_frame(isotp_reader: &mut Option<IsoTpReader>, data: &Vec<u8>) {
    let mut temp_isotp_reader = isotp_reader.as_mut().unwrap();
    // validate we do not already have a first frame
    if temp_isotp_reader.first_frame.len() != 0 {
        on_error(isotp_reader, String::from("unexpected first frame"));
        return;
    }
    temp_isotp_reader.first_frame = vec![];
    temp_isotp_reader.first_frame.extend(data);
    temp_isotp_reader.expected_size = ((low_nibble(data[0]) as u16) << 8) + (data[1] as u16);
}

fn rebuild_multi_frame_message(isotp_reader: &mut Option<IsoTpReader>) {
    let temp_isotp_reader = isotp_reader.as_mut().unwrap();
    let mut output = vec![];
    for i in 2..8 {
        output.push(temp_isotp_reader.first_frame[i]);
    }
    for consecutive_frame in &temp_isotp_reader.consecutive_frames {
        for i in 1..8 {
            output.push(consecutive_frame[i]);
        }
    }
    let isotp_payload = &output[0..temp_isotp_reader.expected_size as usize];
    let service_id = isotp_payload[0];
    let data = &isotp_payload[1..];
    on_pdu(isotp_reader, service_id, data.to_vec())
}

fn record_consecutive_frame(isotp_reader: &mut Option<IsoTpReader>, data: &Vec<u8>) {
    let mut temp_isotp_reader = isotp_reader.as_mut().unwrap();
    // validate we have a first frame
    if temp_isotp_reader.first_frame.len() == 0 {
        on_error(isotp_reader, String::from("unexpected conseuctive frame; no first frame"));
        return;
    }
    // validate sequence number
    let sequence_number = data[0];
    if sequence_number != temp_isotp_reader.sequence_number {
        on_error(isotp_reader, String::from("unexpected sequence number"));
        return;
    }
    // wrap expectedSequenceNumber
    temp_isotp_reader.sequence_number = temp_isotp_reader.sequence_number + 1;
    if temp_isotp_reader.sequence_number == 0x30 {
        temp_isotp_reader.sequence_number = 0x20;
    }
    // store frame
    temp_isotp_reader.consecutive_frames.push(data.to_vec());
    // check if finished receiving
    let current_size = 6 + temp_isotp_reader.consecutive_frames.len() * 7;
    let finished_receiving = current_size >= temp_isotp_reader.expected_size as usize;
    if finished_receiving {
        rebuild_multi_frame_message(isotp_reader);
    }
}

fn record_frame(socket: &mut WebSocket, isotp_reader: &mut Option<IsoTpReader>, st_min: u64, source_arbitration_id: u32, data: &Vec<u8>) {
    let pci = high_nibble(data[0]);
    if pci == 0x00 {
        record_single_frame(isotp_reader, &data);
    } else if pci == 0x01 {
        record_first_frame(isotp_reader, data);
        on_flow_control(socket, st_min, source_arbitration_id);
    } else if pci == 0x02 {
        record_consecutive_frame(isotp_reader, data);
    } else if pci == 0x03 {
        // flow control; ignore
    } else {
        panic!("Unknown PCI");
    }
}

fn main() {
    // parse CLI options
    let matches = App::new("isotprecv")
        .version("0.0.1")
        .about("receive ISO-TP PDUs")
        // flags
        .arg(
            Arg::with_name("listen")
                .short("l")
                .long("listen")
                .help("listen mode"),
        )
        // options
        .arg(
            Arg::with_name("source_arbitration_id")
                .short("s")
                .long("source-arbitration-id")
                .help("source arbitration ID")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("destination_arbitration_id")
                .short("d")
                .long("destination-arbitration-id")
                .help("destination arbitration ID")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("padding_bytes")
                .short("p")
                .long("padding-bytes")
                .help("TX:RX padding byte")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("st_min")
                .short("f")
                .long("st-min")
                .help("STMin in nanoseconds")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("interface")
                .help("CAN interface")
                .required(true),
        )
        .get_matches();
    let interface = matches.value_of("interface").unwrap();
    let st_min: u64 = matches.value_of("st_min").unwrap().parse().unwrap();
    let source_arbitration_id: u32 =
        u32::from_str_radix(matches.value_of("source_arbitration_id").unwrap(), 16).unwrap();
    let destination_arbitration_id: u32 =
        u32::from_str_radix(matches.value_of("destination_arbitration_id").unwrap(), 16).unwrap();
    // connect to server
    let (mut socket, _) = tungstenite::client::connect(url::Url::parse(interface).unwrap()).unwrap();
    // on websocket frame, log to isotpreader
    let mut isotp_reader: Option<IsoTpReader> = None;
    loop {
        // read_from_socket
        let frame = socket.read_message().unwrap().into_data();
        let arbitration_id = u32::from_be_bytes([frame[0], frame[1], frame[2], frame[3]]);
        let should_drop = arbitration_id != destination_arbitration_id;
        if should_drop {
            continue;
        }
        // record_frame
        let data = &frame[4..];
        if isotp_reader.is_none() {
            isotp_reader = Some(IsoTpReader {
                first_frame: vec![],
                consecutive_frames: vec![],
                sequence_number: 0x21,
                expected_size: 0x00,
            });
        }
        record_frame(&mut socket, &mut isotp_reader, st_min, source_arbitration_id, &data.to_vec());
    }
}
