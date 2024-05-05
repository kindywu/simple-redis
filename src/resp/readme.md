# RESP Protocol

- simple string: "+OK\r\n"
- error: "-Error message\r\n"
- bulk string: "$<length>\r\n<data>\r\n"
- bulk error: "!<length>\r\n<error>\r\n"
- integer: ":[<+|->]<value>\r\n"
- null bulk string: "$-1\r\n"
- array: "\*<number-of-elements>\r\n<element-1>...<element-n>"
- "\*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
- null array: "\*-1\r\n"
- null: "\_\r\n"
- boolean: "#<t|f>\r\n"
- double: ",[<+|->]<integral>[.<fractional>]<E|e>[sign]<exponent>]\r\n"
- big number: "([+|-]<number>\r\n"
- map: "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
- set: "~<number-of-elements>\r\n<element-1>...<element-n>"

# enum_dispatch

<code>

#[enum_dispatch(RespEncode)] #[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RespFrame {
SimpleString(SimpleString),
}

#[enum_dispatch(RespEncode)]
pub trait RespEncode {
fn encode(self) -> Vec<u8>;
}

</code>

- `enum_dispatch` 为 `RespFrame` 自动实现 `RespEncode trait`，前提是 `SimpleString` 实现了 `RespEncode trait`.
- `enum_dispatch` 为 `SimpleString` 自动实现 `impl Into<RespFrame>` for SimpleString
