// SPDX-License-Identifier: Apache-2.0

//! Turns one request line into one response. No I/O here, so it is easy to test.

use deecure_core::rpc::{
    EventList, EventParams, EventsParams, FixPlanParams, FixRecordParams, FixUndoneParams,
    InventoryParams, MAX_EVENTS, MAX_MESSAGE_BYTES, MAX_POINTS, MetricsParams, PROTOCOL_VERSION,
    PackageDetailParams, ProcessDetailParams, Request, RequestId, Response, RpcError,
    SetNotifySettingsParams, SetUpdateSettingsParams, SetWarningSettingsParams, SnapshotParams,
    StatusReport, VersionInfo, WarningLimit, WarningRule, WarningSettings, method,
};
use deecure_core::{Timestamp, VERSION, health, thin};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::context::Agent;
use crate::inventory;
use crate::rules::facts::FRESH_FOR_MS;

/// The rules from the folder the agent loaded them from.
fn load_rules(agent: &Agent) -> Vec<crate::rules::Rule> {
    agent
        .facts
        .get()
        .map(|f| crate::rules::load_dir(&f.rules_dir).0)
        .unwrap_or_default()
}

/// Every warning with its limits and the user's choices, for Settings.
fn warning_rules(agent: &Agent, settings: &WarningSettings) -> Vec<WarningRule> {
    load_rules(agent)
        .iter()
        .map(|rule| WarningRule {
            id: rule.id.to_string(),
            name: rule.name.clone(),
            category: rule.id.category().id().to_owned(),
            off: settings.off.contains(rule.id.as_str()),
            limits: rule
                .limits
                .iter()
                .map(|(name, l)| WarningLimit {
                    name: name.clone(),
                    label: l.label.clone(),
                    unit: l.unit.clone(),
                    default: l.default,
                    value: crate::rules::limit_value(rule, name, settings).unwrap_or(l.default),
                    min: l.min,
                    max: l.max,
                })
                .collect(),
        })
        .collect()
}
use crate::rules::{CHECKED_CATEGORIES_SETTING, LAST_CHECK_SETTING};
use crate::store::{self, StoreError};

/// Reads one JSON-RPC message and answers it.
pub async fn handle_line(line: &[u8], agent: &Agent) -> Response {
    let value: Value = match serde_json::from_slice(line) {
        Ok(value) => value,
        Err(e) => {
            let error = RpcError::new(RpcError::PARSE_ERROR, format!("not valid JSON: {e}"));
            return Response::failure(None, error);
        }
    };
    // Keep the ID, if it is readable, so even a bad request gets a matching answer.
    let id = value
        .get("id")
        .and_then(|id| serde_json::from_value::<RequestId>(id.clone()).ok());
    match serde_json::from_value::<Request>(value) {
        Ok(request) => handle(request, agent).await,
        Err(e) => {
            let error = RpcError::new(
                RpcError::INVALID_REQUEST,
                format!("not a valid request: {e}"),
            );
            Response::failure(id, error)
        }
    }
}

