// SPDX-License-Identifier: Apache-2.0

//! Safety checks for clean-ups that the agent offers and the root helper runs, shared so both
//! sides use the same rule (docs/01-product/optimize-catalog.md).

/// Why purging the leftover settings of a removed package is not safe, or `None`.
///
/// `postrm` is the package's removal script, which a purge still runs. Two cases are unsafe:
///
/// - Debian's `docker.io` saved `nuke-graph-directory.sh` on removal and runs it on purge: it
///   deletes everything in `/var/lib/docker`, which `docker-ce` uses too.
/// - The script turns off a systemd unit (`deb-systemd-helper purge '<unit>'`) that exists
///   again (`unit_present`): an installed package, such as `containerd.io` after `containerd`,
///   now uses that unit, and the purge would stop it from starting.
pub fn purge_risk(postrm: &str, unit_present: impl Fn(&str) -> bool) -> Option<String> {
    if postrm.contains("nuke-graph-directory") {
        return Some(
            "its removal script deletes /var/lib/docker, which Docker may still use".into(),
        );
    }
    for line in postrm.lines() {
        let Some(rest) = line.trim().strip_prefix("deb-systemd-helper purge ") else {
            continue;
        };
        for unit in rest.split_whitespace() {
            let unit = unit.trim_matches(|c| c == '\'' || c == '"');
            let is_unit = [
                ".service", ".socket", ".timer", ".path", ".mount", ".target",
            ]
            .iter()
            .any(|s| unit.ends_with(s));
            if is_unit && unit_present(unit) {
                return Some(format!(
                    "its removal script would turn off {unit}, which an installed package uses"
                ));
            }
        }
    }
    None
}

/// The old revisions in `snap list --all`, as `(name, revision)`: rows marked `disabled` of a
/// snap that also has a revision in use. A snap someone turned off (`snap disable`) shows its
/// only revision as `disabled` too; removing that revision would remove the snap, so it is
/// never an old revision.
pub fn old_snap_revisions(list: &str) -> Vec<(String, String)> {
    let rows: Vec<(String, String, bool)> = list
        .lines()
        .skip(1)
        .filter_map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            let disabled = f
                .last()
                .is_some_and(|notes| notes.split(',').any(|n| n == "disabled"));
            Some(((*f.first()?).to_owned(), (*f.get(2)?).to_owned(), disabled))
        })
        .collect();
    rows.iter()
        .filter(|(name, _, disabled)| *disabled && rows.iter().any(|(n, _, d)| n == name && !d))
        .map(|(name, rev, _)| (name.clone(), rev.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docker_io_purge_is_never_safe() {
        let postrm = "case \"$1\" in\n\tpurge)\n\t\tif [ -x /var/lib/docker/nuke-graph-directory.sh ]; then\n\t\t\t/var/lib/docker/nuke-graph-directory.sh /var/lib/docker\n\t\tfi\n\t\t;;\nesac\n";
        assert!(
            purge_risk(postrm, |_| false)
                .unwrap()
                .contains("/var/lib/docker")
        );
    }

    #[test]
    fn a_purge_that_turns_off_a_unit_in_use_is_not_safe() {
        let postrm = "if [ \"$1\" = \"purge\" ]; then\n\tif [ -x \"/usr/bin/deb-systemd-helper\" ]; then\n\t\tdeb-systemd-helper purge 'containerd.service' >/dev/null || true\n\tfi\nfi\n";
        let risk = purge_risk(postrm, |u| u == "containerd.service").unwrap();
        assert!(risk.contains("containerd.service"));
        // Nothing uses the unit any more: the purge is safe.
        assert_eq!(purge_risk(postrm, |_| false), None);
    }

    #[test]
    fn ordinary_purges_are_safe() {
        assert_eq!(purge_risk("", |_| true), None);
        let fan =
            "purge)\n\trm -f /etc/network/fan\n\t;;\nupdate-rc.d ubuntu-fan remove >/dev/null\n";
        assert_eq!(purge_risk(fan, |_| true), None);
    }

    #[test]
    fn a_turned_off_snap_is_not_an_old_revision() {
        let list = "Name    Version  Rev   Tracking       Publisher   Notes\n\
                    firefox 156.0    8969  latest/stable  mozilla**   disabled\n\
                    firefox 157.0    9012  latest/stable  mozilla**   -\n\
                    docker  28.1.1   3265  latest/stable  canonical** disabled\n\
                    snapd   2.76.3   27738 latest/stable  canonical** snapd,disabled\n\
                    snapd   2.77.1   27900 latest/stable  canonical** snapd\n";
        assert_eq!(
            old_snap_revisions(list),
            [
                ("firefox".to_owned(), "8969".to_owned()),
                ("snapd".to_owned(), "27738".to_owned())
            ]
        );
        assert!(old_snap_revisions("").is_empty());
    }
}
