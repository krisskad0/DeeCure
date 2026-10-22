// SPDX-License-Identifier: Apache-2.0

//! Rules: small YAML files that turn facts into findings.
//!
//! File format and the list of facts: `docs/02-architecture/rules-engine.md`.
//! How to write one: `docs/04-development/writing-rules-and-kb.md`.

mod engine;
pub mod expr;
pub mod facts;

use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

use deecure_core::{FixActionId, RuleId, Severity};
use serde::Deserialize;

pub use engine::{
    CHECKED_CATEGORIES_SETTING, LAST_CHECK_SETTING, Outcome, apply, evaluate, fill_in,
};
pub use facts::{Facts, gather};

use expr::Expr;

/// Where installed rules live.
pub const INSTALLED_DIR: &str = "/usr/share/deecure/rules";

/// One loaded, checked rule.
#[derive(Debug, Clone)]
pub struct Rule {
    /// Such as `storage.root_almost_full`.
    pub id: RuleId,
    /// The list to go through, one finding per entry, if any.
    pub for_each: Option<String>,
    /// One short line. May contain `{fact}` placeholders.
    pub title: String,
    /// Plain-English meaning. May contain `{fact}` placeholders.
    pub explain: String,
    /// Conditions, most serious first. The first one that is true decides.
    pub conditions: Vec<(Severity, Expr)>,
    /// Fix actions to offer, in order.
    pub fixes: Vec<FixActionId>,
    /// A knowledge-base page to link to.
    pub learn_more: Option<String>,
}

/// The YAML file, as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleFile {
    id: String,
    category: String,
    #[serde(default)]
    for_each: Option<String>,
    title: String,
    explain: String,
    severity: SeverityFile,
    #[serde(default)]
    fixes: Vec<FixRef>,
    #[serde(default)]
    cause_hint: Option<String>,
    #[serde(default)]
    learn_more: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SeverityFile {
    critical: Option<String>,
    warning: Option<String>,
    info: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FixRef {
    action: String,
}

impl Rule {
    /// Reads and checks one rule file.
    pub fn from_yaml(text: &str) -> Result<Rule, String> {
        let file: RuleFile =
            serde_saphyr::from_str(text).map_err(|e| format!("not valid rule YAML: {e}"))?;
        let id = RuleId::new(file.id).map_err(|e| format!("id: {e}"))?;
        if file.category != id.category().id() {
            return Err(format!(
                "category is '{}', but the id starts with '{}'",
                file.category,
                id.category()
            ));
        }
        if let Some(list) = &file.for_each
            && !facts::LISTS.contains(&list.as_str())
        {
            return Err(format!(
                "for_each: unknown list '{list}' (known: {})",
                facts::LISTS.join(", ")
            ));
        }

        let levels = [
            (Severity::Critical, file.severity.critical),
            (Severity::Warning, file.severity.warning),
            (Severity::Info, file.severity.info),
        ];
        let mut conditions = Vec::new();
        for (severity, source) in levels {
            let Some(source) = source else { continue };
            let condition =
                expr::parse(&source).map_err(|e| format!("severity {severity:?}: {e}"))?;
            conditions.push((severity, condition));
        }
        if conditions.is_empty() {
            return Err("severity needs at least one of critical, warning, info".to_owned());
        }

        let title = file.title.trim().to_owned();
        let explain = file.explain.trim().to_owned();
        let mut names: Vec<String> = conditions
            .iter()
            .flat_map(|(_, c)| c.fact_names())
            .map(str::to_owned)
            .collect();
        names.extend(
            placeholders(&title)
                .chain(placeholders(&explain))
                .map(str::to_owned),
        );
        if file.for_each.is_none()
            && let Some(name) = names.iter().find(|n| n.starts_with("item."))
        {
            return Err(format!(
                "'{name}' needs for_each, because only lists have items"
            ));
        }

        let fixes = file
            .fixes
            .into_iter()
            .map(|fix| FixActionId::new(fix.action).map_err(|e| format!("fixes: {e}")))
            .collect::<Result<_, _>>()?;
        // cause_hint is part of the format, but nothing uses it before Phase 2.
        let _ = file.cause_hint;

        Ok(Rule {
            id,
            for_each: file.for_each,
            title,
            explain,
            conditions,
            fixes,
            learn_more: file.learn_more,
        })
    }
}

/// The `{name}` placeholders in a text.
fn placeholders(text: &str) -> impl Iterator<Item = &str> {
    text.split('{')
        .skip(1)
        .filter_map(|part| part.split_once('}').map(|(name, _)| name))
}

/// A rule file that could not be loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleError {
    /// The file.
    pub file: PathBuf,
    /// What is wrong with it.
    pub message: String,
}

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.file.display(), self.message)
    }
}

