use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::path::Path;
use crate::error::Result;

/// Trait that all platform providers must implement
#[async_trait]
pub trait PlatformProvider: Send + Sync {
    /// Get platform name
    fn name(&self) -> &str;
    
    /// Check if URL belongs to this platform
    fn matches_url(&self, url: &str) -> bool;
    
    /// Get supported URL patterns (for UI hints)
    fn supported_patterns(&self) -> Vec<String>;
    
    /// Get video information
    async fn get_video_info(&self, url: &str) -> Result<VideoInfo>;
    
    /// Get playlist information
    async fn get_playlist_info(&self, url: &str) -> Result<PlaylistInfo>;
    
    /// Get channel information
    async fn get_channel_info(&self, url: &str) -> Result<ChannelInfo>;
    
    /// Download video
    async fn download_video(
        &self,
        url: &str,
        options: DownloadOptions,
        save_path: &Path,
        progress_callback: Box<dyn Fn(DownloadProgress) + Send>,
    ) -> Result<()>;
    
    /// Check platform dependencies
    async fn check_dependencies(&self) -> Result<Vec<Dependency>>;
    
    /// Get platform-specific settings
    fn get_platform_settings(&self) -> Vec<PlatformSetting>;
    
    /// Enable downcasting to concrete types
    fn as_any(&self) -> &dyn Any;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VideoInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub thumbnail: String,
    pub duration: u64,
    pub uploader: String,
    pub upload_date: String,
    pub view_count: u64,
    pub available_formats: Vec<FormatInfo>,
    pub platform: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlaylistInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub uploader: String,
    pub video_count: usize,
    pub videos: Vec<VideoInfo>,
    pub platform: String,
    pub url: String,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub page: usize,
    #[serde(default)]
    pub page_size: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChannelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub playlists: Vec<PlaylistInfo>,
    pub all_videos: Vec<VideoInfo>,
    pub platform: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FormatInfo {
    pub format_id: String,
    pub ext: String,
    pub resolution: Option<String>,
    pub filesize: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownloadOptions {
    pub quality: String,
    pub format: String,
    pub audio_only: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownloadProgress {
    pub percentage: f64,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed: f64,
    pub eta: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Dependency {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub install_instructions: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlatformSetting {
    pub key: String,
    pub label: String,
    pub setting_type: SettingType,
    pub default_value: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum SettingType {
    Boolean,
    String,
    Number,
    Select { options: Vec<String> },
}
