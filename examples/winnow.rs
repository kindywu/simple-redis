use winnow::{combinator::terminated, token::take_until, PResult, Parser};

fn main() -> PResult<()> {
    let s = b"OK\r\n";
    let ret = parse(&mut s.as_ref())?;
    println!("{ret}");

    let s = b"OK\r\n";
    let ret = parse_v2(s.as_ref());
    println!("{:?}", ret);
    Ok(())
}

const CRLF: &[u8] = b"\r\n";

#[allow(unused)]
pub fn parse(input: &mut &[u8]) -> PResult<String> {
    terminated(take_until(0.., CRLF), CRLF)
        .map(|v| String::from_utf8_lossy(v).to_string())
        .parse_next(input)
}

#[allow(unused)]
pub fn parse_v2(mut input: &[u8]) -> PResult<String> {
    let mut parser = take_until(0.., CRLF);
    let ret = parser.parse_peek(input)?;

    terminated(parser, CRLF)
        .map(|v| String::from_utf8_lossy(v).to_string())
        .parse_next(&mut input)
}
