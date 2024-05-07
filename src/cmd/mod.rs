mod echo;
mod hmget;
mod sadd;
mod sismember;
mod unrecognized;

use enum_dispatch::enum_dispatch;
// you could also use once_cell instead of lazy_static
use lazy_static::lazy_static;
use thiserror::Error;

use crate::{Backend, RespArray, RespError, RespFrame, SimpleString};

use self::{
    echo::Echo, hmget::HmGet, sadd::SAdd, sismember::SisMember, unrecognized::Unrecognized,
};

lazy_static! {
    static ref RESP_OK: RespFrame = SimpleString::new("OK").into();
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("{0}")]
    RespError(#[from] RespError),
    #[error("Utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

#[enum_dispatch]
pub trait CommandExecutor {
    fn execute(self, backend: &Backend) -> RespFrame;
}

#[enum_dispatch(CommandExecutor)]
#[derive(Debug)]
pub enum Command {
    Echo(Echo),
    SAdd(SAdd),
    SisMember(SisMember),
    HmGet(HmGet),
    // unrecognized command
    Unrecognized(Unrecognized),
}

impl TryFrom<RespFrame> for Command {
    type Error = CommandError;
    fn try_from(v: RespFrame) -> Result<Self, Self::Error> {
        match v {
            RespFrame::Array(array) => match array {
                Some(array) => array.try_into(),
                _ => Err(CommandError::InvalidCommand("Command is null".to_string())),
            },
            _ => Err(CommandError::InvalidCommand(
                "Command must be an Array".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for Command {
    type Error = CommandError;
    fn try_from(v: RespArray) -> Result<Self, Self::Error> {
        match v.first() {
            Some(RespFrame::BulkString(ref cmd)) => match cmd {
                Some(cmd) => match cmd.as_ref() {
                    b"echo" => Ok(Echo::try_from(v)?.into()),
                    b"hmget" => Ok(HmGet::try_from(v)?.into()),
                    b"sadd" => Ok(SAdd::try_from(v)?.into()),
                    b"sismember" => Ok(SisMember::try_from(v)?.into()),
                    _ => Ok(Unrecognized.into()),
                },
                _ => Err(CommandError::InvalidCommand("Command is null".to_string())),
            },
            _ => Err(CommandError::InvalidCommand(
                "Command must have a BulkString as the first argument".to_string(),
            )),
        }
    }
}

enum ArgsCheckRule {
    Equal,
    EqualOrGreater,
}

fn validate_command(
    value: &RespArray,
    names: &[&'static str],
    n_args: usize,
    rule: ArgsCheckRule,
) -> Result<(), CommandError> {
    match rule {
        ArgsCheckRule::Equal => {
            if value.len() != n_args + names.len() {
                return Err(CommandError::InvalidArgument(format!(
                    "{} command must have exactly {} argument",
                    names.join(" "),
                    n_args
                )));
            }
        }
        ArgsCheckRule::EqualOrGreater => {
            if value.len() < n_args + names.len() {
                return Err(CommandError::InvalidArgument(format!(
                    "{} command must have minimum required {} argument",
                    names.join(" "),
                    n_args
                )));
            }
        }
    }

    for (i, name) in names.iter().enumerate() {
        match value[i] {
            RespFrame::BulkString(ref cmd) => {
                if let Some(cmd) = cmd {
                    if cmd.as_ref().to_ascii_lowercase() != name.as_bytes() {
                        return Err(CommandError::InvalidCommand(format!(
                            "Invalid command: expected {}, got {}",
                            name,
                            String::from_utf8_lossy(cmd.as_ref())
                        )));
                    }
                } else {
                    return Err(CommandError::InvalidCommand("Command is null".to_string()));
                }
            }
            _ => {
                return Err(CommandError::InvalidCommand(
                    "Command must have a BulkString as the first argument".to_string(),
                ))
            }
        }
    }
    Ok(())
}

fn extract_args(value: RespArray, start: usize) -> Result<Vec<RespFrame>, CommandError> {
    Ok(value.0.into_iter().skip(start).collect::<Vec<RespFrame>>())
}
