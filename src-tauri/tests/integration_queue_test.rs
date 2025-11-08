// Integration tests for download queue persistence and management
// These tests verify queue state management without requiring full Tauri runtime

use youtube_downloader_gui::download::{DownloadItem, DownloadStatus};
use youtube_downloader_gui::storage::settings::QueueState;

#[test]
fn test_queue_state_default() {
    let queue = QueueState::default();
    
    assert!(queue.items.is_empty());
    assert!(!queue.last_updated.is_empty());
}

#[test]
fn test_queue_state_add_items() {
    let mut queue = QueueState::default();
    
    let item = DownloadItem {
        id: "test-1".to_string(),
        video_id: "dQw4w9WgXcQ".to_string(),
        title: "Test Video".to_string(),
        thumbnail: "https://example.com/thumb.jpg".to_string(),
        status: DownloadStatus::Queued,
        progress: 0.0,
        speed: 0.0,
        eta: 0,
        save_path: "/Users/test/Downloads/video.mp4".to_string(),
        error: None,
        url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
        platform: "YouTube".to_string(),
    };
    
    queue.items.push(item);
    
    assert_eq!(queue.items.len(), 1);
    assert_eq!(queue.items[0].video_id, "dQw4w9WgXcQ");
    assert_eq!(queue.items[0].status, DownloadStatus::Queued);
}

#[test]
fn test_queue_state_serialization() {
    let mut queue = QueueState::default();
    
    let item = DownloadItem {
        id: "test-1".to_string(),
        video_id: "dQw4w9WgXcQ".to_string(),
        title: "Test Video".to_string(),
        thumbnail: "https://example.com/thumb.jpg".to_string(),
        status: DownloadStatus::Downloading,
        progress: 45.5,
        speed: 1024.0 * 1024.0, // 1 MB/s
        eta: 120,
        save_path: "/Users/test/Downloads/video.mp4".to_string(),
        error: None,
        url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
        platform: "YouTube".to_string(),
    };
    
    queue.items.push(item);
    
    // Serialize
    let json = serde_json::to_string(&queue).unwrap();
    assert!(json.contains("dQw4w9WgXcQ"));
    assert!(json.contains("Downloading"));
    
    // Deserialize
    let deserialized: QueueState = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.items.len(), 1);
    assert_eq!(deserialized.items[0].video_id, "dQw4w9WgXcQ");
    assert_eq!(deserialized.items[0].status, DownloadStatus::Downloading);
    assert_eq!(deserialized.items[0].progress, 45.5);
}

#[test]
fn test_download_item_status_transitions() {
    let mut item = DownloadItem {
        id: "test-1".to_string(),
        video_id: "dQw4w9WgXcQ".to_string(),
        title: "Test Video".to_string(),
        thumbnail: "https://example.com/thumb.jpg".to_string(),
        status: DownloadStatus::Queued,
        progress: 0.0,
        speed: 0.0,
        eta: 0,
        save_path: "/Users/test/Downloads/video.mp4".to_string(),
        error: None,
        url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
        platform: "YouTube".to_string(),
    };
    
    // Queued -> Downloading
    item.status = DownloadStatus::Downloading;
    assert_eq!(item.status, DownloadStatus::Downloading);
    
    // Downloading -> Paused
    item.status = DownloadStatus::Paused;
    assert_eq!(item.status, DownloadStatus::Paused);
    
    // Paused -> Downloading
    item.status = DownloadStatus::Downloading;
    assert_eq!(item.status, DownloadStatus::Downloading);
    
    // Downloading -> Completed
    item.status = DownloadStatus::Completed;
    item.progress = 100.0;
    assert_eq!(item.status, DownloadStatus::Completed);
    assert_eq!(item.progress, 100.0);
}

#[test]
fn test_download_item_with_error() {
    let item = DownloadItem {
        id: "test-1".to_string(),
        video_id: "invalid".to_string(),
        title: "Failed Video".to_string(),
        thumbnail: "https://example.com/thumb.jpg".to_string(),
        status: DownloadStatus::Failed,
        progress: 25.0,
        speed: 0.0,
        eta: 0,
        save_path: "/Users/test/Downloads/video.mp4".to_string(),
        error: Some("Network error: Connection timeout".to_string()),
        url: "https://www.youtube.com/watch?v=invalid".to_string(),
        platform: "YouTube".to_string(),
    };
    
    assert_eq!(item.status, DownloadStatus::Failed);
    assert!(item.error.is_some());
    assert!(item.error.unwrap().contains("Network error"));
}

