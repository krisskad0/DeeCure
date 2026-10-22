// SPDX-License-Identifier: Apache-2.0

//! Disk space of local filesystems, from `/proc/mounts` and `statvfs`.

use std::io;
use std::path::Path;

use deecure_core::{Snapshot, SnapshotKind, Timestamp, Unit};
use serde::Serialize;

use super::{Collector, Sample, SystemRoot, metric};

/// Filesystem types stored on this computer's own disks.
///
/// Network filesystems are left out on purpose: checking them can hang when the
/// server is away. So are pseudo filesystems (proc, tmpfs) and snap images (squashfs).
const LOCAL_TYPES: &[&str] = &[
    "ext2", "ext3", "ext4", "btrfs", "xfs", "f2fs", "vfat", "exfat", "ntfs", "ntfs3", "fuseblk",
    "zfs", "jfs", "reiserfs", "bcachefs",
];

/// One mounted local filesystem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Mount {
    /// Where it is mounted, such as `/` or `/boot/efi`.
    pub mount_point: String,
    /// The device, such as `/dev/nvme0n1p2`.
    pub device: String,
    /// The filesystem type, such as `ext4`.
    pub fs_type: String,
}

/// Space on one filesystem, in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FsUsage {
    /// Total size.
    pub size: u64,
    /// Space in use.
    pub used: u64,
    /// Space a normal user can still write (the root-only reserve is not counted).
    pub free: u64,
}

impl FsUsage {
    /// Percent used, computed like `df` does: used / (used + free).
    pub fn used_percent(&self) -> f64 {
        let usable = self.used + self.free;
        if usable == 0 {
            0.0
        } else {
            self.used as f64 * 100.0 / usable as f64
        }
    }
}

