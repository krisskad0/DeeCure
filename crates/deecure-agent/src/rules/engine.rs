// SPDX-License-Identifier: Apache-2.0

//! Runs rules against facts and keeps the stored findings in step.

use std::collections::HashSet;

use deecure_core::{Category, Finding, FindingState, Severity, Timestamp};

use super::Rule;
use super::expr::{EvalError, Lookup};
use super::facts::{Facts, ItemScope};
use crate::store::{self, Store};

/// The setting that holds when the rules last ran.
pub const LAST_CHECK_SETTING: &str = "rules.checked_at";
/// The setting that holds the categories that have rules.
pub const CHECKED_CATEGORIES_SETTING: &str = "rules.categories";

/// What one rule decided about one subject.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// The problem is there.
    Fired {
        /// How serious it is.
        severity: Severity,
        /// The title, with facts filled in.
        title: String,
        /// The explanation, with facts filled in.
        explain: String,
    },
    /// The problem is not there.
    Clear,
    /// Some fact is not known right now, so nothing changes.
    Unknown,
    /// The rule itself has a mistake, such as comparing text with a number.
    Broken(String),
}

/// Decides one rule. Returns one outcome per subject, or `None` when the rule goes
/// through a list that is not known right now (nothing should change).
pub fn evaluate(rule: &Rule, facts: &Facts) -> Option<Vec<(Option<String>, Outcome)>> {
    match &rule.for_each {
        None => Some(vec![(None, decide(rule, facts))]),
        Some(list) => {
            let items = facts.lists.get(list)?;
            Some(
                items
                    .iter()
                    .map(|item| {
                        let scope = ItemScope { facts, item };
                        (Some(item.subject.clone()), decide(rule, &scope))
                    })
                    .collect(),
            )
        }
    }
}

fn decide(rule: &Rule, facts: &dyn Lookup) -> Outcome {
    for (severity, condition) in &rule.conditions {
        match condition.eval_bool(facts) {
            Ok(true) => {
                return Outcome::Fired {
                    severity: *severity,
                    title: fill_in(&rule.title, facts),
                    explain: fill_in(&rule.explain, facts),
                };
            }
            Ok(false) => {}
            Err(EvalError::Missing(_)) => return Outcome::Unknown,
            Err(EvalError::Type(why)) => return Outcome::Broken(why),
        }
    }
    Outcome::Clear
}

/// Replaces `{fact.name}` with the fact's value, or "unknown".
pub fn fill_in(template: &str, facts: &dyn Lookup) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        let Some(len) = rest[start..].find('}') else {
            break;
        };
        out.push_str(&rest[..start]);
        let name = &rest[start + 1..start + len];
        let value = facts
            .get(name)
            .map_or_else(|| "unknown".to_owned(), |v| v.display());
        out.push_str(&value);
        rest = &rest[start + len + 1..];
    }
    out.push_str(rest);
    out
}

