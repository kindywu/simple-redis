use crate::{CommandError, CommandExecutor, RespArray, RespFrame};

use super::{extract_args, validate_command};

#[derive(Debug)]
pub struct SisMember {
    pub key: String,
    pub member: RespFrame,
}

impl CommandExecutor for SisMember {
    fn execute(self) -> RespFrame {
        println!("{:?}", self);
        1.into()
    }
}

impl TryFrom<RespArray> for SisMember {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(
            &value,
            &["sismember"],
            2,
            super::ArgsCheckRule::EqualOrGreater,
        )?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(Some(key))), Some(member)) => Ok(SisMember {
                key: String::from_utf8(key.0)?,
                member,
            }),
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
    fn test_sismember() -> Result<()> {
        let frame: RespFrame = Some(RespArray::new(vec![
            Some(BulkString::new("sismember".to_string())).into(),
            Some(BulkString::new("myset".to_string())).into(),
            Some(BulkString::new("A".to_string())).into(),
        ]))
        .into();

        let sis_member = Command::try_from(frame)?;
        let ret = sis_member.execute();
        assert_eq!(ret, 1.into());
        Ok(())
    }
}
