// SPDX-License-Identifier: Apache-2.0

//! A tiny, blocking client for the agent's socket, shared by the CLI and the desktop app.

use std::fmt;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::rpc::{MAX_MESSAGE_BYTES, Outcome, Request, Response, RpcError};
use serde::de::DeserializeOwned;
use serde_json::Value;

/// How long to wait for the agent before giving up.
const TIMEOUT: Duration = Duration::from_secs(10);

/// Why a call to the agent failed.
#[derive(Debug)]
pub enum ClientError {
    /// Nothing answers at the socket: the agent is not running.
    NotRunning(PathBuf),
    /// Reading or writing the socket failed.
    Io(io::Error),
    /// The agent sent something that is not a valid answer.
    BadReply(String),
    /// The agent answered with an error.
    Agent(RpcError),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::NotRunning(path) => write!(
                f,
                "the DeeCure agent is not running (nothing answers at {}).\n\
                 Start it with:   systemctl --user start deecure-agent.socket\n\
                 In a source checkout:   make agent",
                path.display()
            ),
            ClientError::Io(e)
                if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut =>
            {
                write!(
                    f,
                    "the agent did not answer within {} seconds",
                    TIMEOUT.as_secs()
                )
            }
            ClientError::Io(e) => write!(f, "could not talk to the agent: {e}"),
            ClientError::BadReply(why) => write!(
                f,
                "the agent sent an answer this version cannot read: {why}"
            ),
            ClientError::Agent(e) => write!(f, "the agent reported an error: {}", e.message),
        }
    }
}

impl std::error::Error for ClientError {}

impl From<io::Error> for ClientError {
    fn from(e: io::Error) -> Self {
        ClientError::Io(e)
    }
}

/// Calls one agent method and reads its result.
pub fn call<T: DeserializeOwned>(
    socket: &Path,
    method: &str,
    params: Option<Value>,
) -> Result<T, ClientError> {
    let stream = UnixStream::connect(socket).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused => {
            ClientError::NotRunning(socket.to_owned())
        }
        _ => ClientError::Io(e),
    })?;
    stream.set_read_timeout(Some(TIMEOUT))?;
    stream.set_write_timeout(Some(TIMEOUT))?;

    let mut line =
        serde_json::to_vec(&Request::new(1, method, params)).map_err(io::Error::other)?;
    line.push(b'\n');
    (&stream).write_all(&line)?;

    let mut reply = String::new();
    BufReader::new((&stream).take(MAX_MESSAGE_BYTES as u64 + 1)).read_line(&mut reply)?;
    if reply.is_empty() {
        return Err(ClientError::BadReply(
            "the agent closed the connection without answering".into(),
        ));
    }
    let response: Response =
        serde_json::from_str(&reply).map_err(|e| ClientError::BadReply(e.to_string()))?;
    match response.outcome {
        Outcome::Success(value) => {
            serde_json::from_value(value).map_err(|e| ClientError::BadReply(e.to_string()))
        }
        Outcome::Failure(error) => Err(ClientError::Agent(error)),
    }
}
