use bytes::BytesMut;
use enum_dispatch::enum_dispatch;
use thiserror::Error;

use crate::{BulkString, RespArray, RespNull, SimpleError, SimpleString};

#[enum_dispatch(RespEncode)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RespFrame {
    Null(RespNull),
    SimpleString(SimpleString),
    Error(SimpleError),
    Integer(i64),
    BulkString(Option<BulkString>),
    Array(Option<RespArray>),
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

#[enum_dispatch(RespEncode)]
pub trait RespEncode {
    fn encode(self) -> Vec<u8>;
}

// #[enum_dispatch(RespDecode)]
pub trait RespDecode: Sized {
    const PREFIX: &'static str;
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError>;
    fn expect_length(buf: &[u8]) -> Result<usize, RespError>;
}

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

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RespError {
    #[error("Invalid frame: {0}")]
    InvalidFrame(String),
    #[error("Invalid frame type: {0}")]
    InvalidFrameType(String),
    #[error("Invalid frame length： {0}")]
    InvalidFrameLength(isize),
    #[error("Frame is not complete")]
    NotComplete,
    #[error("Parse error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Parse float error: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}
