extern crate clap;
extern crate hex;
extern crate tungstenite;
extern crate url;

use std::collections::HashMap;
use std::io;

use clap::{App, Arg};

struct IsoTpReader {
    pub first_frame: Vec<u8>,
    pub consecutive_frames: Vec<Vec<u8>>,
    pub sequence_number: u8,
    pub expected_size: u16,
}

fn high_nibble(b: u8) -> u8 {
    return ((b) >> 4) & 0x0F;
}

fn record_single_frame(
    arbitration_id: u32,
    data: Vec<u8>,
    isotp_reader: &IsoTpReader,
    on_flow_control: fn(Vec<u8>),
    on_pdu: fn(),
    on_error: fn(),
) {
    /*
      const length = data[0]
      const serviceId = data[1]
      const payload = data.slice(2, length + 1)
      this.onPdu(serviceId, payload)
    */
}

fn record_first_frame(
    arbitration_id: u32,
    data: Vec<u8>,
    isotp_reader: &IsoTpReader,
    on_flow_control: fn(Vec<u8>),
    on_pdu: fn(),
    on_error: fn(),
) {
    /*
      this.firstFrame = data
      this.expectedSize = (lowNibble(data[0]) << 8) + data[1]
    */
}

fn rebuild_multi_frame_message(
    arbitration_id: u32,
    data: Vec<u8>,
    isotp_reader: &IsoTpReader,
    on_flow_control: fn(Vec<u8>),
    on_pdu: fn(),
    on_error: fn(),
) {
    /*
      const output = []
      // skip first 2 bytes of first frame
      for (let i = 2; i < 8; ++i) {
        output.push(this.firstFrame[i])
      }
      this.consecutiveFrames.forEach(frame => {
        // skip first byte of consecutive frames
        for (let i = 1; i < 8; ++i) {
          output.push(frame[i])
        }
      })
      const isotpPayload = output.slice(0, this.expectedSize)
      const serviceId = isotpPayload[0]
      const data = isotpPayload.slice(1)
      this.onPdu(serviceId, data)
    */
}

fn record_consecutive_frame(
    arbitration_id: u32,
    data: Vec<u8>,
    isotp_reader: &IsoTpReader,
    on_flow_control: fn(Vec<u8>),
    on_pdu: fn(),
    on_error: fn(),
) {
    /*
      // validate we have a first frame
      if (!this.firstFrame) {
        this.onError(new Error('received unexpected consecutive frame'))
        return
      }
      // validate sequence number
      const sequenceNumber = data[0]
      if (sequenceNumber !== this.expectedSequenceNumber) {
        this.onError(new Error('received unexpected sequence number'))
        return
      }
      // wrap expectedSequenceNumber
      this.expectedSequenceNumber += 1
      if (this.expectedSequenceNumber === 0x30) {
        this.expectedSequenceNumber = 0x20
      }
      // store frame
      this.consecutiveFrames.push(data)
      // check if finished receiving
      const currentSize = 6 + this.consecutiveFrames.length * 7
      const finishedReceiving = currentSize >= this.expectedSize
      if (finishedReceiving) {
        this.rebuildMultiFrameMessage()
      }
    */
}

fn record_frame(
    arbitration_id: u32,
    data: Vec<u8>,
    isotp_reader: &IsoTpReader,
    on_flow_control: fn(Vec<u8>),
    on_pdu: fn(),
    on_error: fn(),
) {
    let pci = high_nibble(data[0]);
    if (pci == 0x00) {
        record_single_frame(
            arbitration,
            data,
            isotp_reader,
            on_flow_control,
            on_pdu,
            on_error,
        );
    } else if (pci == 0x01) {
        record_first_frame(
            arbitration,
            data,
            isotp_reader,
            on_flow_control,
            on_pdu,
            on_error,
        );
    } else if (pci == 0x02) {
        record_consecutive_frame(
            arbitration,
            data,
            isotp_reader,
            on_flow_control,
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
            // TODO: log to stdout
            isotp_reader_map.remove(&arbitration_id);
        };
        let on_error = || {
            isotp_reader_map.remove(&arbitration_id);
        };
        match isotp_reader_map.get(&arbitration_id) {
            Some(isotp_reader) => {
                record_frame(
                    arbitration_id,
                    data.to_vec(),
                    &isotp_reader,
                    on_flow_control,
                    on_pdu,
                    on_error,
                );
            }
            None => {
                let isotp_reader = IsoTpReader {
                    first_frame: vec![],
                    consecutive_frames: vec![],
                    sequence_number: 0x21,
                    expected_size: 0x00,
                };
                isotp_reader_map.insert(arbitration_id, isotp_reader);
                record_frame(
                    arbitration_id,
                    data.to_vec(),
                    &isotp_reader,
                    on_flow_control,
                    on_pdu,
                    on_error,
                );
            }
        }
    }
    // 4. on PDU, log output
}
