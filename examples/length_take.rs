use winnow::binary::be_u16;
use winnow::binary::length_take;
use winnow::error::ErrMode;
use winnow::error::Needed;
use winnow::Bytes;
use winnow::IResult;
use winnow::Parser;
use winnow::Partial;

fn main() {
    assert_eq!(
        parser(stream(b"\x00\x03abcefg")),
        Ok((stream(&b"efg"[..]), &b"abc"[..]))
    );
    assert_eq!(
        parser(stream(b"\x00\x03a")),
        Err(ErrMode::Incomplete(Needed::new(2)))
    );
}

type Stream<'i> = Partial<&'i Bytes>;

fn stream(b: &[u8]) -> Stream<'_> {
    Partial::new(Bytes::new(b))
}

fn parser(s: Stream<'_>) -> IResult<Stream<'_>, &[u8]> {
    length_take(be_u16).parse_peek(s)
}
