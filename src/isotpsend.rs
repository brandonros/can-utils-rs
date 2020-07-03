/*

class IsoTpWriter {
  constructor() {

  }

  buildSingleFrame(serviceId, data) {
    const frame = [data.length + 1, serviceId]
    for (let i = 0; i < data.length; ++i) {
      frame[i + 2] = data[i]
    }
    for (let i = frame.length; i < 8; ++i) {
      frame.push(0x55) // padding
    }
    return Buffer.from(frame)
  }

  buildFirstFrame(serviceId, data) {
    const responseLength = data.length + 1 // add a byte for response SID
    const firstFrameData = data.slice(0, 5)
    const firstFrameHeader = Buffer.from([
      (0x01 << 4) ^ (responseLength >> 8),
      responseLength & 0xFF,
      serviceId
    ])
    return Buffer.concat([
      firstFrameHeader,
      firstFrameData
    ])
  }

  buildConsecutiveFrame(sequenceNumber, remainingData) {
    let frameData = remainingData.slice(0, 7)
    // Pad last frame
    if (frameData.length < 7) {
      const paddingLength = 7 - frameData.length
      const padding = Buffer.from(new Array(paddingLength).fill(0x55))
      frameData = Buffer.concat([
        frameData,
        padding
      ])
    }
    const consecutiveFrameHeader = Buffer.from([
      sequenceNumber
    ])
    return Buffer.concat([
      consecutiveFrameHeader,
      frameData
    ])
  }

  convertPduToFrames(serviceId, data) {
    if (data.length <= 6) {
      return [this.buildSingleFrame(serviceId, data)]
    }
    const frames = []
    frames.push(this.buildFirstFrame(serviceId, data))
    let remainingData = data.slice(5) // first frame data length = 5
    const numConsecutiveFrames = Math.ceil(remainingData.length / 7)
    let sequenceNumber = 0x21
    for (let i = 0; i < numConsecutiveFrames; ++i) {
      frames.push(this.buildConsecutiveFrame(sequenceNumber, remainingData))
      sequenceNumber += 1
      // Wrap consecutive frame counter
      if (sequenceNumber === 0x30) {
        sequenceNumber = 0x20
      }
      remainingData = remainingData.slice(7)
    }
    return frames
  }
}
*/

extern crate tungstenite;
extern crate hex;
extern crate clap;
extern crate url;

use std::io;

use clap::{Arg, App};

fn convert_pdu_to_frames(service_id: u8, data: Vec<u8>) -> Vec<Vec<u8>> {
  return vec![
    vec![0x02, 0x10, 0x03]
  ];
}

fn read_stdin() -> Vec<u8> {
  let stdin = io::stdin();
  let mut buf = String::new();
  stdin.read_line(&mut buf);
  return hex::decode(buf.trim().replace(" ", "")).unwrap();
}

fn main() {
  // parse CLI options
  let matches = App::new("isotpsend")
      .version("0.0.1")
      .about("send a single ISO-TP PDU")
      .arg(Arg::with_name("source_arbitration_id")
           .short("s")
           .long("source-arbitration-id")
           .help("source arbitration ID")
           .takes_value(true)
           .required(true)
      )
      .arg(Arg::with_name("destination_arbitration_id")
           .short("d")
           .long("destination-arbitration-id")
           .help("destination arbitration ID")
           .takes_value(true)
           .required(true)
      )
      .arg(Arg::with_name("padding_bytes")
           .short("p")
           .long("padding-bytes")
           .help("TX:RX padding byte")
           .takes_value(true)
           .required(true)
      )
      .arg(Arg::with_name("st_min")
           .short("f")
           .long("st-min")
           .help("STMin in nanoseconds")
           .takes_value(true)
           .required(true)
      )
      .arg(Arg::with_name("interface")
           .help("CAN interface")
           .required(true)
      )
      .get_matches();
  let interface = matches.value_of("interface").unwrap();
  let st_min: u64 = matches.value_of("st_min").unwrap().parse().unwrap();
  let source_arbitration_id: u32 = u32::from_str_radix(matches.value_of("source_arbitration_id").unwrap(), 16).unwrap();
  // connect to server
  let (mut socket, _) = tungstenite::connect(url::Url::parse(interface).unwrap()).unwrap();
  // read stdin
  let stdin = read_stdin();
  let service_id = stdin[0];
  let data = &stdin[1..];
  // convert stdin to frames
  let frames = convert_pdu_to_frames(service_id, data.to_vec());
  for frame in frames {
    let mut buffer: Vec<u8> = vec![];
    buffer.extend_from_slice(&source_arbitration_id.to_be_bytes());
    buffer.extend_from_slice(&frame);
    socket.write_message(tungstenite::Message::Binary(buffer));
    std::thread::sleep(std::time::Duration::from_nanos(st_min));
  }
  socket.close(None).unwrap();
}
