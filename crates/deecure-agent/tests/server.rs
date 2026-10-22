// SPDX-License-Identifier: Apache-2.0

//! End-to-end tests: a real agent on a real Unix socket.

#![allow(clippy::unwrap_used)] // test-only file: a failed unwrap is a failed test

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use deecure_agent::collect::SystemRoot;
use deecure_agent::context::Agent;
use deecure_agent::store::Store;
use deecure_agent::{server, socket};
use deecure_core::rpc::{MAX_MESSAGE_BYTES, RpcError};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::oneshot;

/// A fresh, short folder for one test (socket paths must stay under 108 bytes).
fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("deecure-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// Starts an agent on `path` and returns the switch that stops it.
fn start_agent(path: &Path) -> oneshot::Sender<()> {
    let listener = socket::bind(path).unwrap();
    let store = Arc::new(Mutex::new(Store::open_in_memory().unwrap()));
    let agent = Agent::new(store, SystemRoot::real());
    let (stop, stopped) = oneshot::channel::<()>();
    tokio::spawn(server::serve(listener, agent, async {
        let _ = stopped.await;
    }));
    stop
}

async fn ask(stream: &mut BufReader<UnixStream>, line: &str) -> Value {
    stream.get_mut().write_all(line.as_bytes()).await.unwrap();
    stream.get_mut().write_all(b"\n").await.unwrap();
    let mut reply = String::new();
    stream.read_line(&mut reply).await.unwrap();
    serde_json::from_str(&reply).unwrap()
}

#[tokio::test]
async fn answers_several_requests_on_one_connection() {
    let dir = test_dir("answers");
    let path = dir.join("agent.sock");
    let stop = start_agent(&path);

    let mut client = BufReader::new(UnixStream::connect(&path).await.unwrap());
    let version = ask(
        &mut client,
        r#"{"jsonrpc":"2.0","id":1,"method":"version"}"#,
    )
    .await;
    assert_eq!(version["result"]["agent"], deecure_core::VERSION);

    // Blank lines are skipped, not answered.
    client.get_mut().write_all(b"\n").await.unwrap();
    let status = ask(&mut client, r#"{"jsonrpc":"2.0","id":2,"method":"status"}"#).await;
    assert_eq!(status["id"], 2);
    assert_eq!(status["result"]["findings"], Value::Array(vec![]));

    let _ = stop.send(());
    let _ = std::fs::remove_dir_all(dir);
}

#[tokio::test]
async fn socket_and_folder_are_private() {
    let dir = test_dir("private");
    let path = dir.join("sub").join("agent.sock");
    let stop = start_agent(&path);

    let socket_mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    let dir_mode = std::fs::metadata(path.parent().unwrap())
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(socket_mode, 0o600);
    assert_eq!(dir_mode, 0o700);

    let _ = stop.send(());
    let _ = std::fs::remove_dir_all(dir);
}

#[tokio::test]
async fn stale_socket_is_replaced_but_live_agent_is_not() {
    let dir = test_dir("stale");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("agent.sock");

    // A socket file with nobody listening, like after a crash.
    drop(std::os::unix::net::UnixListener::bind(&path).unwrap());
    let stop = start_agent(&path);

    // A second agent on the same path must refuse to start.
    let second = socket::bind(&path).unwrap_err();
    assert_eq!(second.kind(), std::io::ErrorKind::AddrInUse);

    let _ = stop.send(());
    let _ = std::fs::remove_dir_all(dir);
}

#[tokio::test]
async fn oversized_message_is_refused_and_connection_closed() {
    let dir = test_dir("big");
    let path = dir.join("agent.sock");
    let stop = start_agent(&path);

    let mut client = BufReader::new(UnixStream::connect(&path).await.unwrap());
    let huge = vec![b'x'; MAX_MESSAGE_BYTES + 10];
    client.get_mut().write_all(&huge).await.unwrap();

    let mut reply = String::new();
    client.read_line(&mut reply).await.unwrap();
    let reply: Value = serde_json::from_str(&reply).unwrap();
    assert_eq!(reply["error"]["code"], RpcError::INVALID_REQUEST);

    let mut rest = Vec::new();
    assert_eq!(
        client.read_to_end(&mut rest).await.unwrap(),
        0,
        "connection should close"
    );

    let _ = stop.send(());
    let _ = std::fs::remove_dir_all(dir);
}
