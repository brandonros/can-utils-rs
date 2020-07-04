extern crate clap;
extern crate hex;
extern crate tungstenite;
extern crate url;

use std::io;

use clap::{App, Arg};

fn build_single_frame(service_id: u8, data: &Vec<u8>, tx_padding_byte: u8) -> Vec<u8> {
    let mut frame = vec![(data.len() + 1) as u8, service_id];
    for i in 0..data.len() {
        frame.push(data[i]);
    }
    let num_padding_bytes = 8 - frame.len();
    for _ in 0..num_padding_bytes {
        frame.push(tx_padding_byte);
    }
    return frame;
}

fn build_first_frame(service_id: u8, data: &Vec<u8>) -> Vec<u8> {
    let response_length = (data.len() + 1) as u16;
    let first_frame_data = &data[0..5];
    let first_frame_header = vec![
        ((0x01 << 4) ^ (response_length >> 8)) as u8,
        (response_length & 0xFF) as u8,
        service_id as u8,
    ];
    let mut frame = vec![];
    frame.extend_from_slice(&first_frame_header);
    frame.extend_from_slice(&first_frame_data);
    return frame;
}

fn build_consecutive_frame(sequence_number: u8, data: &Vec<u8>, tx_padding_byte: u8) -> Vec<u8> {
    let frame_length = if data.len() >= 7 { 7 } else { data.len() };
    let frame_data = &data[0..frame_length];
    let mut frame = vec![];
    frame.push(sequence_number);
    frame.extend_from_slice(frame_data);
    let padding_length = 7 - frame_data.len();
    for _ in 0..padding_length {
        frame.push(tx_padding_byte);
    }
    return frame;
}

fn convert_pdu_to_frames(service_id: u8, data: Vec<u8>, tx_padding_byte: u8) -> Vec<Vec<u8>> {
    if data.len() <= 6 {
        return vec![build_single_frame(service_id, &data, tx_padding_byte)];
    }
    let mut frames = vec![];
    frames.push(build_first_frame(service_id, &data));
    let mut remaining_data = &data[5..];
    let num_conseuctive_frames = if remaining_data.len() % 7 == 0 {
        remaining_data.len() / 7
    } else {
        (remaining_data.len() / 7) + 1
    };
    let mut sequence_number = 0x21;
    for _ in 0..num_conseuctive_frames {
        frames.push(build_consecutive_frame(
            sequence_number,
            &remaining_data.to_vec(),
            tx_padding_byte,
        ));
        sequence_number = sequence_number + 1;
        if sequence_number == 0x30 {
            sequence_number = 0x20;
        }
        remaining_data = if remaining_data.len() >= 7 {
            &remaining_data[7..]
        } else {
            &remaining_data[remaining_data.len()..]
        };
    }
    return frames;
}

fn read_stdin() -> Vec<u8> {
    let stdin = io::stdin();
    let mut buf = String::new();
    stdin.read_line(&mut buf).unwrap();
    return hex::decode(buf.trim().replace(" ", "")).unwrap();
}

fn main() {
    // parse CLI options
    let matches = App::new("isotpsend")
        .version("0.0.1")
        .about("send a single ISO-TP PDU")
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
    let padding_bytes = matches.value_of("padding_bytes").unwrap();
    let split_padding_bytes = padding_bytes.split(":").collect::<Vec<_>>();
    let tx_padding_byte = u32::from_str_radix(split_padding_bytes[0], 16).unwrap() as u8;
    let interface = matches.value_of("interface").unwrap();
    let st_min: u64 = matches.value_of("st_min").unwrap().parse().unwrap();
    let source_arbitration_id: u32 =
        u32::from_str_radix(matches.value_of("source_arbitration_id").unwrap(), 16).unwrap();
    // connect to server
    let (mut socket, _) = tungstenite::connect(url::Url::parse(interface).unwrap()).unwrap();
    // read stdin
    let stdin = read_stdin();
    let service_id = stdin[0];
    let data = &stdin[1..];
    // convert stdin to frames
    let frames = convert_pdu_to_frames(service_id, data.to_vec(), tx_padding_byte);
    for i in 0..frames.len() {
        if i == 1 {
          // TODO: wait for flow control frame
        }
        let frame = &frames[i];
        let mut buffer: Vec<u8> = vec![];
        buffer.extend_from_slice(&source_arbitration_id.to_be_bytes());
        buffer.extend_from_slice(&frame);
        socket.write_message(tungstenite::Message::Binary(buffer)).unwrap();
        std::thread::sleep(std::time::Duration::from_nanos(st_min));
    }
    // TODO: set up websocket reader to watch for flow control frame if pci == 0x03
    // disconnect from server
    socket.close(None).unwrap();
}
