// Integration tests for settings persistence
// These tests verify that settings can be saved and loaded correctly

use youtube_downloader_gui::storage::settings::{AppSettings, CompletedDownload, DownloadHistory};
use std::collections::HashMap;

#[test]
fn test_app_settings_default() {
    let settings = AppSettings::default();
    
    assert_eq!(settings.default_quality, "best");
    assert_eq!(settings.default_format, "mp4");
    assert_eq!(settings.max_concurrent_downloads, 3);
    assert!(settings.auto_retry_on_failure);
    assert_eq!(settings.max_retry_attempts, 3);
    assert_eq!(settings.enabled_platforms, vec!["YouTube".to_string()]);
    assert!(!settings.first_launch_completed);
}

#[test]
fn test_app_settings_serialization() {
    let mut settings = AppSettings::default();
    settings.default_save_path = "/Users/test/Downloads".to_string();
    settings.max_concurrent_downloads = 5;
    
    // Serialize to JSON
    let json = serde_json::to_string(&settings).unwrap();
    assert!(json.contains("Downloads"));
    assert!(json.contains("\"max_concurrent_downloads\":5"));
    
    // Deserialize back
    let deserialized: AppSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.default_save_path, "/Users/test/Downloads");
    assert_eq!(deserialized.max_concurrent_downloads, 5);
}

#[test]
fn test_app_settings_platform_settings() {
    let mut settings = AppSettings::default();
    
    // Add platform-specific settings
    let mut youtube_settings = HashMap::new();
    youtube_settings.insert("prefer_av1".to_string(), serde_json::json!(true));
    youtube_settings.insert("max_resolution".to_string(), serde_json::json!("1080p"));
    
    settings.platform_settings.insert("YouTube".to_string(), youtube_settings);
    
    // Verify settings
    assert!(settings.platform_settings.contains_key("YouTube"));
    let yt_settings = settings.platform_settings.get("YouTube").unwrap();
    assert_eq!(yt_settings.get("prefer_av1").unwrap(), &serde_json::json!(true));
    assert_eq!(yt_settings.get("max_resolution").unwrap(), &serde_json::json!("1080p"));
}

#[test]
fn test_download_history_default() {
    let history = DownloadHistory::default();
    assert!(history.downloads.is_empty());
}

#[test]
fn test_download_history_add_download() {
    let mut history = DownloadHistory::default();
    
    let download = CompletedDownload {
        id: "test-id".to_string(),
        video_id: "dQw4w9WgXcQ".to_string(),
        title: "Test Video".to_string(),
        completed_at: chrono::Utc::now().to_rfc3339(),
        save_path: "/Users/test/Downloads/video.mp4".to_string(),
        file_size: 1024 * 1024 * 50, // 50 MB
        platform: "YouTube".to_string(),
    };
    
    history.downloads.push(download.clone());
    
    assert_eq!(history.downloads.len(), 1);
    assert_eq!(history.downloads[0].video_id, "dQw4w9WgXcQ");
    assert_eq!(history.downloads[0].title, "Test Video");
}

#[test]
fn test_download_history_serialization() {
    let mut history = DownloadHistory::default();
    
    let download = CompletedDownload {
        id: "test-id".to_string(),
        video_id: "dQw4w9WgXcQ".to_string(),
        title: "Test Video".to_string(),
        completed_at: "2024-01-01T00:00:00Z".to_string(),
        save_path: "/Users/test/Downloads/video.mp4".to_string(),
        file_size: 1024 * 1024 * 50,
        platform: "YouTube".to_string(),
    };
    
    history.downloads.push(download);
    
    // Serialize
    let json = serde_json::to_string(&history).unwrap();
    assert!(json.contains("dQw4w9WgXcQ"));
    assert!(json.contains("Test Video"));
    
    // Deserialize
    let deserialized: DownloadHistory = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.downloads.len(), 1);
    assert_eq!(deserialized.downloads[0].video_id, "dQw4w9WgXcQ");
}

#[test]
fn test_completed_download_serialization() {
    let download = CompletedDownload {
        id: "test-id".to_string(),
        video_id: "dQw4w9WgXcQ".to_string(),
        title: "Test Video".to_string(),
        completed_at: "2024-01-01T00:00:00Z".to_string(),
        save_path: "/Users/test/Downloads/video.mp4".to_string(),
        file_size: 52428800, // 50 MB
        platform: "YouTube".to_string(),
    };
    
    let json = serde_json::to_string(&download).unwrap();
    let deserialized: CompletedDownload = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized.id, "test-id");
    assert_eq!(deserialized.video_id, "dQw4w9WgXcQ");
    assert_eq!(deserialized.title, "Test Video");
    assert_eq!(deserialized.file_size, 52428800);
    assert_eq!(deserialized.platform, "YouTube");
}

#[test]
fn test_app_settings_enabled_platforms() {
    let mut settings = AppSettings::default();
    
    // Add more platforms
    settings.enabled_platforms.push("Bilibili".to_string());
    settings.enabled_platforms.push("Vimeo".to_string());
    
    assert_eq!(settings.enabled_platforms.len(), 3);
    assert!(settings.enabled_platforms.contains(&"YouTube".to_string()));
    assert!(settings.enabled_platforms.contains(&"Bilibili".to_string()));
    assert!(settings.enabled_platforms.contains(&"Vimeo".to_string()));
}

#[test]
fn test_app_settings_validation() {
    let mut settings = AppSettings::default();
    
    // Test max concurrent downloads bounds
    settings.max_concurrent_downloads = 0;
    assert_eq!(settings.max_concurrent_downloads, 0); // Should be validated by manager
    
    settings.max_concurrent_downloads = 10;
    assert_eq!(settings.max_concurrent_downloads, 10); // Should be clamped by manager
}
