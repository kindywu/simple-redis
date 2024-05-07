use crate::{CommandExecutor, RespFrame, SimpleString};

#[derive(Debug)]
pub struct Unrecognized;
impl CommandExecutor for Unrecognized {
    fn execute(self) -> RespFrame {
        SimpleString("OK".to_string()).into()
    }
}
