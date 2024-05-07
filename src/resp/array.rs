use std::ops::{Deref, DerefMut};

use bytes::{Buf, BytesMut};

use crate::{RespDecode, RespEncode, RespError, RespFrame};

use super::{calc_total_length, parse_length, CRLF_LEN};

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd)]
pub struct RespArray(pub(crate) Vec<RespFrame>);

// const BUF_CAP: usize = 4096;
const NULL_ARRAY_STRING: &[u8; 5] = b"*-1\r\n";

// - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
impl RespEncode for Option<RespArray> {
    fn encode(self) -> Vec<u8> {
        if let Some(s) = self {
            // let mut buf = Vec::with_capacity(BUF_CAP);
            let mut buf = Vec::new();
            buf.extend_from_slice(&format!("*{}\r\n", s.0.len()).into_bytes());
            for frame in s.0 {
                buf.extend_from_slice(&frame.encode());
            }
            buf
        } else {
            NULL_ARRAY_STRING.to_vec()
        }
    }
}

// - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
// - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
impl RespDecode for Option<RespArray> {
    const PREFIX: &'static str = "*";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;

        if len == -1 {
            Ok(None)
        } else {
            let len = len as usize;
            let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;

            // println!("len={},total_len={}", len, total_len);

            if buf.len() < total_len {
                return Err(RespError::NotComplete);
            }

            buf.advance(end + CRLF_LEN);

            let mut frames = Vec::with_capacity(len);
            for _ in 0..len {
                frames.push(RespFrame::decode(buf)?);
            }

            Ok(Some(RespArray::new(frames)))
        }
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        if len == -1 {
            Ok(5)
        } else {
            let len = len as usize;
            calc_total_length(buf, end, len, Self::PREFIX)
        }
    }
}

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
// pub struct RespNullArray;

// // - null array: "*-1\r\n"
// impl RespEncode for RespNullArray {
//     fn encode(self) -> Vec<u8> {
//         b"*-1\r\n".to_vec()
//     }
// }

// impl RespDecode for RespNullArray {
//     const PREFIX: &'static str = "*";
//     fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
//         extract_fixed_data(buf, "*-1\r\n", "NullArray")?;
//         Ok(RespNullArray)
//     }

//     fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
//         Ok(4)
//     }
// }

impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(s.into())
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RespArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BulkString;
    use anyhow::Result;

    #[test]
    fn test_array_encode() {
        let frame: RespFrame = Some(RespArray::new(vec![
            Some(BulkString::new("set".to_string())).into(),
            Some(BulkString::new("hello".to_string())).into(),
            Some(BulkString::new("world".to_string())).into(),
        ]))
        .into();
        assert_eq!(
            &frame.encode(),
            b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n"
        );
    }

    #[test]
    fn test_array_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$4\r\necho\r\n$5\r\nhello\r\n");

        assert_eq!(Option::<RespArray>::expect_length(&buf), Ok(25));

        let frame = Option::<RespArray>::decode(&mut buf)?;
        assert_eq!(
            frame,
            Some(RespArray::new([b"echo".into(), b"hello".into()]))
        );

        buf.extend_from_slice(b"*2\r\n$4\r\necho\r\n");
        assert_eq!(
            Option::<RespArray>::expect_length(&buf),
            Err(RespError::NotComplete)
        );
        let ret = Option::<RespArray>::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"$5\r\nhello\r\n");
        assert_eq!(Option::<RespArray>::expect_length(&buf), Ok(25));
        let frame = Option::<RespArray>::decode(&mut buf)?;
        assert_eq!(
            frame,
            Some(RespArray::new([b"echo".into(), b"hello".into()]))
        );

        Ok(())
    }

    #[test]
    fn test_null_array_encode() {
        let frame: RespFrame = RespFrame::Array(None);
        assert_eq!(frame.encode(), b"*-1\r\n");
    }

    #[test]
    fn test_null_array_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*-1\r\n");

        assert_eq!(Option::<RespArray>::expect_length(&buf), Ok(5));

        let frame = Option::<RespArray>::decode(&mut buf)?;
        assert_eq!(frame, None);

        Ok(())
    }

    #[test]
    fn test_zero_array_encode() {
        let frame: RespFrame = Some(RespArray::new(Vec::new())).into();
        assert_eq!(frame.encode(), b"*0\r\n");
    }

    #[test]
    fn test_zero_array_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*0\r\n");

        assert_eq!(Option::<RespArray>::expect_length(&buf), Ok(4));

        let frame = Option::<RespArray>::decode(&mut buf)?;
        assert_eq!(frame, Some(RespArray::new(Vec::new())));

        Ok(())
    }
}
