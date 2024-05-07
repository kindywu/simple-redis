use crate::{CommandError, CommandExecutor, RespArray, RespFrame};

use super::{extract_args, validate_command};

#[derive(Debug)]
pub struct Echo {
    pub echo: RespFrame,
}

impl CommandExecutor for Echo {
    fn execute(self) -> RespFrame {
        self.echo
    }
}

impl TryFrom<RespArray> for Echo {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["echo"], 1, super::ArgsCheckRule::Equal)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(echo) => Ok(Echo { echo }),
            _ => Err(CommandError::InvalidArgument("Invalid echo".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BulkString, Command, RespArray, RespFrame};

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_echo() -> Result<()> {
        let frame: RespFrame = Some(RespArray::new(vec![
            Some(BulkString::new("echo".to_string())).into(),
            Some(BulkString::new("hello".to_string())).into(),
        ]))
        .into();

        let echo = Command::try_from(frame)?;
        let ret = echo.execute();
        assert_eq!(ret, Some(BulkString::new("hello".to_string())).into());
        Ok(())
    }
}