/// Runs every rule and updates the stored findings:
///
/// - a rule that fires opens or updates its finding (a dismissed one stays dismissed);
/// - a rule that is clear resolves its open finding;
/// - a `for_each` rule resolves open findings whose subject is gone from the list;
/// - an unknown outcome changes nothing.
///
/// Also records `now` as the time of the last check ([`LAST_CHECK_SETTING`]).
///
/// Returns the mistakes found in rules, for the caller to log.
pub fn apply(
    store: &Store,
    rules: &[Rule],
    facts: &Facts,
    now: Timestamp,
) -> store::Result<Vec<String>> {
    let open: HashSet<(String, Option<String>)> = store
        .open_findings()?
        .into_iter()
        .map(|f| (f.rule.to_string(), f.subject))
        .collect();
    let mut mistakes = Vec::new();

    for rule in rules {
        let Some(results) = evaluate(rule, facts) else {
            continue;
        };
        let mut seen = HashSet::new();
        for (subject, outcome) in results {
            let key = (rule.id.to_string(), subject.clone());
            match outcome {
                Outcome::Fired {
                    severity,
                    title,
                    explain,
                } => {
                    let dismissed = store
                        .finding(&rule.id, subject.as_deref())?
                        .is_some_and(|f| f.state == FindingState::Dismissed);
                    if !dismissed {
                        store.upsert_finding(&Finding {
                            rule: rule.id.clone(),
                            subject: subject.clone(),
                            severity,
                            state: FindingState::Open,
                            title,
                            explain,
                            first_seen: now,
                            last_seen: now,
                            fixes: rule.fixes.clone(),
                        })?;
                    }
                }
                Outcome::Clear if open.contains(&key) => {
                    store.set_finding_state(
                        &rule.id,
                        subject.as_deref(),
                        FindingState::Resolved,
                    )?;
                }
                Outcome::Clear | Outcome::Unknown => {}
                Outcome::Broken(why) => mistakes.push(format!("rule {}: {why}", rule.id)),
            }
            seen.insert(subject);
        }
        if rule.for_each.is_some() {
            for (_, subject) in open
                .iter()
                .filter(|(id, s)| *id == rule.id.as_str() && !seen.contains(s))
            {
                store.set_finding_state(&rule.id, subject.as_deref(), FindingState::Resolved)?;
            }
        }
    }
    store.set_setting(LAST_CHECK_SETTING, &now)?;
    let mut categories: Vec<Category> = rules.iter().map(|r| r.id.category()).collect();
    categories.sort();
    categories.dedup();
    store.set_setting(CHECKED_CATEGORIES_SETTING, &categories)?;
    Ok(mistakes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Rule;
    use crate::rules::expr::Value;
    use crate::rules::facts::Item;
    use std::collections::BTreeMap;

    fn root_rule() -> Rule {
        Rule::from_yaml(
            r#"
id: storage.root_almost_full
category: storage
title: "Main disk is almost full"
explain: "Only {root.free_gb} GB left."
severity:
  critical: "root.used_percent >= 93"
  warning: "root.used_percent >= 85"
fixes:
  - action: storage.clean_apt_cache
"#,
        )
        .unwrap()
    }

    fn service_rule() -> Rule {
        Rule::from_yaml(
            r#"
id: services.unit_failed
category: services
for_each: services
title: "{item.name} has stopped"
explain: "It failed."
severity:
  warning: "item.active == 'failed'"
"#,
        )
        .unwrap()
    }

    fn disk(used: f64, free_gb: f64) -> Facts {
        let mut facts = Facts::default();
        facts
            .values
            .insert("root.used_percent".into(), Value::Number(used));
        facts
            .values
            .insert("root.free_gb".into(), Value::Number(free_gb));
        facts
    }

    fn services(states: &[(&str, &str)]) -> Facts {
        let items = states
            .iter()
            .map(|(name, active)| Item {
                subject: (*name).into(),
                fields: BTreeMap::from([
                    ("name".into(), Value::Text((*name).into())),
                    ("active".into(), Value::Text((*active).into())),
                ]),
            })
            .collect();
        Facts {
            lists: BTreeMap::from([("services".into(), items)]),
            ..Facts::default()
        }
    }

    #[test]
    fn most_serious_matching_level_wins_and_text_is_filled_in() {
        let outcome = evaluate(&root_rule(), &disk(94.2, 13.84)).unwrap();
        assert_eq!(
            outcome,
            [(
                None,
                Outcome::Fired {
                    severity: Severity::Critical,
                    title: "Main disk is almost full".into(),
                    explain: "Only 13.8 GB left.".into(),
                }
            )]
        );
        let warning = evaluate(&root_rule(), &disk(88.0, 30.0)).unwrap();
        assert!(matches!(
            warning[0].1,
            Outcome::Fired {
                severity: Severity::Warning,
                ..
            }
        ));
        assert_eq!(
            evaluate(&root_rule(), &disk(40.0, 300.0)).unwrap()[0].1,
            Outcome::Clear
        );
        assert_eq!(
            evaluate(&root_rule(), &Facts::default()).unwrap()[0].1,
            Outcome::Unknown
        );
    }

    #[test]
    fn for_each_gives_one_outcome_per_item_and_none_without_the_list() {
        let results = evaluate(
            &service_rule(),
            &services(&[("a.service", "active"), ("b.service", "failed")]),
        )
        .unwrap();
        assert_eq!(results[0], (Some("a.service".into()), Outcome::Clear));
        assert!(
            matches!(&results[1], (Some(s), Outcome::Fired { title, .. }) if s == "b.service" && title == "b.service has stopped")
        );
        assert!(evaluate(&service_rule(), &Facts::default()).is_none());
    }

    #[test]
    fn findings_open_resolve_and_respect_dismissal() {
        let store = Store::open_in_memory().unwrap();
        let rules = [root_rule()];

        assert_eq!(
            store.setting::<Timestamp>(LAST_CHECK_SETTING).unwrap(),
            None
        );
        apply(&store, &rules, &disk(95.0, 10.0), Timestamp(1)).unwrap();
        assert_eq!(
            store.setting(LAST_CHECK_SETTING).unwrap(),
            Some(Timestamp(1))
        );
        assert_eq!(
            store
                .setting::<Vec<Category>>(CHECKED_CATEGORIES_SETTING)
                .unwrap(),
            Some(vec![Category::Storage])
        );
        let open = store.open_findings().unwrap();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].fixes[0].as_str(), "storage.clean_apt_cache");

        // Unknown changes nothing.
        apply(&store, &rules, &Facts::default(), Timestamp(2)).unwrap();
        assert_eq!(store.open_findings().unwrap().len(), 1);

        // Clear resolves it.
        apply(&store, &rules, &disk(50.0, 100.0), Timestamp(3)).unwrap();
        assert!(store.open_findings().unwrap().is_empty());

        // A dismissed finding stays dismissed even when the rule fires again.
        apply(&store, &rules, &disk(95.0, 10.0), Timestamp(4)).unwrap();
        store
            .set_finding_state(&rules[0].id, None, FindingState::Dismissed)
            .unwrap();
        apply(&store, &rules, &disk(96.0, 9.0), Timestamp(5)).unwrap();
        assert!(store.open_findings().unwrap().is_empty());
    }

    #[test]
    fn finding_for_a_vanished_item_is_resolved() {
        let store = Store::open_in_memory().unwrap();
        let rules = [service_rule()];
        apply(
            &store,
            &rules,
            &services(&[("b.service", "failed")]),
            Timestamp(1),
        )
        .unwrap();
        assert_eq!(store.open_findings().unwrap().len(), 1);
        apply(
            &store,
            &rules,
            &services(&[("a.service", "active")]),
            Timestamp(2),
        )
        .unwrap();
        assert!(store.open_findings().unwrap().is_empty());
    }

    #[test]
    fn rule_mistakes_are_reported_not_applied() {
        let store = Store::open_in_memory().unwrap();
        let broken = Rule::from_yaml(
            r#"
id: storage.broken
category: storage
title: t
explain: e
severity:
  warning: "root.used_percent == 'full'"
"#,
        )
        .unwrap();
        let mistakes = apply(&store, &[broken], &disk(95.0, 1.0), Timestamp(1)).unwrap();
        assert_eq!(mistakes.len(), 1);
        assert!(mistakes[0].starts_with("rule storage.broken"));
        assert!(store.open_findings().unwrap().is_empty());
    }

    #[test]
    fn fill_in_handles_missing_and_unclosed_names() {
        let facts = disk(94.0, 13.8);
        assert_eq!(
            fill_in("{root.free_gb} GB, {nope}", &facts),
            "13.8 GB, unknown"
        );
        assert_eq!(fill_in("50% {not closed", &facts), "50% {not closed");
    }
}
