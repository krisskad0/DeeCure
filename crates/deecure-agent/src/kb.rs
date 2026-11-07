// SPDX-License-Identifier: Apache-2.0

//! The knowledge base: plain-English explanations of services, ports, errors, packages,
//! and how-to topics, from YAML files in `kb/<type>/` (docs/02-architecture/knowledge-base.md).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// A systemd service, from `kb/services/<unit>.yaml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Service {
    /// The unit, such as `cups.service`.
    pub name: String,
    /// What it does, in one sentence.
    pub plain: String,
    /// Whether turning it off is safe, and when.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_to_disable: Option<String>,
    /// What stops working when it fails.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub if_failed: Option<String>,
}

/// A network port, from `kb/ports/<number>-<proto>.yaml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Port {
    /// The port number.
    pub port: u16,
    /// `tcp` or `udp`.
    pub proto: String,
    /// What usually listens there, such as "SSH".
    pub name: String,
    /// What it is, in one sentence.
    pub plain: String,
    /// `low`, `medium`, or `high` when it is open to other computers.
    pub risk_if_open: String,
}

/// A log message pattern, from `kb/errors/<name>.yaml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorPattern {
    /// Text the message contains.
    #[serde(rename = "match")]
    pub pattern: String,
    /// What it means, in one sentence.
    pub plain: String,
    /// `info`, `warning`, or `critical`.
    pub severity: String,
    /// What to do about it.
    pub next_step: String,
}

/// A package, from `kb/packages/<name>.yaml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Package {
    /// The package name.
    pub name: String,
    /// What it is, in one sentence.
    pub plain: String,
}

/// A how-to article, from `kb/topics/<area>/<name>.yaml`. Rules link to it with `learn_more`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Topic {
    /// `topics/<area>/<name>`, from the file path.
    #[serde(default)]
    pub id: String,
    /// The article title.
    pub title: String,
    /// Why it matters, in a sentence or two.
    pub summary: String,
    /// What to do, step by step.
    #[serde(default)]
    pub steps: Vec<String>,
    /// Anything else worth knowing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Everything in the knowledge base.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Kb {
    /// Services.
    pub services: Vec<Service>,
    /// Ports.
    pub ports: Vec<Port>,
    /// Log message patterns.
    pub errors: Vec<ErrorPattern>,
    /// Packages.
    pub packages: Vec<Package>,
    /// How-to articles.
    pub topics: Vec<Topic>,
    /// Rule ID → topic ID, from each rule's `learn_more`.
    pub rules: BTreeMap<String, String>,
}

/// Where the knowledge base sits next to the rules: `/usr/share/deecure/kb` when installed.
pub fn beside(rules_dir: &Path) -> PathBuf {
    rules_dir
        .parent()
        .map_or_else(|| PathBuf::from("kb"), |p| p.join("kb"))
}

fn yaml_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d)
            .map(|r| r.flatten().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "yaml") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

fn load<T: for<'de> Deserialize<'de>>(dir: &Path, errors: &mut Vec<String>) -> Vec<(PathBuf, T)> {
    yaml_files(dir)
        .into_iter()
        .filter_map(|f| {
            let parsed = std::fs::read_to_string(&f)
                .map_err(|e| e.to_string())
                .and_then(|t| serde_saphyr::from_str::<T>(&t).map_err(|e| e.to_string()));
            match parsed {
                Ok(v) => Some((f, v)),
                Err(e) => {
                    errors.push(format!("{}: {e}", f.display()));
                    None
                }
            }
        })
        .collect()
}