/// Answers one request.
pub async fn handle(request: Request, agent: &Agent) -> Response {
    let store = || store::lock(&agent.store);
    let result = match request.method.as_str() {
        method::VERSION => no_params(&request).and_then(|()| {
            to_result(&VersionInfo {
                agent: VERSION.to_owned(),
                protocol: PROTOCOL_VERSION,
            })
        }),
        method::STATUS => no_params(&request).and_then(|()| {
            let store = store();
            let findings = store.open_findings().map_err(store_failed)?;
            let checked_at = store.setting(LAST_CHECK_SETTING).map_err(store_failed)?;
            let checked = store
                .setting(CHECKED_CATEGORIES_SETTING)
                .map_err(store_failed)?
                .unwrap_or_default();
            // No score before the first check: an empty list would look like a perfect 100.
            let (score, deductions) = match checked_at {
                Some(_) => {
                    let (score, deductions) = health::score(&findings);
                    (Some(score), deductions)
                }
                None => (None, Vec::new()),
            };
            to_result(&StatusReport {
                generated_at: Timestamp::now(),
                checked_at,
                checked,
                score,
                deductions,
                findings,
            })
        }),
        method::METRICS => params::<MetricsParams>(&request).and_then(|p| {
            let points = store()
                .metrics(&p.name, p.subject.as_deref(), p.since, p.until)
                .map_err(store_failed)?;
            let max = p.max_points.unwrap_or(MAX_POINTS).clamp(1, MAX_POINTS);
            let span = p.until.as_millis() - p.since.as_millis();
            to_result(&thin(points, span, max))
        }),
        method::LATEST => no_params(&request).and_then(|()| {
            let since = Timestamp(Timestamp::now().as_millis() - FRESH_FOR_MS);
            to_result(&store().latest_metrics(since).map_err(store_failed)?)
        }),
        method::SNAPSHOT => params::<SnapshotParams>(&request)
            .and_then(|p| to_result(&store().latest_snapshot(p.kind).map_err(store_failed)?)),
        method::INVENTORY => match params::<InventoryParams>(&request) {
            Ok(p) => inventory::read(&p.kind, agent)
                .await
                .map_err(|why| RpcError::new(RpcError::INVALID_PARAMS, why)),
            Err(e) => Err(e),
        },
        method::PROCESS_DETAIL => match params::<ProcessDetailParams>(&request) {
            Ok(p) => inventory::process_detail(p.pid, agent)
                .await
                .map_err(|why| RpcError::new(RpcError::INVALID_PARAMS, why)),
            Err(e) => Err(e),
        },
        method::PACKAGE_DETAIL => match params::<PackageDetailParams>(&request) {
            Ok(p) => inventory::package_detail(&p.name, agent)
                .await
                .map_err(|why| RpcError::new(RpcError::INVALID_PARAMS, why)),
            Err(e) => Err(e),
        },
        method::FIX_PLAN => match params::<FixPlanParams>(&request) {
            Ok(p) => crate::fixes::plan(agent, &p.action, p.subject.as_deref())
                .await
                .map_err(|why| RpcError::new(RpcError::INVALID_PARAMS, why))
                .and_then(|plan| to_result(&plan)),
            Err(e) => Err(e),
        },
        method::FIX_RECORD => params::<FixRecordParams>(&request).and_then(|p| {
            let store = store();
            let id = store.add_fix_run(&p.run).map_err(store_failed)?;
            let mut run = p.run.clone();
            run.id = id;
            let (key, event) = crate::history::fix_event(&run, None);
            store.add_event(&key, &event).map_err(store_failed)?;
            drop(store);
            // Check again now, so the app can show whether the problem is gone.
            agent.recheck.notify_one();
            to_result(&id)
        }),
        method::FIX_RUNS => no_params(&request)
            .and_then(|()| to_result(&store().fix_runs(200).map_err(store_failed)?)),
        method::FIX_UNDONE => params::<FixUndoneParams>(&request).and_then(|p| {
            let store = store();
            let changed = store
                .set_fix_status(p.id, deecure_core::fix::RunStatus::Undone)
                .map_err(store_failed)?;
            if !changed {
                return Err(RpcError::new(
                    RpcError::INVALID_PARAMS,
                    format!("no fix run {}", p.id),
                ));
            }
            if let Some(run) = store.fix_run(p.id).map_err(store_failed)? {
                let (key, event) = crate::history::fix_event(&run, Some(Timestamp::now()));
                store.add_event(&key, &event).map_err(store_failed)?;
                // The applied event no longer offers an undo.
                let (key, event) = crate::history::fix_event(&run, None);
                store.put_event(&key, &event).map_err(store_failed)?;
            }
            drop(store);
            agent.recheck.notify_one();
            to_result(&true)
        }),
        method::EVENTS => params::<EventsParams>(&request).and_then(|p| {
            let until = p.until.unwrap_or_else(Timestamp::now);
            let limit = p.limit.unwrap_or(MAX_EVENTS).clamp(1, MAX_EVENTS);
            let events = store()
                .events(p.since, until, &p.kinds, limit + 1)
                .map_err(store_failed)?;
            to_result(&event_list(events, limit))
        }),
        method::EVENT => params::<EventParams>(&request)
            .and_then(|p| to_result(&store().event_by_id(p.id).map_err(store_failed)?)),
        method::UPDATE_STATUS => no_params(&request)
            .and_then(|()| to_result(&crate::updates::status(agent).map_err(store_failed)?)),
        method::SET_UPDATE_SETTINGS => params::<SetUpdateSettingsParams>(&request).and_then(|p| {
            store()
                .set_setting(crate::updates::SETTINGS, &p.settings)
                .map_err(store_failed)?;
            to_result(&crate::updates::status(agent).map_err(store_failed)?)
        }),
        method::CHECK_UPDATES => match no_params(&request) {
            Ok(()) => crate::updates::check_now(agent)
                .await
                .map_err(store_failed)
                .and_then(|s| to_result(&s)),
            Err(e) => Err(e),
        },
        method::WARNING_SETTINGS => no_params(&request).and_then(|()| {
            let settings: WarningSettings = store()
                .setting(crate::rules::WARNINGS_SETTING)
                .map_err(store_failed)?
                .unwrap_or_default();
            to_result(&warning_rules(agent, &settings))
        }),
        method::SET_WARNING_SETTINGS => {
            params::<SetWarningSettingsParams>(&request).and_then(|p| {
                // Only known warnings and limits are kept; values stay inside each range.
                let rules = load_rules(agent);
                let mut clean = WarningSettings::default();
                for rule in &rules {
                    let id = rule.id.as_str();
                    if p.settings.off.contains(id) {
                        clean.off.insert(id.to_owned());
                    }
                    for (name, limit) in &rule.limits {
                        if let Some(v) = p.settings.limits.get(id).and_then(|l| l.get(name))
                            && v.is_finite()
                        {
                            let v = v.clamp(limit.min, limit.max);
                            if v != limit.default {
                                clean
                                    .limits
                                    .entry(id.to_owned())
                                    .or_default()
                                    .insert(name.clone(), v);
                            }
                        }
                    }
                }
                store()
                    .set_setting(crate::rules::WARNINGS_SETTING, &clean)
                    .map_err(store_failed)?;
                agent.recheck.notify_one();
                to_result(&warning_rules(agent, &clean))
            })
        }
        method::NOTIFY_SETTINGS => no_params(&request)
            .and_then(|()| to_result(&crate::notify::settings(&store()).map_err(store_failed)?)),
        method::SET_NOTIFY_SETTINGS => params::<SetNotifySettingsParams>(&request).and_then(|p| {
            store()
                .set_setting(crate::notify::SETTING, &p.settings)
                .map_err(store_failed)?;
            to_result(&p.settings)
        }),
        method::NOTIFY_TEST => match no_params(&request) {
            Ok(()) => {
                let (title, body) = (
                    "DeeCure notifications work",
                    "This is how DeeCure tells you about serious problems.",
                );
                match crate::notify::show(agent, title, body, false).await {
                    Ok(()) => store()
                        .add_notification("test", Timestamp::now(), title, body)
                        .map_err(store_failed)
                        .and_then(|()| to_result(&true)),
                    Err(why) => Err(RpcError::new(RpcError::INTERNAL_ERROR, why)),
                }
            }
            Err(e) => Err(e),
        },
        method::NOTIFICATIONS => no_params(&request)
            .and_then(|()| to_result(&store().recent_notifications(30).map_err(store_failed)?)),
        method::REPORT => {
            // A missing or empty params object means the last 7 days.
            let p = match &request.params {
                None | Some(Value::Null) => Ok(deecure_core::rpc::ReportParams::default()),
                Some(_) => params::<deecure_core::rpc::ReportParams>(&request),
            };
            p.and_then(|p| {
                let now = Timestamp::now();
                let until = p.until.unwrap_or(now);
                let since = p
                    .since
                    .unwrap_or(Timestamp(until.as_millis() - 7 * 86_400_000));
                if since > until {
                    return Err(RpcError::new(
                        RpcError::INVALID_PARAMS,
                        "since is after until",
                    ));
                }
                let report = crate::report::build(&store(), &agent.root, since, until, now)
                    .map_err(store_failed)?;
                to_result(&report)
            })
        }
        method::WEEKLY_REPORTS => {
            no_params(&request).and_then(|()| to_result(&store().reports(8).map_err(store_failed)?))
        }
        method::DATA_SETTINGS => no_params(&request).and_then(|()| {
            let settings: deecure_core::notify::DataSettings = store()
                .setting(crate::store::DATA_SETTING)
                .map_err(store_failed)?
                .unwrap_or_default();
            to_result(&settings)
        }),
        method::SET_DATA_SETTINGS => {
            params::<deecure_core::rpc::SetDataSettingsParams>(&request).and_then(|p| {
                let store = store();
                store
                    .set_setting(crate::store::DATA_SETTING, &p.settings)
                    .map_err(store_failed)?;
                // A shorter time takes effect now, so the space comes back right away.
                store
                    .apply_retention(Timestamp::now())
                    .map_err(store_failed)?;
                to_result(&p.settings)
            })
        }
        method::DELETE_HISTORY => params::<deecure_core::rpc::DeleteHistoryParams>(&request)
            .and_then(|p| {
                if !p.confirm {
                    return Err(RpcError::new(
                        RpcError::INVALID_PARAMS,
                        "delete_history needs {\"confirm\": true}",
                    ));
                }
                let deleted = store()
                    .delete_history(Timestamp::now())
                    .map_err(store_failed)?;
                to_result(&deleted)
            }),
        other => Err(RpcError::method_not_found(other)),
    };
    match result {
        Ok(value) => Response::success(request.id, value),
        Err(error) => Response::failure(Some(request.id), error),
    }
}

