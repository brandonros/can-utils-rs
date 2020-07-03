extern crate clap;
extern crate hex;
extern crate tungstenite;
extern crate url;

use std::collections::HashMap;
use std::io;

use clap::{App, Arg};

struct IsoTpReader {
    pub first_frame: Option<Vec<u8>>,
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

fn record_single_frame(
    data: &Vec<u8>,
    isotp_reader: &IsoTpReader,
    on_pdu: fn(u8, &Vec<u8>)
) {
    let length = data[0];
    let service_id = data[1];
    let payload = &data[2..((length as usize) + 1)];
    on_pdu(service_id, &payload.to_vec())
}

fn record_first_frame(
    data: Vec<u8>,
    isotp_reader: &IsoTpReader,
    on_error: fn(String)
) {
    // validate we do not already have a first frame
    if (isotp_reader.first_frame.is_some()) {
        on_error(String::from("unexpected first frame"));
        return;
    }
    isotp_reader.first_frame = Some(data);
    isotp_reader.expected_size = ((low_nibble(data[0]) << 8) as u16) + (data[1] as u16);
}

fn rebuild_multi_frame_message(
    data: Vec<u8>,
    isotp_reader: &IsoTpReader,
    on_pdu: fn(u8, &Vec<u8>)
) {
    let mut output = vec![];
    for i in 2..8 {
        output.push(isotp_reader.first_frame.unwrap()[i]);
    }
    for consecutive_frame in isotp_reader.consecutive_frames {
        for i in 1..8 {
            output.push(consecutive_frame[i]);
        }
    }
    let isotp_payload = &output[0..isotp_reader.expected_size as usize];
    let service_id = isotp_payload[0];
    let data = &isotp_payload[1..];
    on_pdu(service_id, &data.to_vec())
}

fn record_consecutive_frame(
    data: Vec<u8>,
    isotp_reader: &IsoTpReader,
    on_pdu: fn(u8, &Vec<u8>),
    on_error: fn(String)
) {
    // validate we have a first frame
    if (isotp_reader.first_frame.is_none()) {
        on_error(String::from("unexpected conseuctive frame; no first frame"));
        return;
    }
    // validate sequence number
    let sequence_number = data[0];
    if sequence_number != isotp_reader.sequence_number {
        on_error(String::from("unexpected sequence number"));
        return;
    }
    // wrap expectedSequenceNumber
    isotp_reader.sequence_number = isotp_reader.sequence_number + 1;
    if isotp_reader.sequence_number == 0x30 {
        isotp_reader.sequence_number = 0x20;
    }
    // store frame
    isotp_reader.consecutive_frames.push(data);
    // check if finished receiving
    let current_size = 6 + isotp_reader.consecutive_frames.len() * 7;
    let finished_receiving = current_size >= isotp_reader.expected_size as usize;
    if (finished_receiving) {
        rebuild_multi_frame_message(data, isotp_reader, on_pdu);
    }
}

fn record_frame(
    data: Vec<u8>,
    isotp_reader: &IsoTpReader,
    on_flow_control: fn(),
    on_pdu: fn(u8, &Vec<u8>),
    on_error: fn(String),
) {
    let pci = high_nibble(data[0]);
    if (pci == 0x00) {
        record_single_frame(
            &data,
            isotp_reader,
            on_pdu
        );
    } else if (pci == 0x01) {
        record_first_frame(
            data,
            isotp_reader,
            on_error
        );
        on_flow_control();
    } else if (pci == 0x02) {
        record_consecutive_frame(
            data,
            isotp_reader,
            on_pdu,
            on_error,
        );
    } else if (pci == 0x03) {
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
    // connect to server
    let (mut socket, _) = tungstenite::connect(url::Url::parse(interface).unwrap()).unwrap();
    // on websocket frame, log to isotpreader
    let isotp_reader_map: HashMap<u32, IsoTpReader> = HashMap::new();
    loop {
        let frame = socket.read_message().unwrap().into_data();
        let arbitration_id = u32::from_be_bytes([frame[0], frame[1], frame[2], frame[3]]);
        let data = &frame[4..];
        let on_flow_control = || {
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
            socket.write_message(tungstenite::Message::Binary(buffer));
        };
        let on_pdu = |pdu: Vec<u8>| {
            let mut output = String::new();
            for byte in pdu {
                output = format!("{} {:02x}", output, byte);
            }
            println!("{}", output.trim());
            isotp_reader_map.remove(&arbitration_id);
        };
        let on_error = |reason: String| {
            println!("error: {}", reason);
            isotp_reader_map.remove(&arbitration_id);
        };
        match isotp_reader_map.get(&arbitration_id) {
            Some(isotp_reader) => {
                record_frame(
                    data.to_vec(),
                    &isotp_reader,
                    on_flow_control,
                    on_pdu,
                    on_error,
                );
            }
            None => {
                let isotp_reader = IsoTpReader {
                    first_frame: Some(vec![]),
                    consecutive_frames: vec![],
                    sequence_number: 0x21,
                    expected_size: 0x00,
                };
                isotp_reader_map.insert(arbitration_id, isotp_reader);
                record_frame(
                    data.to_vec(),
                    &isotp_reader,
                    on_flow_control,
                    on_pdu,
                    on_error,
                );
            }
        }
    }
}
