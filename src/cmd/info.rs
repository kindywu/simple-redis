use crate::{Backend, CommandError, CommandExecutor, RespArray, RespFrame};

use super::validate_command;

#[derive(Debug)]
pub struct Info();

impl CommandExecutor for Info {
    fn execute(self, _: &Backend) -> RespFrame {
        "Ok".into()
    }
}

impl Info {
    pub fn new() -> Self {
        Self()
    }
}

impl Default for Info {
    fn default() -> Self {
        Self::new()
    }
}

impl TryFrom<RespArray> for Info {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["info"], 0, super::ArgsCheckRule::Equal)?;

        Ok(Info::new())
    }
}
