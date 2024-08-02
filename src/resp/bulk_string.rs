use std::{fmt, ops::Deref};

use bytes::{Buf, BytesMut};

use super::{parse_length, RespDecode, RespEncode, RespError, CRLF_LEN};

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
// pub struct RespNullBulkString;

// // - null bulk string: "$-1\r\n"
// impl RespEncode for RespNullBulkString {
//     fn encode(self) -> Vec<u8> {
//         b"$-1\r\n".to_vec()
//     }
// }

// impl RespDecode for RespNullBulkString {
//     const PREFIX: &'static str = "$";
//     fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
//         extract_fixed_data(buf, "$-1\r\n", "NullBulkString")?;
//         Ok(RespNullBulkString)
//     }

//     fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
//         Ok(5)
//     }
// }

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd)]
pub struct BulkString(pub(crate) Vec<u8>);

// pub struct BulkString(pub(crate) Option<Vec<u8>>);

impl fmt::Display for BulkString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 使用 from_utf8 检查 Vec<u8> 是否是有效的 UTF-8 序列
        let str_slice = std::str::from_utf8(&self.0);
        match str_slice {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "Invalid UTF-8 sequence"),
        }
    }
}

//
const NULL_BULK_STRING: &[u8; 5] = b"$-1\r\n";

// - bulk string: "$<length>\r\n<data>\r\n"
impl RespEncode for Option<BulkString> {
    fn encode(self) -> Vec<u8> {
        if let Some(s) = self {
            let mut buf = Vec::with_capacity(s.len() + 16);
            buf.extend_from_slice(&format!("${}\r\n", s.len()).into_bytes());
            buf.extend_from_slice(&s);
            buf.extend_from_slice(b"\r\n");
            buf
        } else {
            NULL_BULK_STRING.to_vec()
        }
    }
}

impl RespDecode for Option<BulkString> {
    const PREFIX: &'static str = "$";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        if buf.starts_with(NULL_BULK_STRING) {
            buf.advance(NULL_BULK_STRING.len());
            Ok(None)
        } else {
            let (end, len) = parse_length(buf, Self::PREFIX)?;
            let len = len as usize;
            let remained = &buf[end + CRLF_LEN..];
            if remained.len() < len + CRLF_LEN {
                return Err(RespError::NotComplete);
            }

            buf.advance(end + CRLF_LEN);

            let data = buf.split_to(len + CRLF_LEN);
            Ok(Some(BulkString::new(data[..len].to_vec())))
        }
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        if len == -1 {
            Ok(5)
        } else {
            let len = len as usize;
            Ok(end + CRLF_LEN + len + CRLF_LEN)
        }
    }
}

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}

impl AsRef<[u8]> for BulkString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for BulkString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for BulkString {
    fn from(s: &str) -> Self {
        BulkString(s.as_bytes().to_vec())
    }
}

impl From<String> for BulkString {
    fn from(s: String) -> Self {
        BulkString(s.into_bytes())
    }
}

impl From<&[u8]> for BulkString {
    fn from(s: &[u8]) -> Self {
        BulkString(s.to_vec())
    }
}

impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_bulk_string_encode() {
        let frame: RespFrame = Some(BulkString::new(b"hello".to_vec())).into();
        assert_eq!(frame.encode(), b"$5\r\nhello\r\n");
    }

    #[test]
    fn test_bulk_string_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$5\r\nhello\r\n");

        assert_eq!(Option::<BulkString>::expect_length(&buf), Ok(11));

        let frame = Option::<BulkString>::decode(&mut buf)?;
        assert_eq!(frame, Some(BulkString::new(b"hello")));

        buf.extend_from_slice(b"$5\r\nhello");
        let ret = Option::<BulkString>::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"\r\n");
        let frame = Option::<BulkString>::decode(&mut buf)?;
        assert_eq!(frame, Some(BulkString::new(b"hello")));

        Ok(())
    }

    #[test]
    fn test_null_bulk_string_encode() {
        let frame: RespFrame = RespFrame::BulkString(None);
        assert_eq!(frame.encode(), b"$-1\r\n");
    }

    #[test]
    fn test_null_bulk_string_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$-1\r\n");

        assert_eq!(Option::<BulkString>::expect_length(&buf), Ok(5));

        let frame = Option::<BulkString>::decode(&mut buf)?;
        assert_eq!(frame, None);

        Ok(())
    }

    #[test]
    fn test_zero_bulk_string_encode() {
        let frame: RespFrame = Some(BulkString::new(Vec::new())).into();
        assert_eq!(frame.encode(), b"$0\r\n\r\n");
    }

    #[test]
    fn test_zero_bulk_string_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$0\r\n\r\n");

        assert_eq!(Option::<BulkString>::expect_length(&buf), Ok(6));

        let frame = Option::<BulkString>::decode(&mut buf)?;
        assert_eq!(frame, Some(BulkString::new(Vec::new())));

        Ok(())
    }
}
