use std::collections::BTreeMap;

use crate::resp::Double;
use crate::{BulkString, RespArray, RespFrame, RespNull, SimpleError, SimpleString};
use winnow::ascii::{crlf, dec_int, digit1, float};
use winnow::combinator::{alt, dispatch, fail, terminated};
use winnow::token::{any, take};
use winnow::{token::take_until, PResult, Parser};

const CRLF: &[u8] = b"\r\n";

pub fn parse_resp(input: &mut &[u8]) -> PResult<RespFrame> {
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
        b',' => double.map(RespFrame::Double),
        b'%' => map.map(RespFrame::Map),
        _ => fail::<_, _, _>,
    }
    .parse_next(input)
}

pub fn parse_length(input: &mut &[u8]) -> PResult<()> {
    let mut simple_parse = terminated(take_until(0.., CRLF), CRLF).value(());
    dispatch! {any;
        b'+' => simple_parse,
        b'-' => simple_parse,
        b'_' => simple_parse,
        b':' => simple_parse,
        b'#' => simple_parse,
        b',' => simple_parse,
        b'*' => array_length,
        b'%' => map_length,
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

// - float: ",3.14\r\n"
fn double(input: &mut &[u8]) -> PResult<Double> {
    terminated(float, CRLF).map(Double).parse_next(input)
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

// "%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n"
fn map(input: &mut &[u8]) -> PResult<BTreeMap<RespFrame, RespFrame>> {
    let len: u32 = digit1.parse_to().parse_next(input)?;
    crlf(input)?;
    let mut map = BTreeMap::new();
    for _ in 0..len {
        let key = parse_resp(input)?;
        let value = parse_resp(input)?;
        map.insert(key, value);
    }
    Ok(map)
}

fn map_length(input: &mut &[u8]) -> PResult<()> {
    let len: u32 = digit1.parse_to().parse_next(input)?;
    crlf(input)?;
    for _ in 0..len {
        let _ = parse_resp(input)?;
        let _ = parse_resp(input)?;
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
    fn respv2_simple_string_should_fail() {
        let s = b"+OK\r";
        let resp = parse_resp(&mut s.as_ref());
        assert!(resp.is_err());
    }

    #[test]
    fn respv2_map_should_work() {
        let s = b"%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n";
        let resp = parse_resp(&mut s.as_ref());
        let mut map = BTreeMap::new();
        map.insert(
            SimpleString::new("hello").into(),
            Some(BulkString::new("world")).into(),
        );
        map.insert(
            SimpleString::new("foo").into(),
            Some(BulkString::new("bar")).into(),
        );
        assert!(resp.is_ok());
        assert_eq!(RespFrame::Map(map), resp.unwrap());

        let s = b"%0\r\n";
        let resp = parse_resp(&mut s.as_ref());
        assert!(resp.is_ok());
        assert_eq!(RespFrame::Map(BTreeMap::new()), resp.unwrap())
    }
}
