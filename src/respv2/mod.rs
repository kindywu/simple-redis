use crate::{BulkString, RespArray, RespError, RespFrame, RespNull, SimpleError, SimpleString};
use bytes::BytesMut;
use winnow::ascii::{crlf, dec_int};
use winnow::combinator::{alt, dispatch, fail, terminated};
use winnow::token::{any, take};
use winnow::{token::take_until, PResult, Parser};

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

const CRLF: &[u8] = b"\r\n";

fn parse_resp(input: &mut &[u8]) -> PResult<RespFrame> {
    // match take(1usize).parse_next(input)? {
    //     b"+" => simple_string(input).map(RespFrame::SimpleString),
    //     b"-" => simple_error(input).map(RespFrame::Error),
    //     _ => todo!(),
    // }

    dispatch! {any;
        b'+' => simple_string.map(RespFrame::SimpleString),
        b'-' => error.map(RespFrame::Error),
        b'_' => null.map(RespFrame::Null),
        b':' => integer.map(RespFrame::Integer),
        b'*' => array.map(RespFrame::Array),
        b'$' => bulk_string.map(RespFrame::BulkString),
        b'#' => boolean.map(RespFrame::Boolean),
        _ => fail::<_, _, _>,
    }
    .parse_next(input)
}

fn parse_length(input: &mut &[u8]) -> PResult<()> {
    let mut simple_parse = terminated(take_until(0.., CRLF), CRLF).value(());
    dispatch! {any;
        b'+' => simple_parse,
        b'-' => simple_parse,
        b'_' => simple_parse,
        b':' => simple_parse,
        b'#' => simple_parse,
        b'*' => array_length,
        b'$' => bulk_string_length,
        _ => fail::<_, _, _>,
    }
    .parse_next(input)
}

fn simple_string(input: &mut &[u8]) -> PResult<SimpleString> {
    Ok(SimpleString::new(parse_string(input)?))
}

fn error(input: &mut &[u8]) -> PResult<SimpleError> {
    Ok(SimpleError::new(parse_string(input)?))
}

// - boolean: "#t\r\n"
fn boolean(input: &mut &[u8]) -> PResult<bool> {
    let b = alt(('t', 'f')).parse_next(input)?;
    Ok(b == 't')
}

// _\r\n
fn null(input: &mut &[u8]) -> PResult<RespNull> {
    crlf(input)?;
    Ok(RespNull)
}

// :[<+|->]<value>\r\n
fn integer(input: &mut &[u8]) -> PResult<i64> {
    dec_int(input)
}

// $5\r\nhello\r\n
// $0\r\n\r\n
// $-1\r\n
fn bulk_string(input: &mut &[u8]) -> PResult<Option<BulkString>> {
    let len: i64 = dec_int(input)?;
    crlf(input)?;
    if len == -1 {
        return Ok(None);
    }
    Ok(Some(BulkString::new(parse_string(input)?)))
}

fn bulk_string_length(input: &mut &[u8]) -> PResult<()> {
    let len: i64 = dec_int(input)?;
    crlf(input)?;
    if len == -1 {
    } else if len == 0 {
        crlf(input)?;
    } else {
        take(len as usize).value(()).parse_next(input)?;
        crlf(input)?;
    }

    Ok(())
    // if len > -1 {
    //     crlf(input)?;
    // }
    // terminated(take_until(0.., CRLF), CRLF)
    //     .value(())
    //     .parse_next(input)
}

// *3\r\n$4\r\necho\r\n$5\r\nhello\r\n+OK\r\n
// *0\r\n
// *-1\r\n
fn array(input: &mut &[u8]) -> PResult<Option<RespArray>> {
    let len: i64 = dec_int(input)?;
    crlf(input)?;
    if len == -1 {
        return Ok(None);
    }
    let mut arr = Vec::new();
    for _ in 0..len {
        arr.push(parse_resp(input)?);
    }

    Ok(Some(RespArray::new(arr)))
}

fn array_length(input: &mut &[u8]) -> PResult<()> {
    let len: i64 = dec_int(input)?;
    crlf(input)?;
    if len > 0 {
        for _ in 0..len {
            parse_length(input)?
        }
    }

    Ok(())
}

fn parse_string(input: &mut &[u8]) -> PResult<String> {
    terminated(take_until(0.., CRLF), CRLF)
        .map(|v| String::from_utf8_lossy(v).to_string())
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn respv2_simple_string_should_work() {
        let s = b"+OK\r\n";
        let resp = parse_resp(&mut s.as_ref()).unwrap();
        assert_eq!(RespFrame::SimpleString(SimpleString::new("OK")), resp)
    }

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