/// Loads `<dir>/<category>/<name>.yaml` files. The file must match its id:
/// `storage/root_almost_full.yaml` holds `storage.root_almost_full`.
///
/// Broken files are reported and skipped, so one bad rule cannot stop the others.
/// The `fixes/` folder holds fix actions, not rules, and is skipped.
pub fn load_dir(dir: &Path) -> (Vec<Rule>, Vec<RuleError>) {
    let mut rules = Vec::new();
    let mut errors = Vec::new();
    let mut seen = HashSet::new();
    let error = |file: &Path, message: String| RuleError {
        file: file.to_owned(),
        message,
    };

    let categories = match sorted_entries(dir) {
        Ok(entries) => entries,
        Err(e) => {
            return (
                rules,
                vec![error(dir, format!("cannot read the rules folder: {e}"))],
            );
        }
    };
    for category_dir in categories.into_iter().filter(|p| p.is_dir()) {
        if category_dir.file_name().is_some_and(|name| name == "fixes") {
            continue;
        }
        let files = match sorted_entries(&category_dir) {
            Ok(files) => files,
            Err(e) => {
                errors.push(error(&category_dir, e.to_string()));
                continue;
            }
        };
        for file in files
            .into_iter()
            .filter(|p| p.extension().is_some_and(|ext| ext == "yaml"))
        {
            let rule = std::fs::read_to_string(&file)
                .map_err(|e| e.to_string())
                .and_then(|text| Rule::from_yaml(&text))
                .and_then(|rule| check_location(&rule, &file).map(|()| rule));
            match rule {
                Ok(rule) if !seen.insert(rule.id.clone()) => {
                    errors.push(error(&file, format!("{} is defined twice", rule.id)));
                }
                Ok(rule) => rules.push(rule),
                Err(message) => errors.push(error(&file, message)),
            }
        }
    }
    (rules, errors)
}

fn sorted_entries(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<Result<_, _>>()?;
    paths.sort();
    Ok(paths)
}

fn check_location(rule: &Rule, file: &Path) -> Result<(), String> {
    let expected = rule.id.as_str().replacen('.', "/", 1) + ".yaml";
    if file.ends_with(&expected) {
        Ok(())
    } else {
        Err(format!("this rule must live at <rules>/{expected}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"
id: power.battery_worn
category: power
for_each: batteries
title: "Battery {item.name} is worn"
explain: >
  It holds {item.health}% of its first capacity.
severity:
  warning: "item.health < 60"
  info: "item.health < 80"
learn_more: kb/power/battery-health.md
"#;

    #[test]
    fn reads_a_good_rule() {
        let rule = Rule::from_yaml(GOOD).unwrap();
        assert_eq!(rule.id.as_str(), "power.battery_worn");
        assert_eq!(rule.for_each.as_deref(), Some("batteries"));
        assert_eq!(
            rule.explain,
            "It holds {item.health}% of its first capacity."
        );
        let order: Vec<_> = rule.conditions.iter().map(|(s, _)| *s).collect();
        assert_eq!(order, [Severity::Warning, Severity::Info]);
    }

    #[test]
    fn mistakes_are_explained() {
        let cases = [
            (
                GOOD.replace("category: power", "category: storage"),
                "category is 'storage'",
            ),
            (
                GOOD.replace("for_each: batteries", "for_each: toasters"),
                "unknown list 'toasters'",
            ),
            (GOOD.replace("for_each: batteries\n", ""), "needs for_each"),
            (
                GOOD.replace("item.health < 60", "item.health <"),
                "severity Warning",
            ),
            (
                GOOD.replace("learn_more", "learn_mroe"),
                "not valid rule YAML",
            ),
            (
                GOOD.replace("id: power.battery_worn", "id: power.Battery"),
                "id:",
            ),
        ];
        for (yaml, expected) in cases {
            let error = Rule::from_yaml(&yaml).unwrap_err();
            assert!(
                error.contains(expected),
                "expected '{expected}' in: {error}"
            );
        }
    }

    #[test]
    fn a_rule_needs_a_severity() {
        let yaml = "id: power.x\ncategory: power\ntitle: t\nexplain: e\nseverity: {}\n";
        assert!(Rule::from_yaml(yaml).unwrap_err().contains("at least one"));
    }

    #[test]
    fn load_dir_checks_locations_and_skips_bad_files() {
        let dir = std::env::temp_dir().join(format!("deecure-rules-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (path, text) in [
            ("power/battery_worn.yaml", GOOD.to_owned()),
            ("power/wrong_name.yaml", GOOD.to_owned()),
            ("storage/broken.yaml", "id: [".to_owned()),
            (
                "fixes/storage/clean_apt_cache.yaml",
                "not a rule".to_owned(),
            ),
            ("power/notes.txt", "ignored".to_owned()),
        ] {
            let file = dir.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, text).unwrap();
        }
        let (rules, errors) = load_dir(&dir);
        assert_eq!(rules.len(), 1);
        assert_eq!(errors.len(), 2, "{errors:?}");
        assert!(
            errors[0]
                .message
                .contains("must live at <rules>/power/battery_worn.yaml")
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn missing_folder_is_one_error() {
        let (rules, errors) = load_dir(Path::new("/nonexistent/deecure/rules"));
        assert!(rules.is_empty());
        assert_eq!(errors.len(), 1);
    }
}
