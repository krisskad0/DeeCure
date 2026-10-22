// SPDX-License-Identifier: Apache-2.0

//! What every request handler can reach: the database, the system root, and live readers.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::collect::SystemRoot;
use crate::inventory::ProcessSampler;
use crate::store::SharedStore;

/// The running agent.
pub struct Agent {
    /// The database.
    pub store: SharedStore,
    /// Where `/proc`, `/sys`, and `/etc` are read from.
    pub root: SystemRoot,
    /// Remembers the last process reading, so CPU use is measured between requests.
    processes: Mutex<ProcessSampler>,
}

impl Agent {
    /// A shared agent around an open database.
    pub fn new(store: SharedStore, root: SystemRoot) -> Arc<Agent> {
        Arc::new(Agent {
            store,
            root,
            processes: Mutex::new(ProcessSampler::default()),
        })
    }

    /// The process reader. A panic elsewhere does not make it unusable.
    pub fn processes(&self) -> MutexGuard<'_, ProcessSampler> {
        self.processes
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}
