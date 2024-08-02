use std::collections::BTreeMap;

use bytes::BytesMut;

use super::{RespDecode, RespEncode, RespError, RespFrame};

// - map: "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
impl RespEncode for BTreeMap<RespFrame, RespFrame> {
    fn encode(self) -> Vec<u8> {
        todo!()
    }
}

impl RespDecode for BTreeMap<RespFrame, RespFrame> {
    const PREFIX: &'static str = ":";
    fn decode(_buf: &mut BytesMut) -> Result<Self, RespError> {
        todo!()
    }

    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        todo!()
    }
}
