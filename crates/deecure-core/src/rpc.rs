// SPDX-License-Identifier: Apache-2.0

//! JSON-RPC 2.0 messages between the agent and its clients (the CLI and the desktop app).
//!
//! The wire format is described in `docs/02-architecture/ipc-protocol.md`.

use std::fmt;
use std::path::PathBuf;

use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

use crate::health::Deduction;
use crate::{Category, Finding, Timestamp};

/// The protocol version the agent speaks. Raise it when a method changes in a way
/// that breaks old clients.
pub const PROTOCOL_VERSION: u32 = 1;

/// The longest message, in bytes, that either side accepts on one line.
pub const MAX_MESSAGE_BYTES: usize = 1024 * 1024;

/// Where the agent listens by default: `$XDG_RUNTIME_DIR/deecure/agent.sock`.
///
/// `None` when `XDG_RUNTIME_DIR` is not set. There is no fallback to `/tmp`,
/// because other users can reach `/tmp`.
pub fn default_socket_path() -> Option<PathBuf> {
    std::env::var_os("XDG_RUNTIME_DIR")
        .filter(|dir| !dir.is_empty())
        .map(|dir| PathBuf::from(dir).join("deecure").join("agent.sock"))
}

/// Method names the agent answers.
pub mod method {
    /// Returns [`VersionInfo`](super::VersionInfo). Takes no params.
    pub const VERSION: &str = "version";
    /// Returns [`StatusReport`](super::StatusReport). Takes no params.
    pub const STATUS: &str = "status";
    /// Returns a list of [`Metric`](crate::Metric). Takes [`MetricsParams`](super::MetricsParams).
    pub const METRICS: &str = "metrics";
    /// Returns the newest point of every metric from the last 10 minutes, as a list of
    /// [`Metric`](crate::Metric). Takes no params.
    pub const LATEST: &str = "latest";
    /// Returns a [`Snapshot`](crate::Snapshot) or `null`. Takes [`SnapshotParams`](super::SnapshotParams).
    pub const SNAPSHOT: &str = "snapshot";
    /// Returns live data read now, such as `system` facts or `processes`. Takes [`InventoryParams`](super::InventoryParams).
    pub const INVENTORY: &str = "inventory";
    /// Everything about one process (`ProcessDetailParams`), for the Expert process drawer.
    pub const PROCESS_DETAIL: &str = "process_detail";
    /// Returns a fix [`Plan`](crate::fix::Plan). Takes [`FixPlanParams`](super::FixPlanParams). Only reads.
    pub const FIX_PLAN: &str = "fix_plan";
    /// Saves a finished [`FixRun`](crate::fix::FixRun), checks the rules again, and returns
    /// its ID. Takes [`FixRecordParams`](super::FixRecordParams).
    pub const FIX_RECORD: &str = "fix_record";
    /// Returns past [`FixRun`](crate::fix::FixRun)s, newest first. Takes no params.
    pub const FIX_RUNS: &str = "fix_runs";
    /// Marks a run as undone. Takes [`FixUndoneParams`](super::FixUndoneParams).
    pub const FIX_UNDONE: &str = "fix_undone";
    /// Returns an [`EventList`](super::EventList): what changed, newest first, without
    /// config diffs. Takes [`EventsParams`](super::EventsParams).
    pub const EVENTS: &str = "events";
    /// Returns one [`Event`](crate::Event) with all its details (such as a config diff),
    /// or `null`. Takes [`EventParams`](super::EventParams).
    pub const EVENT: &str = "event";
    /// Returns the [`NotifySettings`](crate::notify::NotifySettings). Takes no params.
    pub const NOTIFY_SETTINGS: &str = "notify_settings";
    /// Saves new [`NotifySettings`](crate::notify::NotifySettings) and returns them. Takes
    /// [`SetNotifySettingsParams`](super::SetNotifySettingsParams).
    pub const SET_NOTIFY_SETTINGS: &str = "set_notify_settings";
    /// Shows a test notification now. Returns `true`, or an error saying why it could not.
    /// Takes no params.
    pub const NOTIFY_TEST: &str = "notify_test";
    /// Returns the last 30 [`SentNotification`](crate::notify::SentNotification)s, newest
    /// first. Takes no params.
    pub const NOTIFICATIONS: &str = "notifications";
    /// Returns a [`Report`](crate::report::Report) made now. Takes [`ReportParams`](super::ReportParams).
    pub const REPORT: &str = "report";
    /// Returns the last 8 weekly [`Report`](crate::report::Report)s, newest first. Takes no params.
    pub const WEEKLY_REPORTS: &str = "weekly_reports";
    /// Returns the [`DataSettings`](crate::notify::DataSettings). Takes no params.
    pub const DATA_SETTINGS: &str = "data_settings";
    /// Saves new [`DataSettings`](crate::notify::DataSettings), cleans up at once, and
    /// returns them. Takes [`SetDataSettingsParams`](super::SetDataSettingsParams).
    pub const SET_DATA_SETTINGS: &str = "set_data_settings";
    /// Deletes all history (metrics, events, reports, notifications, old snapshots, closed
    /// findings). Fix runs (restore points), settings, and open findings stay. Returns
    /// [`Deleted`](crate::notify::Deleted). Takes `{"confirm": true}`.
    pub const DELETE_HISTORY: &str = "delete_history";
}

