mod backend;
mod cmd;
mod resp;
mod respv2;

pub use backend::*;
pub use cmd::*;
pub use resp::{
    BulkString, RespArray, RespEncode, RespError, RespFrame, RespNull, SimpleError, SimpleString,
};
// pub use resp::*;
pub use respv2::RespDecodeV2;

use anyhow::Result;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug)]
pub struct RespFrameCodec;

impl Encoder<RespFrame> for RespFrameCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: RespFrame, dst: &mut bytes::BytesMut) -> Result<()> {
        let encoded = item.encode();
        dst.extend_from_slice(&encoded);
        Ok(())
    }
}

impl Decoder for RespFrameCodec {
    type Item = RespFrame;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<RespFrame>> {
        match RespFrame::decode(src) {
            Ok(frame) => Ok(Some(frame)),
            Err(RespError::NotComplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
