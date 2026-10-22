// SPDX-License-Identifier: Apache-2.0

//! Runs a read-only helper program, such as `pw-dump` or `journalctl`, with a time limit.

use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;

/// The program's output, or `None` if it is missing, fails, or takes longer than `limit`.
pub async fn output(program: &str, args: &[&str], limit: Duration) -> Option<String> {
    let child = Command::new(program)
        .args(args)
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .ok()?;
    let out = tokio::time::timeout(limit, child.wait_with_output())
        .await
        .ok()?
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Like [`output`], but also returns the output when the program exits with an error,
/// for tools such as `dpkg -S` that fail when only some of their answers are missing.
pub async fn output_any(program: &str, args: &[&str], limit: Duration) -> Option<String> {
    let child = Command::new(program)
        .args(args)
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .ok()?;
    let out = tokio::time::timeout(limit, child.wait_with_output())
        .await
        .ok()?
        .ok()?;
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}
