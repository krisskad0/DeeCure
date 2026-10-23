// SPDX-License-Identifier: Apache-2.0

//! Drives and filesystems in depth for the Expert level (docs/01-product/expert-coverage.md,
//! sections 3 and 4): each disk's queue settings, block sizes, TRIM support, SATA link, NVMe
//! details, and partitions with their alignment; each filesystem's mount options, inodes, and
//! the errors the kernel recorded. No admin rights; read only when asked.

use serde::Serialize;

use crate::collect::SystemRoot;

/// One partition.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Partition {
    pub name: String,
    /// First sector (512 bytes each) and size in bytes.
    pub start_sector: u64,
    pub size: u64,
    /// Starts on a 1 MiB boundary, as SSDs want.
    pub aligned: bool,
}

/// NVMe controller facts.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Nvme {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firmware: Option<String>,
    /// `pcie`, `tcp`, …
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// The PCI address, to find its PCIe link in `pci_detail`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pci_address: Option<String>,
    /// The drive's own warning and critical temperatures, in °C.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_warning: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_critical: Option<f64>,
}

/// One whole disk.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct DiskDetail {
    pub name: String,
    pub rotational: bool,
    /// The scheduler in use, and the ones offered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduler: Option<String>,
    pub schedulers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_ahead_kb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logical_block: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub physical_block: Option<u64>,
    /// The disk accepts TRIM (discard).
    pub trim: bool,
    /// `gpt` or `dos`, from udev.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table: Option<String>,
    pub partitions: Vec<Partition>,
    /// SATA link speed now, and the most it may use, such as `6.0 Gbps`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sata_speed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sata_limit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nvme: Option<Nvme>,
}

/// One mounted local filesystem.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct FsDetail {
    pub device: String,
    pub mount: String,
    pub fstype: String,
    pub options: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inodes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inodes_free: Option<u64>,
    /// Errors ext4 recorded, and when the first and last happened (seconds since 1970).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_error_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error_at: Option<u64>,
}

/// What `storage_detail` answers.
#[derive(Debug, Default, Serialize)]
pub struct StorageDetail {
    pub disks: Vec<DiskDetail>,
    pub filesystems: Vec<FsDetail>,
    /// NVMe power saving: the longest wake-up time allowed, in µs (0 turns it off).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nvme_max_latency_us: Option<u64>,
}

/// Filesystem types worth showing; the rest (proc, tmpfs, squashfs for snaps, …) are left out.
const LOCAL: &[&str] = &[
    "ext4", "ext3", "ext2", "btrfs", "xfs", "vfat", "exfat", "ntfs", "ntfs3", "f2fs", "zfs",
];

