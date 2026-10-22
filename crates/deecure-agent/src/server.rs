// SPDX-License-Identifier: Apache-2.0

//! The socket server: one JSON-RPC message per line, same user only.

use std::future::Future;
use std::time::Duration;

use deecure_core::rpc::{MAX_MESSAGE_BYTES, Response, RpcError};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

use std::sync::Arc;

use crate::context::Agent;
use crate::handler;

/// Serves clients until `shutdown` completes.
pub async fn serve(listener: UnixListener, agent: Arc<Agent>, shutdown: impl Future<Output = ()>) {
    let our_uid = rustix::process::getuid().as_raw();
    tokio::pin!(shutdown);
    loop {
        tokio::select! {
            () = &mut shutdown => break,
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => {
                    tokio::spawn(serve_connection(stream, our_uid, agent.clone()));
                }
                Err(e) => {
                    eprintln!("deecure-agent: accept failed: {e}");
                    // Avoid a busy loop if accepting keeps failing (for example, out of file handles).
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            },
        }
    }
}

async fn serve_connection(stream: UnixStream, our_uid: u32, agent: Arc<Agent>) {
    match stream.peer_cred() {
        Ok(cred) if cred.uid() == our_uid => {}
        Ok(cred) => {
            eprintln!(
                "deecure-agent: refused a connection from user ID {}",
                cred.uid()
            );
            return;
        }
        Err(e) => {
            eprintln!("deecure-agent: could not check who is connecting: {e}");
            return;
        }
    }

    let (read, mut write) = stream.into_split();
    let mut reader = BufReader::new(read);
    let mut line = Vec::new();
    loop {
        line.clear();
        let limit = (MAX_MESSAGE_BYTES + 1) as u64;
        let read = match (&mut reader).take(limit).read_until(b'\n', &mut line).await {
            Ok(0) | Err(_) => return,
            Ok(n) => n,
        };
        let complete = line.last() == Some(&b'\n');
        let response = if !complete && read > MAX_MESSAGE_BYTES {
            let error = RpcError::new(
                RpcError::INVALID_REQUEST,
                format!("message is longer than {MAX_MESSAGE_BYTES} bytes"),
            );
            Response::failure(None, error)
        } else if line.trim_ascii().is_empty() {
            continue;
        } else {
            handler::handle_line(&line, &agent).await
        };
        if send(&mut write, &response).await.is_err() || !complete {
            // The client hung up, or the message was cut off: close the connection.
            return;
        }
    }
}

async fn send(
    write: &mut (impl AsyncWriteExt + Unpin),
    response: &Response,
) -> std::io::Result<()> {
    let mut bytes = serde_json::to_vec(response).map_err(std::io::Error::other)?;
    bytes.push(b'\n');
    write.write_all(&bytes).await?;
    write.flush().await
}
