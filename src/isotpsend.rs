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

fn main() {
  // 1. parse CLI options
  // 2. read stdin
  // 3. connect to socket
  // 4. convert convertPduToFrames
  // 5. send every frame to device over websocket?
}