/// Up to `limit` events, without config diffs (the `event` method has them), and only as
/// many as fit in one message with room to spare.
fn event_list(events: Vec<deecure_core::Event>, limit: usize) -> EventList {
    let budget = MAX_MESSAGE_BYTES * 3 / 4;
    let mut truncated = events.len() > limit;
    let mut used = 0;
    let mut out = Vec::new();
    for mut e in events.into_iter().take(limit) {
        if let Some(data) = e.data.as_mut().and_then(|d| d.as_object_mut()) {
            data.remove("diff");
        }
        used += serde_json::to_vec(&e).map_or(0, |v| v.len()) + 1;
        if used > budget {
            truncated = true;
            break;
        }
        out.push(e);
    }
    EventList {
        events: out,
        truncated,
    }
}

/// Accepts a missing, `null`, `{}`, or `[]` params field.
fn no_params(request: &Request) -> Result<(), RpcError> {
    let empty = match &request.params {
        None | Some(Value::Null) => true,
        Some(Value::Object(map)) => map.is_empty(),
        Some(Value::Array(list)) => list.is_empty(),
        Some(_) => false,
    };
    if empty {
        Ok(())
    } else {
        Err(RpcError::new(
            RpcError::INVALID_PARAMS,
            format!("'{}' takes no params", request.method),
        ))
    }
}

