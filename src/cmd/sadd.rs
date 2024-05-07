use crate::{CommandError, CommandExecutor, RespArray, RespFrame};

use super::{extract_args, validate_command};

#[derive(Debug)]
pub struct SAdd {
    pub key: String,
    pub members: RespArray,
}

impl CommandExecutor for SAdd {
    fn execute(self) -> RespFrame {
        println!("{:?}", self);
        let len = self.members.len() as i64;
        len.into()
    }
}

impl TryFrom<RespArray> for SAdd {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["sadd"], 2, super::ArgsCheckRule::EqualOrGreater)?;

        let mut args = extract_args(value, 1)?.into_iter();
        let key = match args.next() {
            Some(RespFrame::BulkString(Some(key))) => Ok(String::from_utf8(key.0)?),
            _ => Err(CommandError::InvalidArgument("Invalid sadd".to_string())),
        }?;

        let mut members = RespArray::new(vec![]);
        for arg in args {
            members.push(arg)
        }

        Ok(SAdd { key, members })
    }
}

#[cfg(test)]
mod tests {
    use crate::{BulkString, Command, RespArray, RespFrame};

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_sadd() -> Result<()> {
        let frame: RespFrame = Some(RespArray::new(vec![
            Some(BulkString::new("sadd".to_string())).into(),
            Some(BulkString::new("myset".to_string())).into(),
            Some(BulkString::new("A".to_string())).into(),
            Some(BulkString::new("B".to_string())).into(),
            Some(BulkString::new("C".to_string())).into(),
        ]))
        .into();

        let sadd = Command::try_from(frame)?;
        let ret = sadd.execute();
        assert_eq!(ret, 3.into());
        Ok(())
    }
}
