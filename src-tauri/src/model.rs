use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub id: String,
    pub name: String,
    pub version: String,
    pub platform: String,
    pub directory: String,
    pub jar_path: Option<String>,
    pub java_path: Option<String>,
    pub memory_min_mb: u32,
    pub memory_max_mb: u32,
    pub storage_target_gb: u32,
    pub created_at: String,
    pub state: String,
    pub image_data: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateServer {
    pub name: String,
    pub version: String,
    pub platform: String,
    pub memory_min_mb: u32,
    pub memory_max_mb: u32,
    pub max_players: u32,
    pub port: u16,
    pub storage_target_gb: u32,
    pub image_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportInspection {
    pub path: String,
    pub suggested_name: String,
    pub jar_files: Vec<String>,
    pub detected_version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportServer {
    pub path: String,
    pub name: String,
    pub version: String,
    pub platform: String,
    pub jar_file: String,
    pub copy: bool,
    pub storage_target_gb: u32,
    pub image_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub id: String,
    pub kind: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub server_id: String,
    pub label: String,
    pub received: u64,
    pub total: Option<u64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateEvent {
    pub server_id: String,
    pub state: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent {
    pub server_id: String,
    pub level: String,
    pub line: String,
    pub message: String,
    pub timestamp: Option<String>,
    pub event: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    pub pid: Option<u32>,
    pub uptime_seconds: Option<u64>,
    pub memory_bytes: Option<u64>,
    pub cpu_percent: Option<f32>,
    pub size_bytes: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Backup {
    pub id: String,
    pub server_id: String,
    pub created_at: String,
    pub size_bytes: u64,
    pub path: String,
}