/// Loads the knowledge base. Broken files are reported and skipped. `learn_more` holds
/// each rule's topic link; links to missing topics are reported too.
pub fn load_dir(dir: &Path, learn_more: &[(String, String)]) -> (Kb, Vec<String>) {
    let mut errors = Vec::new();
    let topics_dir = dir.join("topics");
    let topics: Vec<Topic> = load::<Topic>(&topics_dir, &mut errors)
        .into_iter()
        .map(|(f, mut t)| {
            t.id = f
                .strip_prefix(dir)
                .unwrap_or(&f)
                .with_extension("")
                .to_string_lossy()
                .into_owned();
            t
        })
        .collect();
    let mut rules = BTreeMap::new();
    for (rule, topic) in learn_more {
        if topics.iter().any(|t| &t.id == topic) {
            rules.insert(rule.clone(), topic.clone());
        } else {
            errors.push(format!("rule {rule} links to a missing topic: {topic}"));
        }
    }
    let kb = Kb {
        services: load(&dir.join("services"), &mut errors)
            .into_iter()
            .map(|(_, v)| v)
            .collect(),
        ports: load(&dir.join("ports"), &mut errors)
            .into_iter()
            .map(|(_, v)| v)
            .collect(),
        errors: load(&dir.join("errors"), &mut errors)
            .into_iter()
            .map(|(_, v)| v)
            .collect(),
        packages: load(&dir.join("packages"), &mut errors)
            .into_iter()
            .map(|(_, v)| v)
            .collect(),
        topics,
        rules,
    };
    (kb, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_the_shipped_knowledge_base() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../kb");
        let links = [
            (
                "storage.root_almost_full".to_owned(),
                "topics/storage/disk-full".to_owned(),
            ),
            ("x.y".to_owned(), "topics/none".to_owned()),
        ];
        let (kb, errors) = load_dir(&dir, &links);
        assert_eq!(errors, ["rule x.y links to a missing topic: topics/none"]);
        assert!(kb.services.iter().any(|s| s.name == "cups.service"));
        assert!(kb.ports.iter().any(|p| p.port == 22 && p.proto == "tcp"));
        assert!(kb.errors.iter().any(|e| e.pattern == "EXT4-fs error"));
        assert_eq!(
            kb.rules.get("storage.root_almost_full").map(String::as_str),
            Some("topics/storage/disk-full")
        );
        assert!(
            kb.topics
                .iter()
                .all(|t| t.id.starts_with("topics/") && !t.steps.is_empty())
        );
    }

    #[test]
    fn every_shipped_rule_links_to_a_real_topic() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (rules, _) = crate::rules::load_dir(&root.join("rules"));
        let links: Vec<(String, String)> = rules
            .iter()
            .filter_map(|r| Some((r.id.to_string(), r.learn_more.clone()?)))
            .collect();
        assert_eq!(links.len(), rules.len(), "every rule has a learn_more link");
        let (_, errors) = load_dir(&root.join("kb"), &links);
        assert!(errors.is_empty(), "{errors:?}");
    }

    /// The writing rules from docs/04-development/writing-rules-and-kb.md that a computer
    /// can check, so a contributor sees a broken one in CI, not in review.
    #[test]
    fn shipped_entries_follow_the_writing_rules() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../kb");
        let mut bad = Vec::new();
        let stem = |f: &Path| f.file_stem().unwrap().to_string_lossy().into_owned();
        let sentence = |bad: &mut Vec<String>, f: &Path, field: &str, text: &str| {
            if !text.ends_with('.') {
                bad.push(format!(
                    "{}: {field} must be a full sentence ending in a full stop",
                    f.display()
                ));
            }
        };
        for f in yaml_files(&dir) {
            let text = std::fs::read_to_string(&f).unwrap();
            if text.contains(['\u{2013}', '\u{2014}']) {
                bad.push(format!(
                    "{}: use a colon, comma, or full stop instead of a long dash",
                    f.display()
                ));
            }
        }
        let mut errors = Vec::new();
        for (f, s) in load::<Service>(&dir.join("services"), &mut errors) {
            if s.name != format!("{}.service", stem(&f)) {
                bad.push(format!(
                    "{}: name must be \"{}.service\"",
                    f.display(),
                    stem(&f)
                ));
            }
            sentence(&mut bad, &f, "plain", &s.plain);
            for (field, v) in [
                ("safe_to_disable", &s.safe_to_disable),
                ("if_failed", &s.if_failed),
            ] {
                if let Some(v) = v {
                    sentence(&mut bad, &f, field, v);
                }
            }
        }
        for (f, p) in load::<Port>(&dir.join("ports"), &mut errors) {
            if stem(&f) != format!("{}-{}", p.port, p.proto) {
                bad.push(format!(
                    "{}: file name must be \"{}-{}.yaml\"",
                    f.display(),
                    p.port,
                    p.proto
                ));
            }
            if !["tcp", "udp"].contains(&p.proto.as_str()) {
                bad.push(format!("{}: proto must be tcp or udp", f.display()));
            }
            if !["low", "medium", "high"].contains(&p.risk_if_open.as_str()) {
                bad.push(format!(
                    "{}: risk_if_open must be low, medium, or high",
                    f.display()
                ));
            }
            sentence(&mut bad, &f, "plain", &p.plain);
        }
        let mut matches = BTreeMap::new();
        for (f, e) in load::<ErrorPattern>(&dir.join("errors"), &mut errors) {
            if !["info", "warning", "critical"].contains(&e.severity.as_str()) {
                bad.push(format!(
                    "{}: severity must be info, warning, or critical",
                    f.display()
                ));
            }
            if let Some(other) = matches.insert(e.pattern.clone(), f.clone()) {
                bad.push(format!(
                    "{}: same match as {}",
                    f.display(),
                    other.display()
                ));
            }
            sentence(&mut bad, &f, "plain", &e.plain);
            sentence(&mut bad, &f, "next_step", &e.next_step);
        }
        for (f, p) in load::<Package>(&dir.join("packages"), &mut errors) {
            if p.name != stem(&f) {
                bad.push(format!("{}: name must be \"{}\"", f.display(), stem(&f)));
            }
            sentence(&mut bad, &f, "plain", &p.plain);
        }
        assert!(errors.is_empty(), "{errors:?}");
        assert!(bad.is_empty(), "\n{}", bad.join("\n"));
    }

    #[test]
    fn finds_the_folder_beside_the_rules() {
        assert_eq!(
            beside(Path::new("/usr/share/deecure/rules")),
            PathBuf::from("/usr/share/deecure/kb")
        );
        assert_eq!(beside(Path::new("rules")), PathBuf::from("kb"));
    }
}
