use enum_dispatch::enum_dispatch;
// you could also use once_cell instead of lazy_static
use lazy_static::lazy_static;
use thiserror::Error;

use crate::{RespArray, RespError, RespFrame, SimpleString};

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
    fn execute(self) -> RespFrame;
}

#[enum_dispatch(CommandExecutor)]
#[derive(Debug)]
pub enum Command {
    Echo(Echo),

    // unrecognized command
    Unrecognized(Unrecognized),
}

#[derive(Debug)]
pub struct Echo {
    echo: RespFrame,
}
impl CommandExecutor for Echo {
    fn execute(self) -> RespFrame {
        self.echo
    }
}

#[derive(Debug)]
pub struct Unrecognized;
impl CommandExecutor for Unrecognized {
    fn execute(self) -> RespFrame {
        RESP_OK.clone()
    }
}

impl TryFrom<RespFrame> for Command {
    type Error = CommandError;
    fn try_from(v: RespFrame) -> Result<Self, Self::Error> {
        match v {
            RespFrame::Array(array) => array.try_into(),
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
            Some(RespFrame::BulkString(ref cmd)) => match cmd.as_ref() {
                b"echo" => Ok(Echo::try_from(v)?.into()),
                _ => Ok(Unrecognized.into()),
            },
            _ => Err(CommandError::InvalidCommand(
                "Command must have a BulkString as the first argument".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for Echo {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["echo"], 1)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(echo) => Ok(Echo { echo }),
            _ => Err(CommandError::InvalidArgument("Invalid echo".to_string())),
        }
    }
}

fn validate_command(
    value: &RespArray,
    names: &[&'static str],
    n_args: usize,
) -> Result<(), CommandError> {
    if value.len() != n_args + names.len() {
        return Err(CommandError::InvalidArgument(format!(
            "{} command must have exactly {} argument",
            names.join(" "),
            n_args
        )));
    }

    for (i, name) in names.iter().enumerate() {
        match value[i] {
            RespFrame::BulkString(ref cmd) => {
                if cmd.as_ref().to_ascii_lowercase() != name.as_bytes() {
                    return Err(CommandError::InvalidCommand(format!(
                        "Invalid command: expected {}, got {}",
                        name,
                        String::from_utf8_lossy(cmd.as_ref())
                    )));
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
