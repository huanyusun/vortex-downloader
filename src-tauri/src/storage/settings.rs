use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    pub default_save_path: String,
    pub default_quality: String,
    pub default_format: String,
    pub max_concurrent_downloads: usize,
    pub auto_retry_on_failure: bool,
    pub max_retry_attempts: usize,
    pub platform_settings: HashMap<String, HashMap<String, serde_json::Value>>,
    pub enabled_platforms: Vec<String>,
    #[serde(default)]
    pub first_launch_completed: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_save_path: String::new(),
            default_quality: "best".to_string(),
            default_format: "mp4".to_string(),
            max_concurrent_downloads: 3,
            auto_retry_on_failure: true,
            max_retry_attempts: 3,
            platform_settings: HashMap::new(),
            enabled_platforms: vec!["YouTube".to_string()],
            first_launch_completed: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QueueState {
    pub items: Vec<crate::download::DownloadItem>,
    pub last_updated: String,
}

impl Default for QueueState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownloadHistory {
    pub downloads: Vec<CompletedDownload>,
}

impl Default for DownloadHistory {
    fn default() -> Self {
        Self {
            downloads: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompletedDownload {
    pub id: String,
    pub video_id: String,
    pub title: String,
    pub completed_at: String,
    pub save_path: String,
    pub file_size: u64,
    pub platform: String,
}
