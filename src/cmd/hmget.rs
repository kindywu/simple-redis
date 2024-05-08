use crate::{Backend, CommandError, CommandExecutor, RespArray, RespFrame, SimpleError};

use super::{extract_args, validate_command};

#[derive(Debug)]
pub struct HmGet {
    pub key: String,
    pub members: Vec<String>,
}

impl CommandExecutor for HmGet {
    fn execute(self, backend: &Backend) -> RespFrame {
        let mut result = RespArray::new(vec![]);

        match backend.hmap.get(&self.key) {
            Some(item) => {
                let map = item.value();
                for member in self.members {
                    match map.get(&member) {
                        Some(value) => result.push(value.value().clone()),
                        None => result.push(RespFrame::BulkString(None)), //兼容RESP 2
                    }
                }
            }
            None => {
                return RespFrame::Error(SimpleError::new(format!(
                    "key {} is not exist",
                    &self.key
                )))
            }
        };

        RespFrame::Array(Some(result))
    }
}

impl TryFrom<RespArray> for HmGet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hmget"], 2, super::ArgsCheckRule::EqualOrGreater)?;

        let mut args = extract_args(value, 1)?.into_iter();
        let key = match args.next() {
            Some(RespFrame::BulkString(Some(key))) => Ok(String::from_utf8(key.0)?),
            _ => Err(CommandError::InvalidArgument("Invalid hmget".to_string())),
        }?;

        let members = args
            .filter_map(|arg| {
                if let RespFrame::BulkString(Some(key)) = arg {
                    String::from_utf8(key.0).ok()
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();

        if members.is_empty() {
            return Err(CommandError::InvalidArgument("Invalid hmget".to_string()));
        }

        Ok(HmGet { key, members })
    }
}

#[cfg(test)]
mod tests {
    use crate::{Backend, BulkString, Command, RespArray, RespFrame};

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_hmget() -> Result<()> {
        let backend = &Backend::new();
        let frame: RespFrame = Some(RespArray::new(vec![
            Some(BulkString::new("hset".to_string())).into(),
            Some(BulkString::new("myhash".to_string())).into(),
            Some(BulkString::new("field1".to_string())).into(),
            Some(BulkString::new("value1".to_string())).into(),
        ]))
        .into();

        let sadd = Command::try_from(frame)?;
        let ret = sadd.execute(backend);

        assert_eq!(ret, 1.into());

        let frame: RespFrame = Some(RespArray::new(vec![
            Some(BulkString::new("hset".to_string())).into(),
            Some(BulkString::new("myhash".to_string())).into(),
            Some(BulkString::new("field2".to_string())).into(),
            Some(BulkString::new("value2".to_string())).into(),
        ]))
        .into();

        let sadd = Command::try_from(frame)?;
        let ret = sadd.execute(backend);

        assert_eq!(ret, 1.into());

        let frame: RespFrame = Some(RespArray::new(vec![
            Some(BulkString::new("hmget".to_string())).into(),
            Some(BulkString::new("myhash".to_string())).into(),
            Some(BulkString::new("field1".to_string())).into(),
            Some(BulkString::new("field2".to_string())).into(),
            Some(BulkString::new("nofield".to_string())).into(),
        ]))
        .into();

        let sadd = Command::try_from(frame)?;
        let ret = sadd.execute(backend);

        assert_eq!(
            ret,
            Some(RespArray::new(vec![
                Some(BulkString::new("value1".to_string())).into(),
                Some(BulkString::new("value2".to_string())).into(),
                RespFrame::BulkString(None),
            ]))
            .into()
        );
        Ok(())
    }
}
