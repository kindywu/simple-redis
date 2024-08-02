use winnow::{
    error::{ErrMode, ErrorKind, InputError},
    Parser,
};

fn main() {
    use winnow::combinator::terminated;

    let mut parser = terminated("abc", "efg");

    assert_eq!(parser.parse_peek("abcefg"), Ok(("", "abc")));
    assert_eq!(parser.parse_peek("abcefghij"), Ok(("hij", "abc")));
    assert_eq!(
        parser.parse_peek(""),
        Err(ErrMode::Backtrack(InputError::new("", ErrorKind::Tag)))
    );
    assert_eq!(
        parser.parse_peek("123"),
        Err(ErrMode::Backtrack(InputError::new("123", ErrorKind::Tag)))
    );
}
