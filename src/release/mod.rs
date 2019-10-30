use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Release {
    pub name: String,
    // TODO: add chart
    pub info: Info,
    pub config: HashMap<String, Value>,
    pub manifest: String,
    // TODO: hooks
    pub version: u64,
    pub namespace: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Info {
    // FirstDeployed is when the release was first deployed.
    pub first_deployed: Option<DateTime<Utc>>,
    // LastDeployed is when the release was last deployed.
    pub last_deployed: Option<DateTime<Utc>>,
    // Deleted tracks when this object was deleted.
    pub deleted: Option<DateTime<Utc>>,
    // Description is human-friendly "log entry" about this release.
    pub description: String,
    // Status is the current state of the release
    pub status: Status,
    // Contains the rendered templates/NOTES.txt if available
    pub notes: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Status {
    Unknown,
    Deployed,
    Uninstalled,
    Superseded,
    Failed,
    Uninstalling,
    PendingInstall,
    PendingUpgrade,
    PendingRollback,
}

impl Default for Status {
    fn default() -> Self {
        Status::Unknown
    }
}
