// SPDX-License-Identifier: Apache-2.0

//! Drivers and firmware (Optimize batch 2): the hardware video driver, a second graphics card
//! that stays awake, drivers Ubuntu recommends, firmware that failed to load, firmware updates,
//! and extra drivers (DKMS) missing for an installed kernel.

use std::time::Duration;

use deecure_core::SnapshotKind;

use super::super::{command, firmware, gpu_detail, hw_errors, kernel_detail, pci_detail};
use super::{Check, advice, fix};
use crate::context::Agent;
use crate::store;

/// The VA-API drivers that fit a kernel graphics driver, with the package of each.
pub fn video_packages(driver: &str) -> &'static [(&'static str, &'static str)] {
    match driver {
        // Intel graphics from 2014 on (Broadwell and newer) use iHD; i965 is for older ones.
        "i915" | "xe" => &[("iHD", "intel-media-va-driver"), ("i965", "i965-va-driver")],
        "amdgpu" => &[("radeonsi", "mesa-va-drivers")],
        "radeon" => &[("r600", "mesa-va-drivers"), ("radeonsi", "mesa-va-drivers")],
        "nouveau" => &[("nouveau", "mesa-va-drivers")],
        _ => &[],
    }
}

async fn installed(package: &str) -> bool {
    command::output(
        "dpkg-query",
        &["-W", "-f=${db:Status-Abbrev}", "--", package],
        Duration::from_secs(5),
    )
    .await
    .is_some_and(|s| s.starts_with("ii"))
}

