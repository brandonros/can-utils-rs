/*
const highNibble = (b) => (((b) >> 4) & 0x0F)
const lowNibble = (b) => ((b) & 0x0F)

class IsoTpReader {
  constructor(sendFlowControlFrame, onPdu, onError) {
    this.firstFrame = null
    this.expectedSize = null
    this.expectedSequenceNumber = 0x21
    this.consecutiveFrames = []
    this.sendFlowControlFrame = sendFlowControlFrame
    this.onPdu = onPdu
    this.onError = onError
  }

  recordSingleFrame(data) {
    const length = data[0]
    const serviceId = data[1]
    const payload = data.slice(2, length + 1)
    this.onPdu(serviceId, payload)
  }

  recordFirstFrame(data) {
    this.firstFrame = data
    this.expectedSize = (lowNibble(data[0]) << 8) + data[1]
  }

  rebuildMultiFrameMessage() {
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
  }

  recordConsecutiveFrame(data) {
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
  }

  recordFlowControlFrame(data) {
    const length = 8
    const serviceId = data[0]
    const payload = data.slice(1, length + 1)
    //this.onPdu(serviceId, payload)
  }

  recordFrame(data) {
    const pci = highNibble(data[0])
    if (pci === 0x00) { // single frame
      this.recordSingleFrame(data)
    } else if (pci === 0x01) { // first frame
      this.recordFirstFrame(data)
      this.sendFlowControlFrame()
    } else if (pci === 0x02) { // consecutive frame
      this.recordConsecutiveFrame(data)
    } else if (pci === 0x03) { // flow control frame
      this.recordFlowControlFrame(data)
    }
  }
}
*/

fn main() {
  // 1. parse CLI options
  // 2. connect to socket
  // 3. on websocket frame, log to isotpreader
  // 4. on PDU, log output
}
