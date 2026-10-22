// SPDX-License-Identifier: Apache-2.0

//! Opening the agent's private socket.

use std::fs::{self, DirBuilder, Permissions};
use std::io;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::Path;

use tokio::net::UnixListener;

/// Takes the socket systemd opened for us (`deecure-agent.socket`), if any.
///
/// Returns `None` when the agent was started by hand rather than by socket activation.
pub fn from_systemd() -> io::Result<Option<UnixListener>> {
    let mut fds = listenfd::ListenFd::from_env();
    match fds.take_unix_listener(0)? {
        Some(listener) => {
            listener.set_nonblocking(true)?;
            UnixListener::from_std(listener).map(Some)
        }
        None => Ok(None),
    }
}

/// Opens the socket at `path`, readable only by this user.
///
/// - Creates the parent folder with mode `0700` if it is missing.
/// - Replaces a stale socket left by an agent that crashed.
/// - Fails with `AddrInUse` if another agent is already answering there.
pub fn bind(path: &Path) -> io::Result<UnixListener> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        DirBuilder::new().recursive(true).mode(0o700).create(dir)?;
    }
    if fs::symlink_metadata(path).is_ok() {
        if std::os::unix::net::UnixStream::connect(path).is_ok() {
            return Err(io::Error::new(
                io::ErrorKind::AddrInUse,
                format!("another agent is already listening on {}", path.display()),
            ));
        }
        fs::remove_file(path)?;
    }
    let listener = std::os::unix::net::UnixListener::bind(path)?;
    fs::set_permissions(path, Permissions::from_mode(0o600))?;
    listener.set_nonblocking(true)?;
    UnixListener::from_std(listener)
}
