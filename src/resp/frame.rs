use std::{collections::BTreeMap, hash::Hash};

use bytes::BytesMut;
use enum_dispatch::enum_dispatch;

use crate::{BulkString, RespArray, RespNull, SimpleError, SimpleString};

use super::{Double, RespDecode, RespError};

#[enum_dispatch(RespEncode)]
#[derive(Debug, Clone, PartialEq, Ord, Hash, Eq, PartialOrd)]
pub enum RespFrame {
    Null(RespNull),
    SimpleString(SimpleString),
    Error(SimpleError),
    Integer(i64),
    Boolean(bool),
    BulkString(Option<BulkString>),
    Array(Option<RespArray>),
    Double(Double),
    Map(BTreeMap<RespFrame, RespFrame>),
}

// impl RespEncode for RespFrame {
//     fn encode(self) -> Vec<u8> {
//         match self {
//             RespFrame::SimpleString(s) => s.encode(),
//         }
//     }
// }

impl From<&str> for RespFrame {
    fn from(s: &str) -> Self {
        SimpleString(s.to_string()).into()
    }
}

impl From<&[u8]> for RespFrame {
    fn from(s: &[u8]) -> Self {
        Some(BulkString(s.to_vec())).into()
    }
}

impl<const N: usize> From<&[u8; N]> for RespFrame {
    fn from(s: &[u8; N]) -> Self {
        Some(BulkString(s.to_vec())).into()
    }
}

// #[enum_dispatch(RespDecode)]

impl RespDecode for RespFrame {
    const PREFIX: &'static str = "";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'+') => {
                let frame = SimpleString::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'-') => match SimpleError::decode(buf) {
                Ok(frame) => Ok(frame.into()),
                Err(e) => Err(e),
            },
            Some(b'$') => {
                let frame: Option<BulkString> = Option::<BulkString>::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'*') => match Option::<RespArray>::decode(buf) {
                Ok(frame) => Ok(frame.into()),
                Err(e) => Err(e),
            },
            None => Err(RespError::NotComplete),
            _ => Err(RespError::InvalidFrameType(format!(
                "expect_length: unknown frame type: {:?}",
                buf
            ))),
        }
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'*') => Option::<RespArray>::expect_length(buf),
            Some(b'$') => Option::<BulkString>::expect_length(buf),
            Some(b'+') => SimpleString::expect_length(buf),
            _ => Err(RespError::NotComplete),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn respv2_should_work() {
        let s: &str = "*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n*1\r\n+OK\r\n*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n*4\r\n$4\r\nHSET\r\n$3\r\nkey\r\n$5\r\nfield\r\n$5\r\nvalue\r\n*1\r\n-ERR\r\n*3\r\n$4\r\nHGET\r\n$3\r\nkey\r\n$5\r\nfield\r\n$5\r\nvalue\r\n*3\r\n$4\r\nSADD\r\n$3\r\nkey\r\n$6\r\nmember\r\n:1\r\n";

        let mut buf = BytesMut::from(s);

        let resp = RespFrame::decode(&mut buf);
        assert!(resp.is_ok());
    }
}
