use crate::{RespError, RespFrame};
use bytes::BytesMut;

mod parse;

use parse::*;

#[allow(unused)]
pub trait RespDecodeV2: Sized {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError>;
    fn expect_length(buf: &[u8]) -> Result<usize, RespError>;
}

impl RespDecodeV2 for RespFrame {
    fn decode(buf: &mut bytes::BytesMut) -> Result<Self, crate::RespError> {
        let len = Self::expect_length(buf)?;
        let data = buf.split_to(len);
        parse_resp(&mut data.as_ref()).map_err(|e| RespError::InvalidFrame(e.to_string()))
    }

    fn expect_length(input: &[u8]) -> Result<usize, crate::RespError> {
        let target = &mut (&*input);
        let ret = parse_length(target);
        match ret {
            Ok(_) => {
                let start = input.as_ptr() as usize;
                let stop = (*target).as_ptr() as usize;
                let len = stop - start;
                Ok(len)
            }
            Err(_) => Err(RespError::NotComplete),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BulkString, RespArray, RespNull, SimpleError, SimpleString};

    use super::*;

    #[test]
    fn respv2_decode_simple_string_should_work() {
        let mut buf = BytesMut::from("+OK\r\n");
        let resp = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(RespFrame::SimpleString(SimpleString::new("OK")), resp)
    }

    #[test]
    fn respv2_decode_uncomple_simple_string_should_fail() {
        let mut buf = BytesMut::from("+OK\r");
        let resp = RespFrame::decode(&mut buf);
        assert_eq!(RespError::NotComplete, resp.unwrap_err())
    }

    #[test]
    fn respv2_decode_integer_should_work() {
        let mut buf = BytesMut::from(":10\r\n");
        let resp = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(RespFrame::Integer(10), resp);

        let mut buf = BytesMut::from(":-10\r\n");
        let resp = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(RespFrame::Integer(-10), resp)
    }

    #[test]
    fn respv2_decode_error_should_work() {
        let mut buf = BytesMut::from("-ERR\r\n");
        let resp = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(RespFrame::Error(SimpleError::new("ERR")), resp)
    }

    #[test]
    fn respv2_decode_null_should_work() {
        let mut buf = BytesMut::from("+OK\r\n_\r\n+OK\r\n");

        let resp = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(RespFrame::SimpleString(SimpleString::new("OK")), resp);

        let resp = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(RespFrame::Null(RespNull), resp);

        let resp = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(RespFrame::SimpleString(SimpleString::new("OK")), resp);
    }

    #[test]
    fn respv2_decode_bulk_string_should_work() {
        let test_cases = ["$5\r\nhello\r\n", "$0\r\n\r\n", "$-1\r\n"];
        let test_expecteds = [
            RespFrame::BulkString(Some(BulkString::new("hello"))),
            RespFrame::BulkString(Some(BulkString::new(""))),
            RespFrame::BulkString(None),
        ];

        for (&test, excepted) in test_cases.iter().zip(test_expecteds) {
            let mut buf = BytesMut::from(test);
            let result = RespFrame::decode(&mut buf);
            assert!(result.is_ok());
            assert_eq!(excepted, result.unwrap());
        }
    }

    #[test]
    fn respv2_decode_array_string_should_work() {
        let test_cases = [
            "*-1\r\n",
            "*0\r\n",
            "*3\r\n$4\r\necho\r\n$5\r\nhello\r\n+OK\r\n",
        ];
        let test_expecteds = [
            RespFrame::Array(None),
            RespFrame::Array(Some(RespArray::new(vec![]))),
            RespFrame::Array(Some(RespArray::new([
                b"echo".into(),
                b"hello".into(),
                RespFrame::SimpleString(SimpleString::new("OK")),
            ]))),
        ];

        for (&test, excepted) in test_cases.iter().zip(test_expecteds) {
            let mut buf = BytesMut::from(test);
            let result = RespFrame::decode(&mut buf);
            assert!(result.is_ok());
            assert_eq!(excepted, result.unwrap());
        }
    }
}
