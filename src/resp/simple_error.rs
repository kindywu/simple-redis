use bytes::BytesMut;
use std::ops::Deref;

use anyhow::Result;

use super::{extract_simple_frame_data, RespDecode, RespEncode, RespError, CRLF_LEN};
// use crate::RespFrame;
#[derive(Debug, Clone, Hash, Ord, PartialEq, Eq, PartialOrd)]
pub struct SimpleError(pub(crate) String);

impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}
impl From<&str> for SimpleError {
    fn from(s: &str) -> Self {
        SimpleError(s.to_string())
    }
}

impl AsRef<str> for SimpleError {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for SimpleError {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RespEncode for SimpleError {
    fn encode(self) -> Vec<u8> {
        format!("-{}\r\n", self.0).into_bytes()
    }
}

impl RespDecode for SimpleError {
    const PREFIX: &'static str = "-";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        // split the buffer
        let data = buf.split_to(end + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[Self::PREFIX.len()..end]);
        Ok(SimpleError::new(s.to_string()))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RespFrame;
    use anyhow::Result;
    use bytes::BufMut;

    #[test]
    fn test_simple_error_encode() {
        // let frame: RespFrame = RespFrame::SimpleString(SimpleString::new("OK".to_string()));
        let frame: RespFrame = SimpleError::new("ERROR".to_string()).into();
        assert_eq!(frame.encode(), b"-ERROR\r\n");
    }

    #[test]
    fn test_simple_error_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"-ERROR\r\n");

        let frame = SimpleError::decode(&mut buf)?;
        assert_eq!(frame, SimpleError::new("ERROR".to_string()));

        buf.extend_from_slice(b"-hello\r");

        let ret = SimpleError::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.put_u8(b'\n');
        let frame = SimpleError::decode(&mut buf)?;
        assert_eq!(frame, SimpleError::new("hello".to_string()));

        Ok(())
    }
}
