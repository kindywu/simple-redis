use crate::{RespError, RespFrame};
use bytes::BytesMut;

mod parse;

use parse::*;
use winnow::{
    ascii::{crlf, dec_int},
    combinator::terminated,
    token::{take, take_until},
    PResult, Parser,
};

const CRLF: &[u8] = b"\r\n";

#[allow(unused)]
pub trait RespDecodeV3: Sized {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError>;
    fn expect_length(buf: &[u8]) -> Result<usize, RespError>;
}

impl RespDecodeV3 for RespFrame {
    fn decode(buf: &mut bytes::BytesMut) -> Result<Self, crate::RespError> {
        let len = Self::expect_length(buf)?;
        let data = buf.split_to(len);
        parse_resp(&mut data.as_ref()).map_err(|e| RespError::InvalidFrame(e.to_string()))
    }

    fn expect_length(input: &[u8]) -> Result<usize, RespError> {
        match expect_length_inner(input) {
            Ok(v) => Ok(v.1),
            Err(_) => Err(RespError::NotComplete),
        }
    }
}

fn expect_length_inner(input: &[u8]) -> PResult<(&[u8], usize)> {
    let (remain, len) = match take(1usize).parse_peek(input)? {
        (i, b"+") => cal_util_crlf(i)?,
        (i, b"-") => cal_util_crlf(i)?,
        (i, b"_") => cal_util_crlf(i)?,
        (i, b":") => cal_util_crlf(i)?,
        (i, b"#") => cal_util_crlf(i)?,
        (i, b",") => cal_util_crlf(i)?,
        (i, b"$") => cal_by_len(i)?,
        (i, b"*") => cal_array(i)?,
        (i, b"%") => cal_map(i)?,
        _ => {
            unreachable!()
        }
    };
    Ok((remain, len + 1))
}

// \r\n
#[inline]
fn cal_crlf(input: &[u8]) -> PResult<(&[u8], usize)> {
    let (remain, found) = crlf.parse_peek(input)?;
    Ok((remain, found.len()))
}

// +OK\r\n
#[inline]
fn cal_util_crlf(input: &[u8]) -> PResult<(&[u8], usize)> {
    let (remain, found) = terminated(take_until(0.., CRLF), CRLF).parse_peek(input)?;
    Ok((remain, found.len() + 2))
}

#[inline]
fn cal_by_len(input: &[u8]) -> PResult<(&[u8], usize)> {
    let (remain, len): (&[u8], i64) = dec_int.parse_peek(input)?;

    if len == -1 {
        let mut total = 2;
        let (remain, size) = cal_crlf(remain)?;
        total += size;
        return Ok((remain, total));
    } else if len == 0 {
        let mut total = 1;
        let (remain, size) = cal_crlf(remain)?;
        total += size;
        let (remain, size) = cal_crlf(remain)?;
        total += size;
        return Ok((remain, total));
    }

    let len = len as usize;
    let mut total = len / 10 + 1;
    let (remain, size) = cal_crlf(remain)?;
    total += size;
    let (remain, found) = take(len).parse_peek(remain)?;
    total += found.len();
    let (remain, size) = cal_crlf(remain)?;
    total += size;
    Ok((remain, total))
}

#[inline]
fn cal_array(input: &[u8]) -> PResult<(&[u8], usize)> {
    let (remain, len): (&[u8], i64) = dec_int.parse_peek(input.as_ref())?;
    if len == 0 {
        let (remain, size) = cal_crlf(remain)?;
        return Ok((remain, size + 1));
    } else if len == -1 {
        let (remain, size) = cal_crlf(remain)?;
        return Ok((remain, size + 2));
    }

    let len = len as usize;
    let mut total = len / 10 + 1;
    let (mut r1, size) = cal_crlf(remain)?;
    total += size;

    for _ in 0..len {
        let (r2, size) = expect_length_inner(r1)?;
        total += size;
        r1 = r2;
    }

    Ok((r1, total))
}

#[inline]
fn cal_map(input: &[u8]) -> PResult<(&[u8], usize)> {
    // 2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n

    let (remain, len): (&[u8], i64) = dec_int.parse_peek(input.as_ref())?;
    if len == 0 {
        let (remain, size) = cal_crlf(remain)?;
        return Ok((remain, size + 1));
    } else if len == -1 {
        let (remain, size) = cal_crlf(remain)?;
        return Ok((remain, size + 2));
    }

    let len = len as usize;
    let mut total = len / 10 + 1;
    let (mut r1, size) = cal_crlf(remain)?;
    total += size;

    for _ in 0..len {
        let (r2, size) = expect_length_inner(r1)?;
        total += size;

        let (r3, size) = expect_length_inner(r2)?;
        total += size;
        r1 = r3;
    }

    Ok((r1, total))
}

#[cfg(test)]
mod tests {
    use crate::{BulkString, RespArray, RespNull, SimpleError, SimpleString};

    use super::*;

    #[test]
    fn respv3_should_work() {
        let s: &str = "*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n*1\r\n+OK\r\n*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n*4\r\n$4\r\nHSET\r\n$3\r\nkey\r\n$5\r\nfield\r\n$5\r\nvalue\r\n*1\r\n-ERR\r\n*3\r\n$4\r\nHGET\r\n$3\r\nkey\r\n$5\r\nfield\r\n$5\r\nvalue\r\n*3\r\n$4\r\nSADD\r\n$3\r\nkey\r\n$6\r\nmember\r\n:1\r\n";

        let mut buf = BytesMut::from(s);

        let resp = RespFrame::decode(&mut buf);
        assert!(resp.is_ok());
    }

    #[test]
    fn respv3_decode_simple_string_should_work() {
        let mut buf = BytesMut::from("+OK\r\n");
        let resp = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(RespFrame::SimpleString(SimpleString::new("OK")), resp)
    }

    #[test]
    fn respv3_decode_uncomple_simple_string_should_fail() {
        let mut buf = BytesMut::from("+OK\r");
        let resp = RespFrame::decode(&mut buf);
        assert_eq!(RespError::NotComplete, resp.unwrap_err())
    }

    #[test]
    fn respv3_decode_integer_should_work() {
        let mut buf = BytesMut::from(":10\r\n");
        let resp = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(RespFrame::Integer(10), resp);

        let mut buf = BytesMut::from(":-10\r\n");
        let resp = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(RespFrame::Integer(-10), resp)
    }

    #[test]
    fn respv3_decode_error_should_work() {
        let mut buf = BytesMut::from("-ERR\r\n");
        let resp = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(RespFrame::Error(SimpleError::new("ERR")), resp)
    }

    #[test]
    fn respv3_decode_null_should_work() {
        let mut buf = BytesMut::from("+OK\r\n_\r\n+OK\r\n");

        let resp = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(RespFrame::SimpleString(SimpleString::new("OK")), resp);

        let resp = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(RespFrame::Null(RespNull), resp);

        let resp = RespFrame::decode(&mut buf).unwrap();
        assert_eq!(RespFrame::SimpleString(SimpleString::new("OK")), resp);
    }

    #[test]
    fn respv3_decode_bulk_string_should_work() {
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
    fn respv3_decode_array_string_should_work() {
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
