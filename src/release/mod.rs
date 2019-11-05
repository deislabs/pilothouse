pub mod sort;

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
    pub version: usize,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

// We need to implement Display to match what the current Go statuses are and
// have a `to_string` method for label queries
impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Status::Unknown => write!(f, "unknown"),
            Status::Deployed => write!(f, "deployed"),
            Status::Uninstalled => write!(f, "uninstalled"),
            Status::Superseded => write!(f, "superseded"),
            Status::Failed => write!(f, "failed"),
            Status::Uninstalling => write!(f, "uninstalling"),
            Status::PendingInstall => write!(f, "pending-install"),
            Status::PendingUpgrade => write!(f, "pending-upgrade"),
            Status::PendingRollback => write!(f, "pending-rollback"),
        }
    }
}
