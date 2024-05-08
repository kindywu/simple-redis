use crate::{Backend, CommandError, CommandExecutor, RespArray, RespFrame};

use super::{extract_args, validate_command};

#[derive(Debug)]
pub struct HSet {
    key: String,
    field: String,
    value: RespFrame,
}

impl CommandExecutor for HSet {
    fn execute(self, backend: &Backend) -> RespFrame {
        let hmap = backend.hmap.entry(self.key).or_default();
        match hmap.insert(self.field, self.value) {
            Some(_) => 0.into(), //update
            None => 1.into(),    //insert
        }
    }
}

impl TryFrom<RespArray> for HSet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hset"], 3, super::ArgsCheckRule::Equal)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next(), args.next()) {
            (
                Some(RespFrame::BulkString(Some(key))),
                Some(RespFrame::BulkString(Some(field))),
                Some(value),
            ) => Ok(HSet {
                key: String::from_utf8(key.0)?,
                field: String::from_utf8(field.0)?,
                value,
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid key, field or value".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Backend, BulkString, Command, RespArray, RespFrame};

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_hset() -> Result<()> {
        let frame: RespFrame = Some(RespArray::new(vec![
            Some(BulkString::new("hset".to_string())).into(),
            Some(BulkString::new("myhash".to_string())).into(),
            Some(BulkString::new("field1".to_string())).into(),
            Some(BulkString::new("value".to_string())).into(),
        ]))
        .into();

        let sadd = Command::try_from(frame)?;
        let ret = sadd.execute(&Backend::new());

        assert_eq!(ret, 1.into());
        Ok(())
    }
}
