// SPDX-License-Identifier: Apache-2.0

//! Turns one request line into one response. No I/O here, so it is easy to test.

use deecure_core::rpc::{
    MetricsParams, PROTOCOL_VERSION, Request, RequestId, Response, RpcError, SnapshotParams,
    StatusReport, VersionInfo, method,
};
use deecure_core::{Timestamp, VERSION};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::rules::{CHECKED_CATEGORIES_SETTING, LAST_CHECK_SETTING};
use crate::store::{Store, StoreError};

/// Reads one JSON-RPC message and answers it.
pub fn handle_line(line: &[u8], store: &Store) -> Response {
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
        Ok(request) => handle(request, store),
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
pub fn handle(request: Request, store: &Store) -> Response {
    let result = match request.method.as_str() {
        method::VERSION => no_params(&request).and_then(|()| {
            to_result(&VersionInfo {
                agent: VERSION.to_owned(),
                protocol: PROTOCOL_VERSION,
            })
        }),
        method::STATUS => no_params(&request).and_then(|()| {
            let findings = store.open_findings().map_err(store_failed)?;
            let checked_at = store.setting(LAST_CHECK_SETTING).map_err(store_failed)?;
            let checked = store
                .setting(CHECKED_CATEGORIES_SETTING)
                .map_err(store_failed)?
                .unwrap_or_default();
            to_result(&StatusReport {
                generated_at: Timestamp::now(),
                checked_at,
                checked,
                findings,
            })
        }),
        method::METRICS => params::<MetricsParams>(&request).and_then(|p| {
            let points = store
                .metrics(&p.name, p.subject.as_deref(), p.since, p.until)
                .map_err(store_failed)?;
            to_result(&points)
        }),
        method::SNAPSHOT => params::<SnapshotParams>(&request)
            .and_then(|p| to_result(&store.latest_snapshot(p.kind).map_err(store_failed)?)),
        other => Err(RpcError::method_not_found(other)),
    };
    match result {
        Ok(value) => Response::success(request.id, value),
        Err(error) => Response::failure(Some(request.id), error),
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
    use deecure_core::rpc::Outcome;
    use deecure_core::{Finding, FindingState, RuleId, Severity};
    use serde_json::json;

    fn answer(line: &str) -> Value {
        let store = Store::open_in_memory().unwrap();
        serde_json::to_value(handle_line(line.as_bytes(), &store)).unwrap()
    }

    #[test]
    fn version_reports_agent_and_protocol() {
        assert_eq!(
            answer(r#"{"jsonrpc":"2.0","id":1,"method":"version"}"#),
            json!({"jsonrpc": "2.0", "id": 1, "result": {"agent": VERSION, "protocol": PROTOCOL_VERSION}})
        );
    }

    #[test]
    fn status_returns_open_findings_from_the_store() {
        let store = Store::open_in_memory().unwrap();
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
        store.upsert_finding(&finding).unwrap();

        let response = handle_line(
            br#"{"jsonrpc":"2.0","id":"s","method":"status","params":{}}"#,
            &store,
        );
        assert_eq!(response.id, Some(RequestId::from("s")));
        let Outcome::Success(result) = response.outcome else {
            panic!("expected a result");
        };
        let report: StatusReport = serde_json::from_value(result).unwrap();
        assert_eq!(report.findings, [finding]);
    }

    #[test]
    fn unknown_method_is_reported() {
        let reply = answer(r#"{"jsonrpc":"2.0","id":2,"method":"reboot"}"#);
        assert_eq!(reply["id"], 2);
        assert_eq!(reply["error"]["code"], RpcError::METHOD_NOT_FOUND);
    }

    #[test]
    fn bad_json_gets_parse_error_with_null_id() {
        let reply = answer("{not json");
        assert_eq!(reply["id"], Value::Null);
        assert_eq!(reply["error"]["code"], RpcError::PARSE_ERROR);
    }

    #[test]
    fn invalid_request_keeps_its_id() {
        let reply = answer(r#"{"jsonrpc":"2.0","id":9}"#);
        assert_eq!(reply["id"], 9);
        assert_eq!(reply["error"]["code"], RpcError::INVALID_REQUEST);
    }

    #[test]
    fn metrics_and_snapshot_read_the_store() {
        let store = Store::open_in_memory().unwrap();
        store
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
            &store,
        ))
        .unwrap();
        assert_eq!(reply["result"][0]["value"], 12.0);

        let reply = serde_json::to_value(handle_line(
            br#"{"jsonrpc":"2.0","id":5,"method":"snapshot","params":{"kind":"services"}}"#,
            &store,
        ))
        .unwrap();
        assert_eq!(reply["result"], Value::Null, "nothing stored yet");
    }

    #[test]
    fn missing_or_unknown_params_are_rejected() {
        for line in [
            r#"{"jsonrpc":"2.0","id":6,"method":"metrics"}"#,
            r#"{"jsonrpc":"2.0","id":7,"method":"snapshot","params":{"kind":"toasters"}}"#,
            r#"{"jsonrpc":"2.0","id":8,"method":"snapshot","params":{"kind":"services","extra":1}}"#,
        ] {
            assert_eq!(
                answer(line)["error"]["code"],
                RpcError::INVALID_PARAMS,
                "{line}"
            );
        }
    }

    #[test]
    fn unexpected_params_are_rejected() {
        let reply = answer(r#"{"jsonrpc":"2.0","id":3,"method":"version","params":{"x":1}}"#);
        assert_eq!(reply["error"]["code"], RpcError::INVALID_PARAMS);
    }
}
