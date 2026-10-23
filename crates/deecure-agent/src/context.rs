// SPDX-License-Identifier: Apache-2.0

//! What every request handler can reach: the database, the system root, and live readers.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::collect::SystemRoot;
use crate::inventory::{ProcessSampler, UsageState, VerifyState};
use crate::store::SharedStore;

/// The running agent.
pub struct Agent {
    /// The database.
    pub store: SharedStore,
    /// Where `/proc`, `/sys`, and `/etc` are read from.
    pub root: SystemRoot,
    /// Remembers the last process reading, so CPU use is measured between requests.
    processes: Mutex<ProcessSampler>,
    /// Wakes the collectors now instead of at the next minute (after a fix).
    pub recheck: Arc<tokio::sync::Notify>,
    /// Set once at start by `main`; empty in tests.
    pub facts: std::sync::OnceLock<Facts>,
    /// The last folder-size scan, shared with the thread that runs a new one.
    pub usage: Arc<Mutex<UsageState>>,
    /// The last full package file check, shared with the task that runs a new one.
    pub verify: Arc<Mutex<VerifyState>>,
    /// D-Bus connections, opened on first use and dropped after an error.
    system_bus: tokio::sync::Mutex<Option<zbus::Connection>>,
    session_bus: tokio::sync::Mutex<Option<zbus::Connection>>,
}

/// What the agent was started with, for Settings › About.
#[derive(Debug, Clone)]
pub struct Facts {
    /// The database file.
    pub database: std::path::PathBuf,
    /// Where rules were loaded from, and how many.
    pub rules_dir: std::path::PathBuf,
    /// How many rules loaded.
    pub rules: usize,
    /// When the agent started.
    pub started: std::time::Instant,
    /// The knowledge base.
    pub kb: crate::kb::Kb,
}

/// Which D-Bus bus to talk to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bus {
    /// The system bus: systemd, logind, UPower, BlueZ, fwupd.
    System,
    /// The user's session bus: the desktop shell.
    Session,
}

impl Agent {
    /// A shared agent around an open database.
    pub fn new(store: SharedStore, root: SystemRoot) -> Arc<Agent> {
        Arc::new(Agent {
            store,
            root,
            processes: Mutex::new(ProcessSampler::default()),
            usage: Arc::new(Mutex::new(UsageState::default())),
            verify: Arc::new(Mutex::new(VerifyState::default())),
            facts: std::sync::OnceLock::new(),
            recheck: Arc::new(tokio::sync::Notify::new()),
            system_bus: tokio::sync::Mutex::new(None),
            session_bus: tokio::sync::Mutex::new(None),
        })
    }

    /// A connection to `bus`, reused between requests.
    pub async fn bus(&self, bus: Bus) -> zbus::Result<zbus::Connection> {
        let slot = match bus {
            Bus::System => &self.system_bus,
            Bus::Session => &self.session_bus,
        };
        let mut slot = slot.lock().await;
        if let Some(conn) = slot.as_ref() {
            return Ok(conn.clone());
        }
        let conn = match bus {
            Bus::System => zbus::Connection::system().await?,
            Bus::Session => zbus::Connection::session().await?,
        };
        *slot = Some(conn.clone());
        Ok(conn)
    }

    /// Forgets a connection after an error, so the next call opens a new one.
    pub async fn drop_bus(&self, bus: Bus) {
        match bus {
            Bus::System => *self.system_bus.lock().await = None,
            Bus::Session => *self.session_bus.lock().await = None,
        }
    }

    /// True when `name` (such as `org.freedesktop.fwupd`) is running now. Asking never
    /// starts a service: DeeCure must not wake or change anything on the system.
    pub async fn is_running(&self, bus: Bus, name: &str) -> bool {
        let Ok(conn) = self.bus(bus).await else {
            return false;
        };
        let reply = conn
            .call_method(
                Some("org.freedesktop.DBus"),
                "/org/freedesktop/DBus",
                Some("org.freedesktop.DBus"),
                "NameHasOwner",
                &(name,),
            )
            .await;
        match reply {
            Ok(reply) => reply.body().deserialize::<bool>().unwrap_or(false),
            Err(_) => {
                self.drop_bus(bus).await;
                false
            }
        }
    }

    /// The process reader. A panic elsewhere does not make it unusable.
    pub fn processes(&self) -> MutexGuard<'_, ProcessSampler> {
        self.processes
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}