#[test]
fn test_queue_state_multiple_items() {
    let mut queue = QueueState::default();
    
    // Add multiple items with different statuses
    for i in 0..5 {
        let status = match i {
            0 => DownloadStatus::Completed,
            1 => DownloadStatus::Downloading,
            2 => DownloadStatus::Queued,
            3 => DownloadStatus::Paused,
            _ => DownloadStatus::Failed,
        };
        
        let item = DownloadItem {
            id: format!("test-{}", i),
            video_id: format!("video-{}", i),
            title: format!("Test Video {}", i),
            thumbnail: "https://example.com/thumb.jpg".to_string(),
            status,
            progress: if i == 0 { 100.0 } else { i as f64 * 10.0 },
            speed: 0.0,
            eta: 0,
            save_path: format!("/Users/test/Downloads/video-{}.mp4", i),
            error: if i == 4 { Some("Test error".to_string()) } else { None },
            url: format!("https://www.youtube.com/watch?v=video-{}", i),
            platform: "YouTube".to_string(),
        };
        
        queue.items.push(item);
    }
    
    assert_eq!(queue.items.len(), 5);
    
    // Count items by status
    let completed = queue.items.iter().filter(|i| i.status == DownloadStatus::Completed).count();
    let downloading = queue.items.iter().filter(|i| i.status == DownloadStatus::Downloading).count();
    let queued = queue.items.iter().filter(|i| i.status == DownloadStatus::Queued).count();
    let paused = queue.items.iter().filter(|i| i.status == DownloadStatus::Paused).count();
    let failed = queue.items.iter().filter(|i| i.status == DownloadStatus::Failed).count();
    
    assert_eq!(completed, 1);
    assert_eq!(downloading, 1);
    assert_eq!(queued, 1);
    assert_eq!(paused, 1);
    assert_eq!(failed, 1);
}

#[test]
fn test_queue_state_restore_after_crash() {
    let mut queue = QueueState::default();
    
    // Simulate items that were downloading when app crashed
    let item1 = DownloadItem {
        id: "test-1".to_string(),
        video_id: "video-1".to_string(),
        title: "Test Video 1".to_string(),
        thumbnail: "https://example.com/thumb.jpg".to_string(),
        status: DownloadStatus::Downloading,
        progress: 45.0,
        speed: 1024.0 * 1024.0,
        eta: 60,
        save_path: "/Users/test/Downloads/video-1.mp4".to_string(),
        error: None,
        url: "https://www.youtube.com/watch?v=video-1".to_string(),
        platform: "YouTube".to_string(),
    };
    
    let item2 = DownloadItem {
        id: "test-2".to_string(),
        video_id: "video-2".to_string(),
        title: "Test Video 2".to_string(),
        thumbnail: "https://example.com/thumb.jpg".to_string(),
        status: DownloadStatus::Queued,
        progress: 0.0,
        speed: 0.0,
        eta: 0,
        save_path: "/Users/test/Downloads/video-2.mp4".to_string(),
        error: None,
        url: "https://www.youtube.com/watch?v=video-2".to_string(),
        platform: "YouTube".to_string(),
    };
    
    queue.items.push(item1);
    queue.items.push(item2);
    
    // Serialize (simulating save before crash)
    let json = serde_json::to_string(&queue).unwrap();
    
    // Deserialize (simulating restore after restart)
    let mut restored: QueueState = serde_json::from_str(&json).unwrap();
    
    // Reset downloading items to queued (as DownloadManager does)
    for item in &mut restored.items {
        if item.status == DownloadStatus::Downloading {
            item.status = DownloadStatus::Queued;
            item.progress = 0.0;
            item.speed = 0.0;
            item.eta = 0;
        }
    }
    
    // Verify restoration
    assert_eq!(restored.items.len(), 2);
    assert_eq!(restored.items[0].status, DownloadStatus::Queued); // Was downloading, now queued
    assert_eq!(restored.items[1].status, DownloadStatus::Queued); // Was queued, still queued
}

#[test]
fn test_download_status_equality() {
    assert_eq!(DownloadStatus::Queued, DownloadStatus::Queued);
    assert_ne!(DownloadStatus::Queued, DownloadStatus::Downloading);
    assert_ne!(DownloadStatus::Completed, DownloadStatus::Failed);
}
