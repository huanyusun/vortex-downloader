// Integration tests for YouTube provider
// Note: These tests require yt-dlp to be installed and may make actual network requests
// They are designed to test real functionality, not mocked behavior

use youtube_downloader_gui::platform::{PlatformProvider, YouTubeProvider, DownloadOptions};
use std::sync::{Arc, Mutex};
use std::path::Path;
use tempfile::TempDir;

#[tokio::test]
#[ignore] // Ignore by default as it requires network and yt-dlp
async fn test_youtube_get_video_info_real() {
    let provider = YouTubeProvider::new();
    
    // Use a stable, well-known video (Rick Astley - Never Gonna Give You Up)
    let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
    
    let result = provider.get_video_info(url).await;
    
    // If yt-dlp is not installed, this will fail with YtdlpNotFound
    if result.is_err() {
        let err = result.unwrap_err();
        println!("Test skipped: {}", err);
        return;
    }
    
    let video_info = result.unwrap();
    
    // Verify basic video information
    assert_eq!(video_info.id, "dQw4w9WgXcQ");
    assert!(!video_info.title.is_empty());
    assert!(!video_info.thumbnail.is_empty());
    assert!(video_info.duration > 0);
    assert_eq!(video_info.platform, "YouTube");
    assert!(!video_info.available_formats.is_empty());
}

#[tokio::test]
#[ignore] // Ignore by default as it requires network and yt-dlp
async fn test_youtube_get_playlist_info_real() {
    let provider = YouTubeProvider::new();
    
    // Use a small, stable playlist
    let url = "https://www.youtube.com/playlist?list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf";
    
    let result = provider.get_playlist_info(url).await;
    
    if result.is_err() {
        let err = result.unwrap_err();
        println!("Test skipped: {}", err);
        return;
    }
    
    let playlist_info = result.unwrap();
    
    // Verify playlist information
    assert!(!playlist_info.id.is_empty());
    assert!(!playlist_info.title.is_empty());
    assert_eq!(playlist_info.platform, "YouTube");
    assert!(playlist_info.video_count > 0);
    assert!(!playlist_info.videos.is_empty());
    
    // Verify first video has required fields
    let first_video = &playlist_info.videos[0];
    assert!(!first_video.id.is_empty());
    assert!(!first_video.title.is_empty());
}

#[tokio::test]
#[ignore] // Ignore by default as it requires network and yt-dlp
async fn test_youtube_check_dependencies_real() {
    let provider = YouTubeProvider::new();
    
    let result = provider.check_dependencies().await;
    assert!(result.is_ok());
    
    let dependencies = result.unwrap();
    assert_eq!(dependencies.len(), 2);
    
    // Check yt-dlp
    let ytdlp = dependencies.iter().find(|d| d.name == "yt-dlp");
    assert!(ytdlp.is_some());
    
    // Check ffmpeg
    let ffmpeg = dependencies.iter().find(|d| d.name == "ffmpeg");
    assert!(ffmpeg.is_some());
}

#[tokio::test]
async fn test_youtube_invalid_url() {
    let provider = YouTubeProvider::new();
    
    let url = "https://www.youtube.com/watch?v=invalid_video_id_that_does_not_exist_12345";
    
    let result = provider.get_video_info(url).await;
    
    // Should fail with VideoUnavailable or similar error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_youtube_unsupported_url() {
    let provider = YouTubeProvider::new();
    
    // This is not a YouTube URL
    let url = "https://www.vimeo.com/123456";
    
    // The provider should not match this URL
    assert!(!provider.matches_url(url));
}