pub async fn checks(agent: &Agent) -> Vec<Check> {
    let root = &agent.root;
    let mut out = Vec::new();
    let gpu = gpu_detail::read(root);
    let stack = gpu_detail::read_stack(root).await;

    // Hardware video decoding.
    for card in gpu.cards.iter().filter(|c| c.driver.is_some()) {
        let driver = card.driver.as_deref().unwrap_or("");
        let fitting = video_packages(driver);
        if fitting.is_empty() {
            continue;
        }
        let has = fitting
            .iter()
            .any(|(va, _)| stack.video_drivers.iter().any(|v| v == va));
        let (va, package) = fitting[0];
        out.push(fix(
            &format!("graphics.video_driver:{}", card.card),
            "Install the hardware video driver",
            if has {
                format!("{} ({driver}) has its video-decoding driver.", card.card)
            } else {
                format!("{} ({driver}) can decode video, but no fitting video-decoding driver ({va}) is installed, so video plays on the CPU and uses more battery.", card.card)
            },
            !has,
            "packages.install_driver",
            Some(package.to_owned()),
            "admin",
        ));
    }

    // A second graphics card that stays awake.
    let pci = pci_detail::read(root);
    for card in gpu
        .cards
        .iter()
        .filter(|c| !c.boot_vga && c.pci_address.is_some())
    {
        let address = card.pci_address.clone().unwrap_or_default();
        let users: Vec<String> = gpu
            .clients
            .iter()
            .filter(|c| c.pci_address.as_deref() == Some(address.as_str()))
            .map(|c| c.name.clone())
            .collect();
        let screens = card.connectors.iter().any(|(_, s)| s == "connected");
        let awake = card.runtime_status.as_deref() == Some("active");
        let dev = pci.iter().find(|d| d.address == address);
        let saving_off = dev.is_some_and(|d| d.power_control.as_deref() == Some("on"));
        let title = "Let the second graphics card sleep";
        let id = format!("graphics.second_card:{address}");
        if !awake {
            out.push(fix(
                &id,
                title,
                format!("{} sleeps when no app uses it.", card.card),
                false,
                "",
                None,
                "admin",
            ));
        } else if screens {
            out.push(fix(
                &id,
                title,
                format!(
                    "{} is awake because a screen is connected to it.",
                    card.card
                ),
                false,
                "",
                None,
                "admin",
            ));
        } else if !users.is_empty() {
            out.push(advice(
                &id,
                title,
                format!("{} is awake because {} uses it.", card.card, users.join(", ")),
                true,
                "Close those apps when you do not need them, or start them on the main graphics card; the second card then sleeps and saves battery.",
            ));
        } else if saving_off {
            out.push(fix(
                &id,
                title,
                format!(
                    "{} is awake while no app uses it, because its power saving is off.",
                    card.card
                ),
                true,
                "power.set_device_power_saving",
                dev.and_then(|d| d.sys_path.clone())
                    .map(|p| format!("{p}|on")),
                "admin",
            ));
        } else {
            out.push(advice(
                &id,
                title,
                format!("{} is awake while no app of yours uses it, and its power saving is on.", card.card),
                true,
                "Another user's app, or its driver, keeps it awake. With NVIDIA, power saving needs the driver's runtime power management (nvidia-prime on-demand mode).",
            ));
        }
    }

    // Drivers Ubuntu recommends, and firmware that failed to load.
    let errors = hw_errors::read(root).await;
    for offer in errors.driver_offers.unwrap_or_default() {
        let Some((package, _)) = offer.drivers.iter().find(|(_, rec)| *rec) else {
            continue;
        };
        let name = offer
            .model
            .clone()
            .or(offer.vendor.clone())
            .unwrap_or_else(|| "a device".into());
        let has = installed(package).await;
        out.push(fix(
            &format!("drivers.recommended:{package}"),
            "Install the recommended driver",
            if has {
                format!("{package}, which Ubuntu recommends for {name}, is installed.")
            } else {
                format!("Ubuntu recommends {package} for {name}, and it is not installed.")
            },
            !has,
            "packages.install_driver",
            Some(package.clone()),
            "admin",
        ));
    }
    let failed = errors
        .kinds
        .as_ref()
        .and_then(|k| k.iter().find(|k| k.kind == "firmware_missing"))
        .filter(|k| k.count > 0);
    if let Some(f) = failed {
        let update: bool = store::lock(&agent.store)
            .latest_snapshot(SnapshotKind::Updates)
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_value::<crate::collect::UpdatesInfo>(s.data).ok())
            .is_some_and(|u| u.pending.iter().any(|p| p.name == "linux-firmware"));
        let example = f.examples.first().cloned().unwrap_or_default();
        out.push(fix(
            "drivers.firmware_missing",
            "Install missing firmware",
            format!(
                "Firmware failed to load {} times in the last 7 days ({example}). {}",
                f.count,
                if update {
                    "A newer linux-firmware is waiting, which may have it."
                } else {
                    "No newer linux-firmware is offered, so updating will not help."
                }
            ),
            update,
            "system.install_updates",
            Some("linux-firmware".into()),
            "admin",
        ));
    }

    // Firmware updates from fwupd.
    let fw = firmware::read(agent).await;
    for d in fw
        .devices
        .iter()
        .filter(|d| d.available.is_some() && d.id.is_some())
    {
        out.push(fix(
            &format!("firmware.update:{}", d.id.as_deref().unwrap_or("")),
            &format!("Install the firmware update for {}", d.name),
            format!(
                "{} runs firmware {} and {} is offered. Some firmware cannot go back to an older version, and some updates finish during a restart.",
                d.name,
                d.version.as_deref().unwrap_or("?"),
                d.available.as_deref().unwrap_or("?")
            ),
            true,
            "system.update_firmware",
            d.id.clone(),
            "you",
        ));
    }

    // Extra drivers (DKMS) not built for an installed kernel.
    let k = kernel_detail::read(root).await;
    if let Some(dkms) = &k.dkms {
        for m in dkms {
            // Only kernels that can start: a modules folder can outlive its removed kernel.
            let missing: Vec<&String> = k
                .kernels
                .iter()
                .filter(|kernel| root.path(&format!("/boot/vmlinuz-{kernel}")).exists())
                .filter(|kernel| !m.kernels.contains(kernel))
                .collect();
            out.push(fix(
                &format!("drivers.dkms:{}", m.module),
                &format!("Rebuild the {} driver", m.module),
                if missing.is_empty() {
                    format!("{} {} is built for every installed kernel.", m.module, m.version)
                } else {
                    format!("{} {} is not built for kernel {}, so it is missing when that kernel starts.", m.module, m.version, missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "))
                },
                !missing.is_empty(),
                "system.rebuild_dkms",
                missing.first().map(|s| (*s).clone()),
                "admin",
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn video_drivers_fit_the_graphics_driver() {
        assert_eq!(video_packages("i915")[0], ("iHD", "intel-media-va-driver"));
        assert_eq!(video_packages("amdgpu")[0].1, "mesa-va-drivers");
        assert!(video_packages("nvidia").is_empty());
    }
}
