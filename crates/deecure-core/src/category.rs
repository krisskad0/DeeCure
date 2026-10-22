// SPDX-License-Identifier: Apache-2.0

//! The areas of the computer that DeeCure checks.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// An area of the computer. Each one is a page in the app, and every rule belongs to one.
///
/// The IDs match the page IDs in `docs/03-ui-design/05-pages/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    /// CPU, memory, disk I/O, network, GPU, pressure.
    Performance,
    /// Journal, errors, crashes, kernel, boots.
    Logs,
    /// What changed: packages, config, devices, DeeCure actions.
    Activity,
    /// OS, boot, updates, kernel, time and locale.
    System,
    /// Processor, memory, storage devices, graphics, sensors, firmware.
    Hardware,
    /// Battery, power profile, sleep.
    Power,
    /// Displays and audio.
    Display,
    /// Running processes.
    Processes,
    /// System services, user services, startup apps.
    Services,
    /// Cron, systemd timers, anacron, at jobs.
    Jobs,
    /// Containers, images, volumes, virtual machines.
    Containers,
    /// Installed packages, sources, cleanup, permissions.
    Packages,
    /// Filesystems, disk usage, cleanup.
    Storage,
    /// Backup jobs, snapshots, restore points.
    Backups,
    /// Interfaces, connections, ports, firewall, DNS and VPN.
    Network,
    /// Users, groups, sessions, logins, SSH keys.
    Accounts,
    /// Vulnerabilities, hardening, file integrity, malware.
    Security,
    /// Device access, app permissions, privacy settings.
    Privacy,
}

impl Category {
    /// Every category, in sidebar order.
    pub const ALL: [Category; 18] = [
        Category::Performance,
        Category::Logs,
        Category::Activity,
        Category::System,
        Category::Hardware,
        Category::Power,
        Category::Display,
        Category::Processes,
        Category::Services,
        Category::Jobs,
        Category::Containers,
        Category::Packages,
        Category::Storage,
        Category::Backups,
        Category::Network,
        Category::Accounts,
        Category::Security,
        Category::Privacy,
    ];

    /// The ID used in rule IDs, JSON, and the database, such as `storage`.
    pub const fn id(self) -> &'static str {
        match self {
            Category::Performance => "performance",
            Category::Logs => "logs",
            Category::Activity => "activity",
            Category::System => "system",
            Category::Hardware => "hardware",
            Category::Power => "power",
            Category::Display => "display",
            Category::Processes => "processes",
            Category::Services => "services",
            Category::Jobs => "jobs",
            Category::Containers => "containers",
            Category::Packages => "packages",
            Category::Storage => "storage",
            Category::Backups => "backups",
            Category::Network => "network",
            Category::Accounts => "accounts",
            Category::Security => "security",
            Category::Privacy => "privacy",
        }
    }

    /// The name shown to users, such as `Scheduled Jobs`.
    pub const fn label(self) -> &'static str {
        match self {
            Category::Performance => "Performance",
            Category::Logs => "Logs",
            Category::Activity => "Activity",
            Category::System => "System",
            Category::Hardware => "Hardware",
            Category::Power => "Power",
            Category::Display => "Display & Audio",
            Category::Processes => "Processes",
            Category::Services => "Services",
            Category::Jobs => "Scheduled Jobs",
            Category::Containers => "Containers",
            Category::Packages => "Packages",
            Category::Storage => "Storage",
            Category::Backups => "Backups",
            Category::Network => "Network",
            Category::Accounts => "Accounts",
            Category::Security => "Security",
            Category::Privacy => "Privacy",
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

/// The text is not a known category ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownCategory(pub String);

impl fmt::Display for UnknownCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown category '{}'", self.0)
    }
}

impl std::error::Error for UnknownCategory {}

impl FromStr for Category {
    type Err = UnknownCategory;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Category::ALL
            .into_iter()
            .find(|c| c.id() == s)
            .ok_or_else(|| UnknownCategory(s.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_round_trips_through_from_str_and_serde() {
        for c in Category::ALL {
            assert_eq!(c.id().parse::<Category>(), Ok(c));
            assert_eq!(serde_json::to_value(c).unwrap(), c.id());
        }
    }

    #[test]
    fn unknown_id_is_rejected() {
        assert_eq!(
            "overview".parse::<Category>(),
            Err(UnknownCategory("overview".into()))
        );
    }

    #[test]
    fn labels_match_the_ui() {
        assert_eq!(Category::Jobs.label(), "Scheduled Jobs");
        assert_eq!(Category::Display.label(), "Display & Audio");
    }
}