/// Reads the params a method needs.
fn params<T: DeserializeOwned>(request: &Request) -> Result<T, RpcError> {
    let value = request.params.clone().unwrap_or(Value::Null);
    serde_json::from_value(value).map_err(|e| {
        RpcError::new(
            RpcError::INVALID_PARAMS,
            format!("bad params for '{}': {e}", request.method),
        )
    })
}

fn store_failed(e: StoreError) -> RpcError {
    RpcError::new(RpcError::INTERNAL_ERROR, format!("database: {e}"))
}

fn to_result<T: Serialize>(value: &T) -> Result<Value, RpcError> {
    serde_json::to_value(value)
        .map_err(|e| RpcError::new(RpcError::INTERNAL_ERROR, format!("could not encode: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::SystemRoot;
    use crate::store::Store;
    use deecure_core::rpc::Outcome;
    use deecure_core::{Finding, FindingState, RuleId, Severity};
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    fn agent() -> Arc<Agent> {
        let store = Arc::new(Mutex::new(Store::open_in_memory().unwrap()));
        Agent::new(store, SystemRoot::real())
    }

    async fn answer(line: &str) -> Value {
        serde_json::to_value(handle_line(line.as_bytes(), &agent()).await).unwrap()
    }

    #[tokio::test]
    async fn version_reports_agent_and_protocol() {
        assert_eq!(
            answer(r#"{"jsonrpc":"2.0","id":1,"method":"version"}"#).await,
            json!({"jsonrpc": "2.0", "id": 1, "result": {"agent": VERSION, "protocol": PROTOCOL_VERSION}})
        );
    }

    #[tokio::test]
    async fn status_returns_open_findings_from_the_store() {
        let agent = agent();
        let finding = Finding {
            rule: RuleId::new("storage.root_almost_full").unwrap(),
            subject: Some("/".into()),
            severity: Severity::Critical,
            state: FindingState::Open,
            title: "Main disk is almost full".into(),
            explain: "Only 13.8 GB left.".into(),
            first_seen: Timestamp(1),
            last_seen: Timestamp(2),
            fixes: vec![],
        };
        store::lock(&agent.store).upsert_finding(&finding).unwrap();

        let response = handle_line(
            br#"{"jsonrpc":"2.0","id":"s","method":"status","params":{}}"#,
            &agent,
        )
        .await;
        assert_eq!(response.id, Some(RequestId::from("s")));
        let Outcome::Success(result) = response.outcome else {
            panic!("expected a result");
        };
        let report: StatusReport = serde_json::from_value(result).unwrap();
        assert_eq!(report.findings, [finding]);
        assert_eq!(report.score, None, "no score before the first check");
    }

    #[tokio::test]
    async fn status_scores_health_after_a_check() {
        let agent = agent();
        store::lock(&agent.store)
            .set_setting(LAST_CHECK_SETTING, &Timestamp(5))
            .unwrap();
        let reply = answer_with(&agent, r#"{"jsonrpc":"2.0","id":1,"method":"status"}"#).await;
        assert_eq!(reply["result"]["score"], 100);
    }

    #[tokio::test]
    async fn inventory_reads_live_system_facts() {
        let reply =
            answer(r#"{"jsonrpc":"2.0","id":1,"method":"inventory","params":{"kind":"system"}}"#)
                .await;
        assert_eq!(reply["result"]["arch"], std::env::consts::ARCH);
        let reply =
            answer(r#"{"jsonrpc":"2.0","id":2,"method":"inventory","params":{"kind":"toasters"}}"#)
                .await;
        assert_eq!(reply["error"]["code"], RpcError::INVALID_PARAMS);
    }

    async fn answer_with(agent: &Agent, line: &str) -> Value {
        serde_json::to_value(handle_line(line.as_bytes(), agent).await).unwrap()
    }

    #[tokio::test]
    async fn unknown_method_is_reported() {
        let reply = answer(r#"{"jsonrpc":"2.0","id":2,"method":"reboot"}"#).await;
        assert_eq!(reply["id"], 2);
        assert_eq!(reply["error"]["code"], RpcError::METHOD_NOT_FOUND);
    }

    #[tokio::test]
    async fn bad_json_gets_parse_error_with_null_id() {
        let reply = answer("{not json").await;
        assert_eq!(reply["id"], Value::Null);
        assert_eq!(reply["error"]["code"], RpcError::PARSE_ERROR);
    }

    #[tokio::test]
    async fn invalid_request_keeps_its_id() {
        let reply = answer(r#"{"jsonrpc":"2.0","id":9}"#).await;
        assert_eq!(reply["id"], 9);
        assert_eq!(reply["error"]["code"], RpcError::INVALID_REQUEST);
    }

    #[tokio::test]
    async fn metrics_and_snapshot_read_the_store() {
        let agent = agent();
        store::lock(&agent.store)
            .add_metric(&deecure_core::Metric {
                name: "cpu.usage".into(),
                subject: None,
                value: 12.0,
                unit: deecure_core::Unit::Percent,
                at: Timestamp(60_000),
            })
            .unwrap();
        let reply = serde_json::to_value(handle_line(
            br#"{"jsonrpc":"2.0","id":4,"method":"metrics","params":{"name":"cpu.usage","since":0,"until":120000}}"#,
            &agent,
        ).await)
        .unwrap();
        assert_eq!(reply["result"][0]["value"], 12.0);

        let reply = answer_with(&agent, r#"{"jsonrpc":"2.0","id":9,"method":"latest"}"#).await;
        assert_eq!(
            reply["result"],
            json!([]),
            "the point is older than 10 minutes"
        );

        let reply = serde_json::to_value(
            handle_line(
                br#"{"jsonrpc":"2.0","id":5,"method":"snapshot","params":{"kind":"services"}}"#,
                &agent,
            )
            .await,
        )
        .unwrap();
        assert_eq!(reply["result"], Value::Null, "nothing stored yet");
    }

    #[tokio::test]
    async fn missing_or_unknown_params_are_rejected() {
        for line in [
            r#"{"jsonrpc":"2.0","id":6,"method":"metrics"}"#,
            r#"{"jsonrpc":"2.0","id":7,"method":"snapshot","params":{"kind":"toasters"}}"#,
            r#"{"jsonrpc":"2.0","id":8,"method":"snapshot","params":{"kind":"services","extra":1}}"#,
        ] {
            assert_eq!(
                answer(line).await["error"]["code"],
                RpcError::INVALID_PARAMS,
                "{line}"
            );
        }
    }

    #[tokio::test]
    async fn unexpected_params_are_rejected() {
        let reply = answer(r#"{"jsonrpc":"2.0","id":3,"method":"version","params":{"x":1}}"#).await;
        assert_eq!(reply["error"]["code"], RpcError::INVALID_PARAMS);
    }

    #[tokio::test]
    async fn events_come_without_diffs_and_one_event_has_them() {
        let agent = agent();
        let event = |at: i64, diff: &str| deecure_core::Event {
            id: None,
            kind: deecure_core::EventKind::ConfigChanged,
            at: Timestamp(at),
            title: "Changed /etc/hosts".into(),
            detail: None,
            subject: Some("/etc/hosts".into()),
            source: "root".into(),
            data: Some(json!({ "change": "changed", "diff": diff })),
        };
        {
            let s = store::lock(&agent.store);
            s.add_event("a", &event(10, "+ x")).unwrap();
            s.add_event("b", &event(20, "+ y")).unwrap();
        }
        let reply = answer_with(
            &agent,
            r#"{"jsonrpc":"2.0","id":1,"method":"events","params":{"since":0,"limit":1}}"#,
        )
        .await;
        assert_eq!(reply["result"]["truncated"], true);
        let list = reply["result"]["events"].as_array().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0]["at"], 20);
        assert_eq!(list[0]["data"], json!({ "change": "changed" }));
        let id = list[0]["id"].as_i64().unwrap();
        let reply = answer_with(
            &agent,
            &format!(r#"{{"jsonrpc":"2.0","id":2,"method":"event","params":{{"id":{id}}}}}"#),
        )
        .await;
        assert_eq!(reply["result"]["data"]["diff"], "+ y");
        let reply = answer_with(
            &agent,
            r#"{"jsonrpc":"2.0","id":3,"method":"events","params":{"since":0,"kinds":["boot"]}}"#,
        )
        .await;
        assert_eq!(reply["result"]["events"], json!([]));
        assert_eq!(reply["result"]["truncated"], false);
    }

    #[tokio::test]
    async fn warning_settings_keep_only_known_warnings_and_safe_values() {
        let agent = agent();
        let rules = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rules");
        let _ = agent.facts.set(crate::context::Facts {
            database: "test.db".into(),
            rules_dir: rules,
            rules: 0,
            started: std::time::Instant::now(),
            kb: crate::kb::Kb::default(),
        });
        let reply = answer_with(
            &agent,
            r#"{"jsonrpc":"2.0","id":1,"method":"set_warning_settings","params":{"settings":{"off":["network.weak_wifi","made.up"],"limits":{"hardware.cpu_hot":{"minutes":500,"made_up":3},"made.up":{"x":1}}}}}"#,
        )
        .await;
        let list = reply["result"].as_array().unwrap();
        let cpu = list.iter().find(|r| r["id"] == "hardware.cpu_hot").unwrap();
        assert_eq!(cpu["name"], "CPU too hot");
        assert_eq!(cpu["off"], false);
        assert_eq!(cpu["limits"][0]["value"], 60.0, "kept at the most allowed");
        assert_eq!(cpu["limits"][0]["default"], 5.0);
        let wifi = list
            .iter()
            .find(|r| r["id"] == "network.weak_wifi")
            .unwrap();
        assert_eq!(wifi["off"], true);
        let stored: WarningSettings = crate::store::lock(&agent.store)
            .setting(crate::rules::WARNINGS_SETTING)
            .unwrap()
            .unwrap();
        assert_eq!(stored.off.iter().collect::<Vec<_>>(), ["network.weak_wifi"]);
        assert_eq!(
            stored.limits,
            std::collections::BTreeMap::from([(
                "hardware.cpu_hot".to_owned(),
                std::collections::BTreeMap::from([("minutes".to_owned(), 60.0)])
            )])
        );
        let reply = answer_with(
            &agent,
            r#"{"jsonrpc":"2.0","id":2,"method":"warning_settings"}"#,
        )
        .await;
        assert_eq!(reply["result"].as_array().unwrap().len(), list.len());
    }

    #[tokio::test]
    async fn update_settings_are_kept_and_the_status_says_what_runs_now() {
        let agent = agent();
        let reply = answer_with(
            &agent,
            r#"{"jsonrpc":"2.0","id":1,"method":"update_status"}"#,
        )
        .await;
        let s = &reply["result"];
        assert_eq!(s["current"], VERSION);
        assert_eq!(
            s["settings"],
            json!({"check": "daily", "channel": "prerelease", "auto": false}),
            "the defaults"
        );
        assert_eq!(s["available"], false, "nothing was checked yet");
        let reply = answer_with(
            &agent,
            r#"{"jsonrpc":"2.0","id":2,"method":"set_update_settings","params":{"settings":{"check":"weekly","channel":"stable","auto":false}}}"#,
        )
        .await;
        assert_eq!(reply["result"]["settings"]["check"], "weekly");
        let reply = answer_with(
            &agent,
            r#"{"jsonrpc":"2.0","id":3,"method":"set_update_settings","params":{"settings":{"check":"hourly"}}}"#,
        )
        .await;
        assert_eq!(reply["error"]["code"], -32602, "unknown values are refused");
    }

    #[test]
    fn a_long_event_list_is_cut_to_fit_one_message() {
        let big = |i: i64| deecure_core::Event {
            id: Some(i),
            kind: deecure_core::EventKind::PackageUpgraded,
            at: Timestamp(i),
            title: "x".repeat(2000),
            detail: None,
            subject: None,
            source: "apt".into(),
            data: None,
        };
        let list = event_list((0..1000).map(big).collect(), 2000);
        assert!(list.truncated);
        assert!(serde_json::to_vec(&list).unwrap().len() < MAX_MESSAGE_BYTES);
    }

    #[tokio::test]
    async fn notification_settings_round_trip_and_are_checked() {
        let agent = agent();
        let reply = answer_with(
            &agent,
            r#"{"jsonrpc":"2.0","id":1,"method":"notify_settings"}"#,
        )
        .await;
        assert_eq!(reply["result"]["critical"], true);
        assert_eq!(reply["result"]["quiet_hours"], "22-07");
        let reply = answer_with(
            &agent,
            r#"{"jsonrpc":"2.0","id":2,"method":"set_notify_settings","params":{"settings":{"warnings":true,"quiet_hours":"off"}}}"#,
        )
        .await;
        assert_eq!(reply["result"]["warnings"], true);
        let reply = answer_with(
            &agent,
            r#"{"jsonrpc":"2.0","id":3,"method":"notify_settings"}"#,
        )
        .await;
        assert_eq!(reply["result"]["quiet_hours"], "off");
        let reply = answer_with(
            &agent,
            r#"{"jsonrpc":"2.0","id":4,"method":"set_notify_settings","params":{"settings":{"quiet_hours":"all day"}}}"#,
        )
        .await;
        assert_eq!(reply["error"]["code"], RpcError::INVALID_PARAMS);
        let reply = answer_with(
            &agent,
            r#"{"jsonrpc":"2.0","id":5,"method":"notifications"}"#,
        )
        .await;
        assert_eq!(reply["result"], json!([]));
    }

    #[tokio::test]
    async fn delete_history_needs_confirmation() {
        let agent = agent();
        let reply = answer_with(
            &agent,
            r#"{"jsonrpc":"2.0","id":1,"method":"delete_history","params":{"confirm":false}}"#,
        )
        .await;
        assert_eq!(reply["error"]["code"], RpcError::INVALID_PARAMS);
        let reply = answer_with(
            &agent,
            r#"{"jsonrpc":"2.0","id":2,"method":"delete_history"}"#,
        )
        .await;
        assert_eq!(reply["error"]["code"], RpcError::INVALID_PARAMS);
        let reply = answer_with(
            &agent,
            r#"{"jsonrpc":"2.0","id":3,"method":"delete_history","params":{"confirm":true}}"#,
        )
        .await;
        assert!(reply["result"]["rows"].is_number());
        let reply = answer_with(
            &agent,
            r#"{"jsonrpc":"2.0","id":4,"method":"set_data_settings","params":{"settings":{"keep_history":"1y"}}}"#,
        )
        .await;
        assert_eq!(reply["result"]["keep_history"], "1y");
        let reply = answer_with(
            &agent,
            r#"{"jsonrpc":"2.0","id":5,"method":"data_settings"}"#,
        )
        .await;
        assert_eq!(reply["result"]["keep_history"], "1y");
    }
}