fn text(root: &SystemRoot, path: &str) -> Option<String> {
    root.read(path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

fn number(root: &SystemRoot, path: &str) -> Option<u64> {
    text(root, path)?.parse().ok()
}

/// `[none] mq-deadline` gives the one in use and all offered.
pub fn schedulers(text: &str) -> (Option<String>, Vec<String>) {
    let all: Vec<String> = text
        .split_whitespace()
        .map(|s| s.trim_matches(['[', ']']).to_owned())
        .collect();
    let used = text
        .split_whitespace()
        .find(|s| s.starts_with('['))
        .map(|s| s.trim_matches(['[', ']']).to_owned());
    (used, all)
}

fn udev_value(root: &SystemRoot, dev: &str, key: &str) -> Option<String> {
    let data = root.read(&format!("/run/udev/data/b{dev}")).ok()?;
    data.lines()
        .find_map(|l| l.strip_prefix(&format!("E:{key}=")).map(str::to_owned))
}

fn partitions(root: &SystemRoot, disk: &str) -> Vec<Partition> {
    let base = format!("/sys/block/{disk}");
    let mut names: Vec<String> = std::fs::read_dir(root.path(&base))
        .map(|d| {
            d.flatten()
                .filter_map(|e| {
                    e.file_name()
                        .to_str()
                        .filter(|n| n.starts_with(disk) && n.len() > disk.len())
                        .map(str::to_owned)
                })
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
        .into_iter()
        .filter_map(|name| {
            let start = number(root, &format!("{base}/{name}/start"))?;
            let size = number(root, &format!("{base}/{name}/size"))? * 512;
            let offset = number(root, &format!("{base}/{name}/alignment_offset")).unwrap_or(0);
            Some(Partition {
                aligned: offset == 0 && (start * 512) % (1024 * 1024) == 0,
                start_sector: start,
                size,
                name,
            })
        })
        .collect()
}

/// The ATA port number in a disk's real path, such as `ata3` in `…/ata3/host2/…`.
fn ata_port(root: &SystemRoot, disk: &str) -> Option<String> {
    let real = std::fs::canonicalize(root.path(&format!("/sys/block/{disk}"))).ok()?;
    real.components()
        .filter_map(|c| c.as_os_str().to_str())
        .find(|c| c.starts_with("ata") && c[3..].chars().all(|d| d.is_ascii_digit()))
        .map(|c| c[3..].to_owned())
}

fn nvme(root: &SystemRoot, disk: &str) -> Option<Nvme> {
    // nvme0n1 belongs to controller nvme0.
    let controller = disk.split('n').take(2).collect::<Vec<_>>().join("n");
    let base = format!("/sys/class/nvme/{controller}");
    if !root.path(&base).exists() {
        return None;
    }
    let hwmon = std::fs::read_dir(root.path(&base))
        .ok()?
        .flatten()
        .filter_map(|e| {
            e.file_name()
                .to_str()
                .filter(|n| n.starts_with("hwmon"))
                .map(str::to_owned)
        })
        .next();
    let temp = |f: &str| {
        hwmon
            .as_ref()
            .and_then(|h| number(root, &format!("{base}/{h}/{f}")))
            .map(|m| m as f64 / 1000.0)
    };
    Some(Nvme {
        firmware: text(root, &format!("{base}/firmware_rev")),
        transport: text(root, &format!("{base}/transport")),
        state: text(root, &format!("{base}/state")),
        pci_address: text(root, &format!("{base}/address")),
        temp_warning: temp("temp1_max"),
        temp_critical: temp("temp1_crit"),
    })
}

fn disks(root: &SystemRoot) -> Vec<DiskDetail> {
    let mut names: Vec<String> = std::fs::read_dir(root.path("/sys/block"))
        .map(|d| {
            d.flatten()
                .filter_map(|e| e.file_name().to_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    // Only real disks have a device; loop, dm, zram, and RAM disks do not.
    names.retain(|n| root.path(&format!("/sys/block/{n}/device")).exists());
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let q = |f: &str| format!("/sys/block/{name}/queue/{f}");
            let (scheduler, schedulers) = text(root, &q("scheduler"))
                .map(|t| schedulers(&t))
                .unwrap_or_default();
            let link = ata_port(root, &name);
            let ata = |f: &str| {
                link.as_ref()
                    .and_then(|n| text(root, &format!("/sys/class/ata_link/link{n}/{f}")))
                    .filter(|v| v != "<unknown>")
            };
            DiskDetail {
                rotational: text(root, &q("rotational")).as_deref() == Some("1"),
                scheduler,
                schedulers,
                queue_size: number(root, &q("nr_requests")),
                read_ahead_kb: number(root, &q("read_ahead_kb")),
                logical_block: number(root, &q("logical_block_size")),
                physical_block: number(root, &q("physical_block_size")),
                trim: number(root, &q("discard_max_bytes")).is_some_and(|b| b > 0),
                table: text(root, &format!("/sys/block/{name}/dev"))
                    .and_then(|dev| udev_value(root, &dev, "ID_PART_TABLE_TYPE")),
                partitions: partitions(root, &name),
                sata_speed: ata("sata_spd"),
                sata_limit: ata("hw_sata_spd_limit").or_else(|| ata("sata_spd_limit")),
                nvme: nvme(root, &name),
                name,
            }
        })
        .collect()
}

/// `/proc/mounts` escapes spaces and tabs as `\040` and `\011`.
fn unescape_mount(path: &str) -> String {
    path.replace("\\040", " ")
        .replace("\\011", "\t")
        .replace("\\134", "\\")
}

fn filesystems(root: &SystemRoot) -> Vec<FsDetail> {
    let mounts = root.read("/proc/mounts").unwrap_or_default();
    let mut seen = std::collections::BTreeSet::new();
    mounts
        .lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split_whitespace().collect();
            let (device, mount, fstype, options) = (f.first()?, f.get(1)?, f.get(2)?, f.get(3)?);
            if !LOCAL.contains(fstype) || !seen.insert(mount.to_string()) {
                return None;
            }
            let mount = unescape_mount(mount);
            let stat = rustix::fs::statvfs(root.path(&mount).as_path()).ok();
            // ext4 keeps its counters under the device name, such as `nvme0n1p2` or `dm-1`.
            let dev_name = std::fs::canonicalize(root.path(device))
                .ok()
                .and_then(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_owned))
                .unwrap_or_else(|| device.rsplit('/').next().unwrap_or(device).to_owned());
            let ext4 = |f: &str| {
                fstype
                    .starts_with("ext")
                    .then(|| number(root, &format!("/sys/fs/ext4/{dev_name}/{f}")))
                    .flatten()
            };
            Some(FsDetail {
                device: device.to_string(),
                fstype: fstype.to_string(),
                options: options.split(',').map(str::to_owned).collect(),
                inodes: stat.as_ref().map(|s| s.f_files).filter(|n| *n > 0),
                inodes_free: stat.as_ref().filter(|s| s.f_files > 0).map(|s| s.f_ffree),
                errors: ext4("errors_count"),
                first_error_at: ext4("first_error_time").filter(|t| *t > 0),
                last_error_at: ext4("last_error_time").filter(|t| *t > 0),
                mount,
            })
        })
        .collect()
}

/// Reads everything for `storage_detail`.
pub fn read(root: &SystemRoot) -> StorageDetail {
    StorageDetail {
        disks: disks(root),
        filesystems: filesystems(root),
        nvme_max_latency_us: number(
            root,
            "/sys/module/nvme_core/parameters/default_ps_max_latency_us",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn schedulers_show_the_one_in_use() {
        assert_eq!(
            schedulers("[none] mq-deadline"),
            (
                Some("none".into()),
                vec!["none".into(), "mq-deadline".into()]
            )
        );
        assert_eq!(schedulers("none"), (None, vec!["none".into()]));
    }

    #[test]
    fn reads_an_nvme_laptop() {
        let b = "/sys/block/nvme0n1";
        let root = fixture_root(
            "storage-detail",
            &[
                (&format!("{b}/device/model"), "Samsung\n"),
                (&format!("{b}/dev"), "259:0\n"),
                (&format!("{b}/queue/rotational"), "0\n"),
                (&format!("{b}/queue/scheduler"), "[none] mq-deadline\n"),
                (&format!("{b}/queue/nr_requests"), "1023\n"),
                (&format!("{b}/queue/logical_block_size"), "512\n"),
                (&format!("{b}/queue/physical_block_size"), "512\n"),
                (&format!("{b}/queue/discard_max_bytes"), "2199023255040\n"),
                (&format!("{b}/nvme0n1p1/start"), "2048\n"),
                (&format!("{b}/nvme0n1p1/size"), "2097152\n"),
                (&format!("{b}/nvme0n1p1/alignment_offset"), "0\n"),
                (&format!("{b}/nvme0n1p2/start"), "2099263\n"),
                (&format!("{b}/nvme0n1p2/size"), "1000\n"),
                (
                    "/run/udev/data/b259:0",
                    "S:disk/by-id/x\nE:ID_PART_TABLE_TYPE=gpt\n",
                ),
                ("/sys/class/nvme/nvme0/firmware_rev", "4B2QGXA7\n"),
                ("/sys/class/nvme/nvme0/address", "0000:01:00.0\n"),
                ("/sys/class/nvme/nvme0/transport", "pcie\n"),
                ("/sys/class/nvme/nvme0/hwmon2/temp1_max", "81850\n"),
                ("/sys/class/nvme/nvme0/hwmon2/temp1_crit", "84850\n"),
                ("/sys/block/loop0/queue/rotational", "0\n"),
                (
                    "/sys/module/nvme_core/parameters/default_ps_max_latency_us",
                    "100000\n",
                ),
                (
                    "/proc/mounts",
                    "/dev/nvme0n1p1 /boot/efi vfat rw,relatime 0 0\nproc /proc proc rw 0 0\n/dev/sdb1 /media/my\\040disk ext4 rw,noatime 0 0\n",
                ),
                ("/sys/fs/ext4/sdb1/errors_count", "2\n"),
                ("/sys/fs/ext4/sdb1/first_error_time", "1700000000\n"),
                ("/sys/fs/ext4/sdb1/last_error_time", "0\n"),
            ],
        );
        let d = read(&root);
        assert_eq!(d.disks.len(), 1, "loop devices are left out");
        let n = &d.disks[0];
        assert_eq!(n.scheduler.as_deref(), Some("none"));
        assert!(n.trim && !n.rotational);
        assert_eq!(n.table.as_deref(), Some("gpt"));
        assert_eq!(
            n.partitions.iter().map(|p| p.aligned).collect::<Vec<_>>(),
            [true, false]
        );
        assert_eq!(n.partitions[0].size, 2_097_152 * 512);
        let nv = n.nvme.as_ref().unwrap();
        assert_eq!(
            (nv.firmware.as_deref(), nv.temp_warning, nv.temp_critical),
            (Some("4B2QGXA7"), Some(81.85), Some(84.85))
        );
        assert_eq!(nv.pci_address.as_deref(), Some("0000:01:00.0"));
        assert_eq!(n.sata_speed, None);
        assert_eq!(d.nvme_max_latency_us, Some(100_000));
        assert_eq!(
            d.filesystems
                .iter()
                .map(|f| f.mount.as_str())
                .collect::<Vec<_>>(),
            ["/boot/efi", "/media/my disk"]
        );
        let ext = &d.filesystems[1];
        assert_eq!(
            (ext.errors, ext.first_error_at, ext.last_error_at),
            (Some(2), Some(1_700_000_000), None)
        );
        assert!(ext.options.contains(&"noatime".to_owned()));
        assert_eq!(
            d.filesystems[0].errors, None,
            "only ext filesystems keep this count"
        );
    }
}
