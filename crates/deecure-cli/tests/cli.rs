// SPDX-License-Identifier: Apache-2.0

//! Runs the real `deecure` binary against a stand-in agent on a temporary socket.

#![allow(clippy::unwrap_used)] // test-only file: a failed unwrap is a failed test

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread::{self, JoinHandle};

use serde_json::{Value, json};

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("deecure-cli-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Answers one request with `result` and returns the request it got.
fn fake_agent(socket: &Path, result: Value) -> JoinHandle<Value> {
    let listener = UnixListener::bind(socket).unwrap();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut line = String::new();
        BufReader::new(&stream).read_line(&mut line).unwrap();
        let request: Value = serde_json::from_str(&line).unwrap();
        let reply = json!({"jsonrpc": "2.0", "id": request["id"], "result": result});
        writeln!(stream, "{reply}").unwrap();
        request
    })
}

fn deecure(socket: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_deecure"))
        .arg("--socket")
        .arg(socket)
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .unwrap()
}

fn report() -> Value {
    json!({
        "generated_at": 1_000_000,
        "checked_at": 1_000_000,
        "findings": [{
            "rule": "power.battery_worn",
            "subject": "BAT0",
            "severity": "info",
            "state": "open",
            "title": "Battery has lost some of its capacity",
            "explain": "Battery BAT0 now holds 79.3% of the charge it held when new.",
            "first_seen": 1,
            "last_seen": 1_000_000
        }]
    })
}

#[test]
fn status_prints_plain_english() {
    let dir = test_dir("status");
    let socket = dir.join("agent.sock");
    let agent = fake_agent(&socket, report());

    let output = deecure(&socket, &["status"]);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(output.status.success(), "{stdout}");
    assert!(
        stdout.contains("i Info      Battery has lost some of its capacity"),
        "{stdout}"
    );
    assert!(stdout.contains("power.battery_worn · BAT0"), "{stdout}");
    assert!(
        stdout.ends_with("1 problem: 0 critical, 0 warnings, 1 info.\n"),
        "{stdout}"
    );
    assert_eq!(agent.join().unwrap()["method"], "status");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn status_json_is_the_raw_report() {
    let dir = test_dir("json");
    let socket = dir.join("agent.sock");
    let _agent = fake_agent(&socket, report());

    let output = deecure(&socket, &["status", "--json"]);
    assert!(output.status.success());
    let printed: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(printed["findings"][0]["rule"], "power.battery_worn");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn missing_agent_gets_a_helpful_message() {
    let dir = test_dir("missing");
    let output = deecure(&dir.join("nobody.sock"), &["status"]);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr.contains("the DeeCure agent is not running"),
        "{stderr}"
    );
    assert!(
        stderr.contains("systemctl --user start deecure-agent.socket"),
        "{stderr}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn no_command_shows_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_deecure"))
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(
        stderr.contains("Show the problems DeeCure found"),
        "{stderr}"
    );
}

#[test]
fn version_flag_works() {
    let output = Command::new(env!("CARGO_BIN_EXE_deecure"))
        .arg("--version")
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        format!("deecure {}", env!("CARGO_PKG_VERSION"))
    );
}
