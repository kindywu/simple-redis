use crate::{Backend, CommandExecutor, RespFrame};

#[derive(Debug)]
pub struct Unrecognized;
impl CommandExecutor for Unrecognized {
    fn execute(self, _: &Backend) -> RespFrame {
        "OK".into()
    }
}
