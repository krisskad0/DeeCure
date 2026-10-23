// SPDX-License-Identifier: Apache-2.0

//! Turns one request line into one response. No I/O here, so it is easy to test.

use deecure_core::rpc::{
    EventList, EventParams, EventsParams, FixPlanParams, FixRecordParams, FixUndoneParams,
    InventoryParams, MAX_EVENTS, MAX_MESSAGE_BYTES, MAX_POINTS, MetricsParams, PROTOCOL_VERSION,
    Request, RequestId, Response, RpcError, SetNotifySettingsParams, SnapshotParams, StatusReport,
    VersionInfo, method,
};
use deecure_core::{Timestamp, VERSION, health, thin};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::context::Agent;
use crate::inventory;
use crate::rules::facts::FRESH_FOR_MS;
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
}