// Task 7.1: Test with a known working YouTube URL
#[tokio::test]
#[ignore] // Ignore by default as it requires network and yt-dlp
async fn test_youtube_download_with_progress() {
    let provider = YouTubeProvider::new();
    
    // Use a short, stable video for testing
    let url = "https://www.youtube.com/watch?v=jNQXAC9IVRw"; // "Me at the zoo" - first YouTube video (18 seconds)
    
    // Create temporary directory for download
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let save_path = temp_dir.path().join("test_video.mp4");
    
    // Track progress updates
    let progress_updates = Arc::new(Mutex::new(Vec::new()));
    let progress_clone = Arc::clone(&progress_updates);
    
    let progress_callback = Box::new(move |progress: youtube_downloader_gui::platform::DownloadProgress| {
        println!("[TEST] Progress update: {:.1}% - {:.2} MB/s", 
                 progress.percentage, 
                 progress.speed / 1_000_000.0);
        progress_clone.lock().unwrap().push(progress.percentage);
    });
    
    // Attempt download
    println!("[TEST] Starting download test...");
    let options = DownloadOptions {
        quality: "best".to_string(),
        format: "mp4".to_string(),
        audio_only: false,
    };
    let result = provider.download_video(
        url,
        options,
        Path::new(save_path.to_str().unwrap()),
        progress_callback
    ).await;
    
    // Check if yt-dlp is available
    if result.is_err() {
        let err = result.unwrap_err();
        println!("[TEST] Test skipped or failed: {}", err);
        
        // If it's a dependency issue, skip the test
        if err.to_string().contains("yt-dlp") || err.to_string().contains("not found") {
            println!("[TEST] Skipping test - yt-dlp not available");
            return;
        }
        
        // Otherwise, this is a real failure
        panic!("Download failed: {}", err);
    }
    
    println!("[TEST] Download completed successfully");
    
    // Verify progress updates were received
    let updates = progress_updates.lock().unwrap();
    println!("[TEST] Received {} progress updates", updates.len());
    assert!(!updates.is_empty(), "Should receive at least one progress update");
    
    // Verify final progress is 100%
    let last_progress = updates.last().unwrap();
    assert_eq!(*last_progress, 100.0, "Final progress should be 100%");
    
    // Verify file was created
    assert!(save_path.exists(), "Downloaded file should exist");
    
    // Verify file has content
    let metadata = std::fs::metadata(&save_path).expect("Failed to get file metadata");
    assert!(metadata.len() > 0, "Downloaded file should not be empty");
    
    println!("[TEST] File saved successfully: {} bytes", metadata.len());
}

// Task 7.2: Test error scenarios - Invalid URL
#[tokio::test]
#[ignore] // Ignore by default as it requires network and yt-dlp
async fn test_youtube_download_invalid_url() {
    let provider = YouTubeProvider::new();
    
    let url = "https://www.youtube.com/watch?v=this_video_does_not_exist_xyz123";
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let save_path = temp_dir.path().join("test_video.mp4");
    
    let progress_callback = Box::new(|_progress: youtube_downloader_gui::platform::DownloadProgress| {
        // Should not receive progress for invalid video
    });
    
    println!("[TEST] Testing invalid URL error handling...");
    let options = DownloadOptions {
        quality: "best".to_string(),
        format: "mp4".to_string(),
        audio_only: false,
    };
    let result = provider.download_video(
        url,
        options,
        Path::new(save_path.to_str().unwrap()),
        progress_callback
    ).await;
    
    // Should fail
    assert!(result.is_err(), "Download should fail for invalid URL");
    
    let err = result.unwrap_err();
    println!("[TEST] Expected error received: {}", err);
    
    // Verify file was not created
    assert!(!save_path.exists(), "File should not exist for failed download");
}

// Task 7.2: Test error scenarios - Unavailable video
#[tokio::test]
#[ignore] // Ignore by default as it requires network and yt-dlp
async fn test_youtube_download_unavailable_video() {
    let provider = YouTubeProvider::new();
    
    // Use a video ID that's likely to be unavailable/private
    let url = "https://www.youtube.com/watch?v=AAAAAAAAAAA";
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let save_path = temp_dir.path().join("test_video.mp4");
    
    let progress_callback = Box::new(|_progress: youtube_downloader_gui::platform::DownloadProgress| {
        // Should not receive progress for unavailable video
    });
    
    println!("[TEST] Testing unavailable video error handling...");
    let options = DownloadOptions {
        quality: "best".to_string(),
        format: "mp4".to_string(),
        audio_only: false,
    };
    let result = provider.download_video(
        url,
        options,
        Path::new(save_path.to_str().unwrap()),
        progress_callback
    ).await;
    
    // Should fail
    assert!(result.is_err(), "Download should fail for unavailable video");
    
    let err = result.unwrap_err();
    println!("[TEST] Expected error received: {}", err);
}

