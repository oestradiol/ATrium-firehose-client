#[cfg(test)]
mod tests;

use anyhow::bail;
use cbor4ii::core::utils::IoReader;
use ipld_core::ipld::Ipld;
use serde_ipld_dagcbor::de::Deserializer;
use std::io::Cursor;

// original definition:
//```
// export enum FrameType {
//   Message = 1,
//   Error = -1,
// }
// export const messageFrameHeader = z.object({
//   op: z.literal(FrameType.Message), // Frame op
//   t: z.string().optional(), // Message body type discriminator
// })
// export type MessageFrameHeader = z.infer<typeof messageFrameHeader>
// export const errorFrameHeader = z.object({
//   op: z.literal(FrameType.Error),
// })
// export type ErrorFrameHeader = z.infer<typeof errorFrameHeader>
// ```
#[derive(Debug, Clone, PartialEq, Eq)]
enum FrameHeader {
  Message(Option<String>),
  Error,
}

impl TryFrom<Ipld> for FrameHeader {
  type Error = anyhow::Error;

  fn try_from(value: Ipld) -> Result<Self, <Self as TryFrom<Ipld>>::Error> {
    if let Ipld::Map(map) = value {
      if let Some(Ipld::Integer(i)) = map.get("op") {
        match i {
          1 => {
            let t = if let Some(Ipld::String(s)) = map.get("t") {
              Some(s.clone())
            } else {
              None
            };
            return Ok(Self::Message(t));
          }
          -1 => return Ok(Self::Error),
          _ => {}
        }
      }
    }
    Err(anyhow::anyhow!("invalid frame type"))
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
  Message(Option<String>, MessageFrame),
  Error(ErrorFrame),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageFrame {
  pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorFrame {
  // TODO
  // body: Value,
}

impl TryFrom<Vec<u8>> for Frame {
  type Error = anyhow::Error;

  fn try_from(value: Vec<u8>) -> Result<Self, <Self as TryFrom<Vec<u8>>>::Error> {
    let mut cursor = Cursor::new(value);
    let mut deserializer = Deserializer::from_reader(IoReader::new(&mut cursor));
    let header: Ipld = serde::Deserialize::deserialize(&mut deserializer)?;

    // Error means the stream did not end (trailing data), which implies a second IPLD (in this case, the message body).
    // If the stream did end, the message body is empty, in which case we bail.
    let body = if deserializer.end().is_err() {
      let pos = usize::try_from(cursor.position())?;
      cursor.get_mut().drain(pos..).collect()
    } else {
      // TODO: Proper error handling
      bail!("invalid frame type")
    };

    if let FrameHeader::Message(t) = FrameHeader::try_from(header)? {
      Ok(Self::Message(t, MessageFrame { body }))
    } else {
      Ok(Self::Error(ErrorFrame {}))
    }
  }
}
