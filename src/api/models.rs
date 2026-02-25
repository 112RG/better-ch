//! API models for CloudHub.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Application model from CloudHub API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Application {
    /// Application name.
    pub name: String,

    /// Application domain.
    pub domain: String,

    /// Full domain (domain.cloudhub.io).
    #[serde(rename = "fullDomain")]
    pub full_domain: Option<String>,

    /// Application status.
    pub status: String,

    /// Worker configuration.
    pub workers: Workers,

    /// Runtime version.
    #[serde(rename = "runtimeVersion")]
    pub runtime_version: Option<String>,

    /// Mule version.
    #[serde(rename = "muleVersion")]
    pub mule_version: Option<String>,

    /// Region.
    pub region: Option<String>,

    /// Last update timestamp.
    #[serde(rename = "lastUpdateDate")]
    pub last_update_date: Option<DateTime<Utc>>,

    /// Creation timestamp.
    #[serde(rename = "creationDate")]
    pub creation_date: Option<DateTime<Utc>>,

    /// Environment name.
    pub environment: Option<String>,

    /// VPC ID.
    #[serde(rename = "vpcId")]
    pub vpc_id: Option<String>,

    /// Static IPs enabled.
    #[serde(rename = "staticIPsEnabled")]
    pub static_ips_enabled: Option<bool>,

    /// Monitoring enabled.
    #[serde(rename = "monitoringEnabled")]
    pub monitoring_enabled: Option<bool>,

    /// Monitoring auto-restart.
    #[serde(rename = "monitoringAutoRestart")]
    pub monitoring_auto_restart: Option<bool>,

    /// Logging enabled.
    #[serde(rename = "loggingEnabled")]
    pub logging_enabled: Option<bool>,

    /// Persistent queues enabled.
    #[serde(rename = "persistentQueues")]
    pub persistent_queues: Option<bool>,

    /// Object Store v2 enabled.
    #[serde(rename = "objectStoreV2")]
    pub object_store_v2: Option<bool>,

    /// Properties.
    pub properties: Option<serde_json::Value>,

    /// Secure properties.
    #[serde(rename = "secureProperties")]
    pub secure_properties: Option<Vec<String>>,
}

/// Worker configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workers {
    /// Worker type (e.g., "Micro", "Small", "Medium").
    #[serde(rename = "type")]
    pub worker_type: String,

    /// Number of workers.
    pub quantity: u32,
}

/// Application statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationStats {
    /// Application name.
    #[serde(rename = "applicationName")]
    pub application_name: String,

    /// Memory usage.
    pub memory: ResourceMetric,

    /// CPU usage.
    pub cpu: ResourceMetric,

    /// Disk usage.
    pub disk: ResourceMetric,

    /// Number of threads.
    pub threads: Option<u32>,

    /// Heap usage percentage.
    #[serde(rename = "heapUsage")]
    pub heap_usage: Option<f64>,

    /// Uptime in seconds.
    pub uptime: Option<u64>,
}

/// Resource metric (CPU, Memory, Disk).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceMetric {
    /// Used amount.
    pub used: f64,

    /// Maximum amount.
    pub max: f64,

    /// Unit of measurement.
    pub unit: String,
}

/// Application log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    /// Timestamp.
    pub timestamp: DateTime<Utc>,

    /// Log level.
    pub level: String,

    /// Message.
    pub message: String,

    /// Application name.
    #[serde(rename = "applicationName")]
    pub application_name: Option<String>,

    /// Worker ID.
    #[serde(rename = "workerId")]
    pub worker_id: Option<String>,

    /// Thread name.
    #[serde(rename = "threadName")]
    pub thread_name: Option<String>,
}

/// Deployment model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deployment {
    /// Deployment ID.
    pub id: String,

    /// Application name.
    #[serde(rename = "applicationName")]
    pub application_name: String,

    /// Status.
    pub status: String,

    /// Start time.
    #[serde(rename = "startTime")]
    pub start_time: Option<DateTime<Utc>>,

    /// End time.
    #[serde(rename = "endTime")]
    pub end_time: Option<DateTime<Utc>>,

    /// Deployment URL.
    #[serde(rename = "deploymentUrl")]
    pub deployment_url: Option<String>,

    /// Mule version.
    #[serde(rename = "muleVersion")]
    pub mule_version: Option<String>,

    /// Runtime version.
    #[serde(rename = "runtimeVersion")]
    pub runtime_version: Option<String>,

    /// Worker configuration.
    pub workers: Option<Workers>,

    /// Region.
    pub region: Option<String>,
}

/// Alert model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    /// Alert ID.
    pub id: String,

    /// Alert name.
    pub name: String,

    /// Alert type.
    #[serde(rename = "alertType")]
    pub alert_type: String,

    /// Severity.
    pub severity: String,

    /// Application name.
    #[serde(rename = "applicationName")]
    pub application_name: Option<String>,

    /// Message.
    pub message: String,

    /// Timestamp.
    pub timestamp: DateTime<Utc>,

    /// Acknowledged.
    pub acknowledged: Option<bool>,
}

/// Instance model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    /// Instance ID.
    pub id: String,

    /// Instance name.
    pub name: String,

    /// Status.
    pub status: String,

    /// IP address.
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,

    /// Status reason.
    #[serde(rename = "statusReason")]
    pub status_reason: Option<String>,
}

/// Create application request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApplicationRequest {
    /// Application name.
    pub name: String,

    /// Worker configuration.
    pub workers: Workers,

    /// Runtime version.
    #[serde(rename = "runtimeVersion")]
    pub runtime_version: Option<String>,

    /// Region.
    pub region: Option<String>,

    /// Properties.
    pub properties: Option<serde_json::Value>,
}

/// Update application request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateApplicationRequest {
    /// Worker configuration.
    pub workers: Option<Workers>,

    /// Runtime version.
    #[serde(rename = "runtimeVersion")]
    pub runtime_version: Option<String>,

    /// Region.
    pub region: Option<String>,

    /// Monitoring enabled.
    #[serde(rename = "monitoringEnabled")]
    pub monitoring_enabled: Option<bool>,

    /// Logging enabled.
    #[serde(rename = "loggingEnabled")]
    pub logging_enabled: Option<bool>,

    /// Properties.
    pub properties: Option<serde_json::Value>,
}

/// API response wrapper.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub data: Option<T>,
    pub total: Option<u32>,
    pub page: Option<u32>,
    pub size: Option<u32>,
}

/// Status response from action endpoints.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub status: String,
    pub message: Option<String>,
}