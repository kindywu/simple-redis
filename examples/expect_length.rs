use simple_redis::RespError;
use winnow::{
    ascii::{crlf, dec_int},
    combinator::terminated,
    stream::AsChar,
    token::{take, take_until},
    PResult, Parser,
};

const CRLF: &[u8] = b"\r\n";

fn main() {
    // let buf = b"+OK\r\n";
    // expect_length(buf).unwrap();
    // let buf = b"+OK\r";
    // println!("{:?}", expect_length(buf));

    // let buf = b"$5\r\nhello\r\n";
    // expect_length(buf).unwrap();

    // let buf = b"$-1\r\n";
    // expect_length(buf).unwrap();
    // let buf = b"$0\r\n";
    // expect_length(buf).unwrap();
    // let buf = b"$5\r\nhello\r\n";
    // expect_length(buf).unwrap();
    // let buf = b"$5\r\nhello\r";
    // println!("{:?}", expect_length(buf));

    // let buf = b"*-1\r\n";
    // expect_length(buf).unwrap();
    // let buf = b"*0\r\n";
    // expect_length(buf).unwrap();
    // let buf = b"*2\r\n$4\r\necho\r\n$5\r\nhello\r\n";
    // expect_length(buf).unwrap();

    let buf = b"%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n";
    expect_length(buf).unwrap();
}

fn expect_length(input: &[u8]) -> Result<usize, RespError> {
    match expect_length_inner(input) {
        Ok(v) => {
            println!("input:{}, remain: {:?}, len:{}", input.len(), v.0, v.1);
            Ok(v.1)
        }
        Err(_) => Err(RespError::NotComplete),
    }
}

fn expect_length_inner(input: &[u8]) -> PResult<(&[u8], usize)> {
    let (remain, len) = match take(1usize).parse_peek(input)? {
        (i, b"+") => cal_util_crlf(i)?,
        (i, b"$") => cal_by_len(i)?,
        (i, b"*") => cal_array(i)?,
        (i, b"%") => cal_map(i)?,
        (i, p) => {
            println!("{:?} {i:?}", p[0].as_char());
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
mod test {
    use super::*;

    #[test]
    fn cal_crlf_should_work() -> PResult<()> {
        let buf = b"\r\n";
        let result = cal_crlf(buf)?;
        assert_eq!(buf.len(), result.1);
        Ok(())
    }

    #[test]
    fn cal_util_crlf_should_work() -> PResult<()> {
        let buf = b"OK\r\n";
        let result = cal_util_crlf(buf)?;
        assert_eq!(buf.len(), result.1);
        Ok(())
    }

    #[test]
    fn cal_by_len_should_work() -> PResult<()> {
        let buf = b"0\r\n\r\n";
        let result = cal_by_len(buf)?;
        assert_eq!(buf.len(), result.1);

        let buf = b"-1\r\n";
        let result = cal_by_len(buf)?;
        assert_eq!(buf.len(), result.1);

        let buf = b"5\r\nhello\r\n";
        let result = cal_by_len(buf)?;
        assert_eq!(buf.len(), result.1);
        Ok(())
    }

    #[test]
    fn cal_array_should_work() -> PResult<()> {
        let buf = b"-1\r\n";
        let result = cal_array(buf)?;
        assert_eq!(buf.len(), result.1);

        let buf = b"0\r\n";
        let result = cal_array(buf)?;
        assert_eq!(buf.len(), result.1);

        let buf = b"2\r\n$4\r\necho\r\n$5\r\nhello\r\n";
        let result = cal_array(buf)?;
        assert_eq!(buf.len(), result.1);
        Ok(())
    }

    #[test]
    fn cal_map_should_work() -> PResult<()> {
        let buf = b"2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n*2\r\n$4\r\necho\r\n$5\r\nhello\r\n";
        let result = cal_map(buf)?;
        assert_eq!(buf.len(), result.1);
        Ok(())
    }
}
