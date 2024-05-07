use crate::{Backend, CommandExecutor, RespFrame, SimpleString};

#[derive(Debug)]
pub struct Unrecognized;
impl CommandExecutor for Unrecognized {
    fn execute(self, _: &Backend) -> RespFrame {
        SimpleString("OK".to_string()).into()
    }
}