/// Params of the `set_data_settings` method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetDataSettingsParams {
    /// The new settings.
    pub settings: crate::notify::DataSettings,
}

/// Params of the `delete_history` method: `confirm` must be `true`, so a stray call
/// cannot wipe anything.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteHistoryParams {
    /// Must be `true`.
    pub confirm: bool,
}

/// Params of the `report` method. Both are optional: the default is the last 7 days.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ReportParams {
    /// Start of the period; 7 days before `until` when missing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<Timestamp>,
    /// End of the period; now when missing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<Timestamp>,
}

/// Params of the `set_notify_settings` method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetNotifySettingsParams {
    /// The new settings. Missing fields take their defaults.
    pub settings: crate::notify::NotifySettings,
}

/// Params of the `events` method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventsParams {
    /// Start of the time range (included).
    pub since: Timestamp,
    /// End of the time range (included); now when missing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<Timestamp>,
    /// Only these kinds; every kind when missing or empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kinds: Vec<crate::EventKind>,
    /// At most this many, newest first ([`MAX_EVENTS`] when missing or larger).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// The most events one `events` answer holds.
pub const MAX_EVENTS: usize = 2000;

/// What `events` returns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventList {
    /// Newest first.
    pub events: Vec<crate::Event>,
    /// True when more events matched than were sent (the limit, or the message size).
    pub truncated: bool,
}

/// Params of the `event` method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventParams {
    /// The event's number.
    pub id: i64,
}

/// Params of the `metrics` method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricsParams {
    /// The metric, such as `cpu.usage`.
    pub name: String,
    /// Which one, for metrics that have many, such as a mount point.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// Start of the time range (included).
    pub since: Timestamp,
    /// End of the time range (included).
    pub until: Timestamp,
    /// Average the points down to about this many. The agent never sends more than
    /// [`MAX_POINTS`], so long ranges fit in one message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_points: Option<usize>,
}

/// The most points one `metrics` answer holds.
pub const MAX_POINTS: usize = 2000;

/// Params of the `inventory` method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryParams {
    /// What to read, such as `system` or `processes`.
    pub kind: String,
}

/// Params of the `process_detail` method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessDetailParams {
    /// The process ID.
    pub pid: u32,
}

/// Params of the `fix_plan` method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixPlanParams {
    /// The fix, such as `storage.free_space`.
    pub action: String,
    /// The target, for fixes that need one (a service, an app, `pid:name`, a power mode).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
}

/// Params of the `fix_record` method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixRecordParams {
    /// What happened.
    pub run: crate::fix::FixRun,
}

/// Params of the `fix_undone` method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixUndoneParams {
    /// The run's ID.
    pub id: i64,
}

/// Params of the `snapshot` method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotParams {
    /// Which inventory to return, the newest one stored.
    pub kind: crate::SnapshotKind,
}

/// The `"jsonrpc": "2.0"` marker. Any other value is rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct JsonRpc2;

impl Serialize for JsonRpc2 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str("2.0")
    }
}

impl<'de> Deserialize<'de> for JsonRpc2 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let version = String::deserialize(deserializer)?;
        if version == "2.0" {
            Ok(JsonRpc2)
        } else {
            Err(de::Error::custom(format!(
                "expected jsonrpc \"2.0\", got \"{version}\""
            )))
        }
    }
}

/// The ID a client puts on a request. The response carries the same ID back.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    /// A number ID.
    Number(i64),
    /// A text ID.
    Text(String),
}

impl From<i64> for RequestId {
    fn from(id: i64) -> Self {
        RequestId::Number(id)
    }
}

impl From<&str> for RequestId {
    fn from(id: &str) -> Self {
        RequestId::Text(id.to_owned())
    }
}

/// A call from a client to the agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    /// Always `"2.0"`.
    pub jsonrpc: JsonRpc2,
    /// Echoed back in the response.
    pub id: RequestId,
    /// The method to call, from [`method`].
    pub method: String,
    /// The method's input, if it takes any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl Request {
    /// Builds a request.
    pub fn new(id: impl Into<RequestId>, method: &str, params: Option<Value>) -> Self {
        Request {
            jsonrpc: JsonRpc2,
            id: id.into(),
            method: method.to_owned(),
            params,
        }
    }
}

/// The agent's answer to one request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    /// Always `"2.0"`.
    pub jsonrpc: JsonRpc2,
    /// The request's ID, or `null` when the request could not be read at all.
    pub id: Option<RequestId>,
    /// Either a result or an error, never both.
    #[serde(flatten)]
    pub outcome: Outcome,
}

