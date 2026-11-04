// SPDX-License-Identifier: Apache-2.0

//! A tiny, blocking client for the agent's socket, shared by the CLI and the desktop app.

use std::fmt;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::rpc::{MAX_MESSAGE_BYTES, Outcome, Request, Response, RpcError};
use serde::de::DeserializeOwned;
use serde_json::Value;

/// How long to wait for the agent before giving up.
const TIMEOUT: Duration = Duration::from_secs(10);
/// Inventory kinds that run many system tools, and how long they may take: the Optimize checks
/// took 10.6 s on a laptop, so the usual limit failed them every time.
const SLOW_KINDS: [&str; 1] = ["optimize"];
const SLOW_TIMEOUT: Duration = Duration::from_secs(60);

/// The time limit for one call: longer for SLOW_KINDS, so a stuck agent is still found fast.
fn timeout_for(method: &str, params: Option<&Value>) -> Duration {
    let kind = params.and_then(|p| p.get("kind")).and_then(Value::as_str);
    if method == crate::rpc::method::INVENTORY && kind.is_some_and(|k| SLOW_KINDS.contains(&k)) {
        SLOW_TIMEOUT
    } else {
        TIMEOUT
    }
}

/// Why a call to the agent failed.
#[derive(Debug)]
pub enum ClientError {
    /// Nothing answers at the socket: the agent is not running.
    NotRunning(PathBuf),
    /// Reading or writing the socket failed.
    Io(io::Error),
    /// The agent did not answer within this time.
    TimedOut(Duration),
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
                 Start it with:   systemctl --user daemon-reload && systemctl --user start deecure-agent.socket\n\
                 In a source checkout:   make agent",
                path.display()
            ),
            ClientError::TimedOut(limit) => write!(
                f,
                "the agent did not answer within {} seconds",
                limit.as_secs()
            ),
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

/// How often the client may try to start the agent itself.
const START_EVERY: Duration = Duration::from_secs(60);
/// When the client last tried to start the agent.
static LAST_START: Mutex<Option<Instant>> = Mutex::new(None);

/// Runs a short command and waits at most `limit`. True when it exits with success.
fn run_briefly(program: &str, args: &[&str], limit: Duration) -> bool {
    let Ok(mut child) = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    else {
        return false;
    };
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) if started.elapsed() < limit => std::thread::sleep(Duration::from_millis(50)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return false;
            }
        }
    }
}

/// Starts the agent's socket in the user's own systemd (no password, nothing system-wide),
/// and waits up to 5 seconds for it to accept connections. Right after the package is
/// installed, systemd may not know the new units yet, hence the daemon-reload first.
/// Tried at most once a minute. True when the socket answers afterwards.
pub fn start_agent(socket: &Path) -> bool {
    {
        let mut last = LAST_START
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if last.is_some_and(|t| t.elapsed() < START_EVERY) {
            return false;
        }
        *last = Some(Instant::now());
    }
    let limit = Duration::from_secs(10);
    run_briefly("systemctl", &["--user", "daemon-reload"], limit);
    if !run_briefly(
        "systemctl",
        &["--user", "start", "deecure-agent.socket"],
        limit,
    ) {
        return false;
    }
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(5) {
        if UnixStream::connect(socket).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

/// Calls one agent method and reads its result. When nothing answers at the usual
/// socket, the agent is started once (see [`start_agent`]) and the call is tried again.
pub fn call<T: DeserializeOwned>(
    socket: &Path,
    method: &str,
    params: Option<Value>,
) -> Result<T, ClientError> {
    match call_once(socket, method, params.clone()) {
        Err(ClientError::NotRunning(_))
            if crate::rpc::default_socket_path().as_deref() == Some(socket)
                && start_agent(socket) =>
        {
            call_once(socket, method, params)
        }
        other => other,
    }
}

fn call_once<T: DeserializeOwned>(
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
    let limit = timeout_for(method, params.as_ref());
    stream.set_read_timeout(Some(limit))?;
    stream.set_write_timeout(Some(limit))?;
    let timed_out = |e: io::Error| match e.kind() {
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => ClientError::TimedOut(limit),
        _ => ClientError::Io(e),
    };

    let mut line =
        serde_json::to_vec(&Request::new(1, method, params)).map_err(io::Error::other)?;
    line.push(b'\n');
    (&stream).write_all(&line).map_err(timed_out)?;

    let mut reply = String::new();
    BufReader::new((&stream).take(MAX_MESSAGE_BYTES as u64 + 1))
        .read_line(&mut reply)
        .map_err(timed_out)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn only_slow_kinds_get_the_long_limit() {
        let optimize = json!({ "kind": "optimize" });
        let system = json!({ "kind": "system" });
        assert_eq!(timeout_for("inventory", Some(&optimize)), SLOW_TIMEOUT);
        assert_eq!(timeout_for("inventory", Some(&system)), TIMEOUT);
        assert_eq!(timeout_for("status", None), TIMEOUT);
        // The error names the limit that was used.
        assert_eq!(
            ClientError::TimedOut(SLOW_TIMEOUT).to_string(),
            "the agent did not answer within 60 seconds"
        );
    }
}
