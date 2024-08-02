use simple_redis::RespError;
use winnow::{
    ascii::dec_int,
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
    let buf = b"*2\r\n$4\r\necho\r\n$5\r\nhello\r\n";
    expect_length(buf).unwrap();
}

fn expect_length(input: &[u8]) -> Result<usize, RespError> {
    match expect_length_inner(input) {
        Ok(v) => {
            println!("input:{}, remain: {:?}, len:{}", input.len(), v.0, v.1 + 1);
            Ok(v.1 + 1)
        }
        Err(_) => Err(RespError::NotComplete),
    }
}

fn expect_length_inner(input: &[u8]) -> PResult<(&[u8], usize)> {
    Ok(match take(1usize).parse_peek(input)? {
        (i, b"+") => cal_by_crlf(i)?,
        (i, b"$") => cal_by_len(i)?,
        (i, b"*") => array_length(i)?,
        (i, p) => {
            println!("{:?} {i:?}", p[0].as_char());
            unreachable!()
        }
    })
}

fn cal_by_crlf(input: &[u8]) -> PResult<(&[u8], usize)> {
    let (remain, found) = terminated(take_until(0.., CRLF), CRLF).parse_peek(input)?;
    Ok((remain, found.len() + 2))
}

fn cal_by_len(input: &[u8]) -> PResult<(&[u8], usize)> {
    let (remain, len): (&[u8], i64) = dec_int.parse_peek(input)?;

    if len <= 0 {
        let (remain, size) = cal_by_crlf(remain)?;
        let mut total = if len == 0 { 1 } else { 2 };
        total += size;
        Ok((remain, total))
    } else {
        let len = len as usize;
        let mut total = len / 10 + 1;
        let (remain, size) = cal_by_crlf(remain)?;
        total += size;
        let (remain, found) = take(len).parse_peek(remain)?;
        total += found.len();
        let (remain, size) = cal_by_crlf(remain)?;
        total += size;
        Ok((remain, total))
    }
}

fn array_length(input: &[u8]) -> PResult<(&[u8], usize)> {
    let (remain, len): (&[u8], i64) = dec_int.parse_peek(input.as_ref())?;

    if len <= 0 {
        let (remain, size) = cal_by_crlf(remain)?;
        let mut total = if len == 0 { 1 } else { 2 };
        total += size;
        Ok((remain, total))
    } else {
        let mut len = len as usize;
        let mut total = len / 10 + 1;

        let remain = loop {
            let (remain, size) = cal_by_crlf(remain)?;
            total += size;
            let (remain, size) = expect_length_inner(remain)?;
            total += size;
            len -= 1;
            if len == 0 {
                break remain;
            }
        };

        Ok((remain, total))
    }
}