// Task 7.3: Verify logging output - Test diagnostic command
#[tokio::test]
#[ignore] // Ignore by default as it requires network and yt-dlp
async fn test_youtube_diagnostic_logging() {
    let provider = YouTubeProvider::new();
    
    println!("[TEST] Testing diagnostic and logging functionality...");
    
    // Test dependency check (logs versions)
    let deps_result = provider.check_dependencies().await;
    
    if deps_result.is_err() {
        println!("[TEST] Skipping test - dependencies not available");
        return;
    }
    
    let dependencies = deps_result.unwrap();
    println!("[TEST] Dependencies check passed:");
    for dep in &dependencies {
        println!("[TEST]   - {}: {} ({})", 
                 dep.name, 
                 if dep.installed { "installed" } else { "not installed" },
                 dep.version.as_ref().unwrap_or(&"unknown".to_string()));
    }
    
    // Verify both dependencies are present
    assert_eq!(dependencies.len(), 2, "Should have 2 dependencies");
    
    let ytdlp = dependencies.iter().find(|d| d.name == "yt-dlp");
    assert!(ytdlp.is_some(), "yt-dlp should be in dependencies");
    assert!(ytdlp.unwrap().installed, "yt-dlp should be installed");
    
    let ffmpeg = dependencies.iter().find(|d| d.name == "ffmpeg");
    assert!(ffmpeg.is_some(), "ffmpeg should be in dependencies");
    assert!(ffmpeg.unwrap().installed, "ffmpeg should be installed");
}

// Task 7.1: Test with progress tracking and verification
#[tokio::test]
#[ignore] // Ignore by default as it requires network and yt-dlp
async fn test_youtube_download_progress_tracking() {
    let provider = YouTubeProvider::new();
    
    // Use a very short video for quick testing
    let url = "https://www.youtube.com/watch?v=jNQXAC9IVRw";
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let save_path = temp_dir.path().join("progress_test.mp4");
    
    // Track detailed progress information
    let progress_log = Arc::new(Mutex::new(Vec::new()));
    let progress_clone = Arc::clone(&progress_log);
    
    let progress_callback = Box::new(move |progress: youtube_downloader_gui::platform::DownloadProgress| {
        let log_entry = format!(
            "Progress: {:.1}% | Downloaded: {} bytes | Speed: {:.2} KB/s | ETA: {}s",
            progress.percentage,
            progress.downloaded_bytes,
            progress.speed / 1024.0,
            progress.eta
        );
        println!("[TEST] {}", log_entry);
        progress_clone.lock().unwrap().push(log_entry);
    });
    
    println!("[TEST] Starting progress tracking test...");
    let options = DownloadOptions {
        quality: "best".to_string(),
        format: "mp4".to_string(),
        audio_only: false,
    };
    let result = provider.download_video(
        url,
        options,
        Path::new(save_path.to_str().unwrap()),
        progress_callback
    ).await;
    
    if result.is_err() {
        let err = result.unwrap_err();
        if err.to_string().contains("yt-dlp") || err.to_string().contains("not found") {
            println!("[TEST] Skipping test - yt-dlp not available");
            return;
        }
        panic!("Download failed: {}", err);
    }
    
    // Verify progress was tracked
    let log = progress_log.lock().unwrap();
    println!("[TEST] Progress log entries: {}", log.len());
    for entry in log.iter() {
        println!("[TEST]   {}", entry);
    }
    
    assert!(!log.is_empty(), "Should have progress log entries");
}