/// Reads `/proc/mounts`: local filesystems only, one entry per device (the first mount wins).
pub fn parse_mounts(text: &str) -> Vec<Mount> {
    let mut mounts: Vec<Mount> = Vec::new();
    for line in text.lines() {
        let mut fields = line.split_whitespace();
        let (Some(device), Some(mount_point), Some(fs_type)) =
            (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        if !LOCAL_TYPES.contains(&fs_type) || mounts.iter().any(|m| m.device == device) {
            continue;
        }
        mounts.push(Mount {
            mount_point: unescape(mount_point),
            device: unescape(device),
            fs_type: fs_type.to_owned(),
        });
    }
    mounts
}

/// `/proc/mounts` writes spaces, tabs, newlines, and backslashes as `\040`, `\011`, `\012`, `\134`.
fn unescape(field: &str) -> String {
    let bytes = field.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let octal = bytes
            .get(i + 1..i + 4)
            .filter(|digits| bytes[i] == b'\\' && digits.iter().all(|d| (b'0'..=b'7').contains(d)));
        match octal {
            Some(digits) => {
                let value = digits
                    .iter()
                    .fold(0_u32, |acc, d| acc * 8 + u32::from(d - b'0'));
                out.push(u8::try_from(value).unwrap_or(b'?'));
                i += 4;
            }
            None => {
                out.push(bytes[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Asks the kernel how much space a mounted filesystem has.
pub fn statvfs(mount_point: &Path) -> io::Result<FsUsage> {
    let stat = rustix::fs::statvfs(mount_point)?;
    let block = stat.f_frsize;
    Ok(FsUsage {
        size: stat.f_blocks.saturating_mul(block),
        used: stat
            .f_blocks
            .saturating_sub(stat.f_bfree)
            .saturating_mul(block),
        free: stat.f_bavail.saturating_mul(block),
    })
}

type StatFn = Box<dyn Fn(&Path) -> io::Result<FsUsage> + Send>;

/// Disk space per mount point, plus an inventory of the filesystems.
pub struct Filesystems {
    stat: StatFn,
}

impl Filesystems {
    /// Reads real disk space with `statvfs`.
    pub fn real() -> Self {
        Filesystems {
            stat: Box::new(statvfs),
        }
    }

    /// Uses `stat` instead of `statvfs`, for tests and saved fixtures.
    pub fn with_stat(stat: impl Fn(&Path) -> io::Result<FsUsage> + Send + 'static) -> Self {
        Filesystems {
            stat: Box::new(stat),
        }
    }
}

/// One line of the filesystems inventory. Only things that rarely change, so the
/// inventory is stored again only when a disk is added, removed, or resized.
#[derive(Serialize)]
struct InventoryEntry<'a> {
    #[serde(flatten)]
    mount: &'a Mount,
    size: u64,
}

impl Collector for Filesystems {
    fn name(&self) -> &'static str {
        "filesystems"
    }

    fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let mounts = parse_mounts(&root.read("/proc/mounts")?);
        let mut sample = Sample::default();
        let mut inventory = Vec::new();
        for mount in &mounts {
            // A mount we may not read (another user's, for example) is skipped, not fatal.
            let Ok(usage) = (self.stat)(Path::new(&mount.mount_point)) else {
                continue;
            };
            let subject = Some(mount.mount_point.as_str());
            sample.metrics.push(metric(
                "disk.used_percent",
                subject,
                usage.used_percent(),
                Unit::Percent,
                now,
            ));
            sample.metrics.push(metric(
                "disk.free",
                subject,
                usage.free as f64,
                Unit::Bytes,
                now,
            ));
            inventory.push(InventoryEntry {
                mount,
                size: usage.size,
            });
        }
        sample.snapshots.push(Snapshot {
            kind: SnapshotKind::Filesystems,
            taken_at: now,
            data: serde_json::to_value(&inventory).map_err(io::Error::other)?,
        });
        Ok(sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::{fixture_root, value};
    use serde_json::json;

    const MOUNTS: &str = "\
sysfs /sys sysfs rw,nosuid,nodev,noexec,relatime 0 0
proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0
/dev/nvme0n1p2 / btrfs rw,relatime,subvol=/@ 0 0
/dev/loop3 /snap/firefox/4793 squashfs ro,nodev,relatime 0 0
tmpfs /run tmpfs rw,nosuid,nodev,size=1611256k 0 0
/dev/nvme0n1p2 /home btrfs rw,relatime,subvol=/@home 0 0
/dev/nvme0n1p1 /boot/efi vfat rw,relatime,fmask=0077 0 0
/dev/sda1 /media/riya/My\\040Passport ntfs3 rw,nosuid,nodev 0 0
server:/share /mnt/nas nfs4 rw,relatime 0 0
";

    #[test]
    fn keeps_local_filesystems_once_per_device() {
        let points: Vec<_> = parse_mounts(MOUNTS)
            .into_iter()
            .map(|m| m.mount_point)
            .collect();
        assert_eq!(points, ["/", "/boot/efi", "/media/riya/My Passport"]);
    }

    #[test]
    fn unescapes_octal_codes() {
        assert_eq!(unescape(r"a\040b\011c\134d"), "a b\tc\\d");
        assert_eq!(unescape(r"trailing\04"), r"trailing\04");
    }

    #[test]
    fn used_percent_matches_df() {
        let usage = FsUsage {
            size: 1000,
            used: 900,
            free: 50,
        };
        assert!((usage.used_percent() - 94.736).abs() < 0.001);
        assert_eq!(
            FsUsage {
                size: 0,
                used: 0,
                free: 0
            }
            .used_percent(),
            0.0
        );
    }

    #[test]
    fn reports_space_and_a_stable_inventory() {
        let root = fixture_root("filesystems", &[("/proc/mounts", MOUNTS)]);
        let mut collector = Filesystems::with_stat(|path| match path.to_str() {
            Some("/") => Ok(FsUsage {
                size: 238_000,
                used: 224_200,
                free: 13_800,
            }),
            Some("/boot/efi") => Ok(FsUsage {
                size: 500,
                used: 50,
                free: 450,
            }),
            _ => Err(io::Error::from(io::ErrorKind::PermissionDenied)),
        });
        let sample = collector.collect(&root, Timestamp(0)).unwrap();
        assert!((value(&sample, "disk.used_percent", Some("/")).unwrap() - 94.2).abs() < 0.01);
        assert_eq!(value(&sample, "disk.free", Some("/boot/efi")), Some(450.0));
        assert_eq!(
            value(&sample, "disk.free", Some("/media/riya/My Passport")),
            None
        );
        assert_eq!(
            sample.snapshots[0].data,
            json!([
                {"mount_point": "/", "device": "/dev/nvme0n1p2", "fs_type": "btrfs", "size": 238000},
                {"mount_point": "/boot/efi", "device": "/dev/nvme0n1p1", "fs_type": "vfat", "size": 500}
            ])
        );
    }

    #[test]
    fn statvfs_reads_the_real_root() {
        let usage = statvfs(Path::new("/")).unwrap();
        assert!(usage.size > 0 && usage.used <= usage.size);
    }
}