/// What a request produced.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Outcome {
    /// The call worked.
    #[serde(rename = "result")]
    Success(Value),
    /// The call failed.
    #[serde(rename = "error")]
    Failure(RpcError),
}

impl Response {
    /// A successful answer.
    pub fn success(id: RequestId, result: Value) -> Self {
        Response {
            jsonrpc: JsonRpc2,
            id: Some(id),
            outcome: Outcome::Success(result),
        }
    }

    /// A failed answer.
    pub fn failure(id: Option<RequestId>, error: RpcError) -> Self {
        Response {
            jsonrpc: JsonRpc2,
            id,
            outcome: Outcome::Failure(error),
        }
    }
}

/// A JSON-RPC error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcError {
    /// A number for programs. See the constants on this type.
    pub code: i32,
    /// A short sentence for people.
    pub message: String,
    /// Extra detail, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl RpcError {
    /// The message is not valid JSON.
    pub const PARSE_ERROR: i32 = -32700;
    /// The JSON is not a valid request.
    pub const INVALID_REQUEST: i32 = -32600;
    /// The agent has no such method.
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// The params are wrong for this method.
    pub const INVALID_PARAMS: i32 = -32602;
    /// Something went wrong inside the agent.
    pub const INTERNAL_ERROR: i32 = -32603;

    /// Builds an error with no extra data.
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        RpcError {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// The agent has no method with this name.
    pub fn method_not_found(method: &str) -> Self {
        Self::new(Self::METHOD_NOT_FOUND, format!("unknown method '{method}'"))
    }
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (code {})", self.message, self.code)
    }
}

impl std::error::Error for RpcError {}

/// Result of the `version` method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionInfo {
    /// The agent's version, such as `0.1.0`.
    pub agent: String,
    /// The protocol version, see [`PROTOCOL_VERSION`].
    pub protocol: u32,
}

/// Result of the `status` method: what `deecure status` and the Overview page show.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusReport {
    /// When the agent built this report.
    pub generated_at: Timestamp,
    /// When the rules last ran. `None` means they have not run yet (the agent just
    /// started), so an empty `findings` list does not yet mean "all is well".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked_at: Option<Timestamp>,
    /// The areas that have rules, so the UI says "Healthy" only for areas that are
    /// really checked, and "Unknown" for the rest.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checked: Vec<Category>,
    /// The health score out of 100, once the rules have run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<u8>,
    /// What lowered the score, biggest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deductions: Vec<Deduction>,
    /// Open findings, most serious first.
    pub findings: Vec<Finding>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn request_json_shape() {
        let request = Request::new(1, method::STATUS, None);
        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            json!({"jsonrpc": "2.0", "id": 1, "method": "status"})
        );
        let parsed: Request =
            serde_json::from_str(r#"{"jsonrpc":"2.0","id":"a","method":"version"}"#).unwrap();
        assert_eq!(parsed.id, RequestId::from("a"));
    }

    #[test]
    fn wrong_jsonrpc_version_is_rejected() {
        let result = serde_json::from_str::<Request>(r#"{"jsonrpc":"1.0","id":1,"method":"x"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn success_response_round_trips() {
        let info = VersionInfo {
            agent: "0.1.0".into(),
            protocol: PROTOCOL_VERSION,
        };
        let response = Response::success(7.into(), serde_json::to_value(&info).unwrap());
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(
            json,
            json!({"jsonrpc": "2.0", "id": 7, "result": {"agent": "0.1.0", "protocol": 1}})
        );
        assert_eq!(serde_json::from_value::<Response>(json).unwrap(), response);
    }

    #[test]
    fn error_response_round_trips() {
        let response = Response::failure(None, RpcError::new(RpcError::PARSE_ERROR, "bad JSON"));
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(
            json,
            json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": "bad JSON"}})
        );
        assert_eq!(serde_json::from_value::<Response>(json).unwrap(), response);
    }

    #[test]
    fn method_not_found_names_the_method() {
        let error = RpcError::method_not_found("reboot");
        assert_eq!(error.code, RpcError::METHOD_NOT_FOUND);
        assert_eq!(error.to_string(), "unknown method 'reboot' (code -32601)");
    }

    #[test]
    fn status_report_json_shape() {
        let mut report = StatusReport {
            generated_at: Timestamp(3),
            checked_at: None,
            checked: vec![],
            score: None,
            deductions: vec![],
            findings: vec![],
        };
        assert_eq!(
            serde_json::to_value(&report).unwrap(),
            json!({"generated_at": 3, "findings": []})
        );
        report.checked_at = Some(Timestamp(2));
        report.checked = vec![Category::Storage, Category::Power];
        assert_eq!(
            serde_json::to_value(&report).unwrap(),
            json!({"generated_at": 3, "checked_at": 2, "checked": ["storage", "power"], "findings": []})
        );
    }
}
