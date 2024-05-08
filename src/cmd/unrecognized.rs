use crate::{Backend, BulkString, CommandExecutor, RespFrame, SimpleError};

#[derive(Debug)]
pub struct Unrecognized(pub(crate) BulkString);

impl CommandExecutor for Unrecognized {
    fn execute(self, _: &Backend) -> RespFrame {
        SimpleError::new(format!(
            "ERR unknown command '{}', with args beginning with:",
            self.0
        ))
        .into()
    }
}

impl Unrecognized {
    pub fn new(cmd: BulkString) -> Self {
        Self(cmd)
    }
}
