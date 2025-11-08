# Design Document

## Overview

This design addresses the download hang issue by improving yt-dlp command execution, adding comprehensive logging, and making the system more resilient to progress parsing failures. The core issue is that yt-dlp may not be producing parseable output, causing the download to appear hung even though it's actually running.

## Root Cause Analysis

Based on the logs, the download manager is correctly queuing items and starting downloads, but:
1. No progress updates are being emitted from yt-dlp
2. The progress parser may be failing silently
3. There's no visibility into what yt-dlp is actually doing

## Architecture

### Component Changes

1. **YouTubeProvider** (`src-tauri/src/platform/youtube.rs`)
   - Enhanced logging throughout download process
   - Improved yt-dlp command construction
   - Better error handling and reporting
   - Fallback behavior when progress parsing fails

2. **DownloadManager** (`src-tauri/src/download/manager.rs`)
   - Add timeout detection for hung downloads
   - Enhanced logging for debugging

## Detailed Design

### 1. Enhanced yt-dlp Command Execution

**Current Issues:**
- Output may not be in expected format
- Locale issues can cause parsing problems
- Progress output might be buffered

**Solution:**
```rust
async fn download_video_impl(&self, ...) -> Result<()> {
    let mut args = vec![
        "--newline",           // Progress on new lines
        "--no-warnings",       // Reduce noise
        "--no-color",          // No ANSI colors
        "--progress",          // Force progress output
        "-o", save_path,       // Output path
    ];
    
    // Set environment variables
    let mut cmd = Command::new(&self.ytdlp_path);
    cmd.args(&args)
       .env("PYTHONIOENCODING", "utf-8")  // Force UTF-8
       .env("LANG", "en_US.UTF-8")        // English locale
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());
}
```

### 2. Comprehensive Logging Strategy

**Log Points:**
1. Before spawning yt-dlp: log complete command
2. During execution: log every line from stdout/stderr
3. Progress parsing: log successful and failed parse attempts
4. Completion: log final status and any errors

**Implementation:**
```rust
// Log command before execution
println!("[yt-dlp] Executing: {:?} {:?}", self.ytdlp_path, args);

// Log all output
while let Ok(Some(line)) = lines.next_line().await {
    println!("[yt-dlp stdout] {}", line);
    
    if let Some(progress) = self.parse_progress_line(&line) {
        println!("[yt-dlp] Parsed progress: {:.1}%", progress.percentage);
        progress_callback(progress);
    } else {
        println!("[yt-dlp] Could not parse progress from line");
    }
}
```

### 3. Resilient Progress Handling

**Problem:** If progress parsing fails, no updates are sent, making it appear hung.

**Solution:**
- Continue download even if progress parsing fails
- Send periodic "heartbeat" updates
- Ensure completion status is sent regardless of progress updates

```rust
// Track last update time
let last_update = Arc::new(Mutex::new(Instant::now()));

// Spawn heartbeat task
let heartbeat_last_update = Arc::clone(&last_update);
tokio::spawn(async move {
    loop {
        sleep(Duration::from_secs(5)).await;
        let elapsed = heartbeat_last_update.lock().await.elapsed();
        if elapsed > Duration::from_secs(10) {
            // Send heartbeat progress
            progress_callback(DownloadProgress {
                percentage: -1.0,  // Indicates "in progress but unknown"
                downloaded_bytes: 0,
                total_bytes: 0,
                speed: 0.0,
                eta: 0,
            });
        }
    }
});
```

### 4. Improved Progress Parsing

**Current Issue:** Parser may be too strict or missing patterns.

**Solution:**
- Add more flexible regex patterns
- Handle different yt-dlp output formats
- Log failed parse attempts with the actual line

```rust
fn parse_progress_line(&self, line: &str) -> Option<DownloadProgress> {
    // Try multiple patterns
    
    // Pattern 1: [download]  45.8% of 123.45MiB at 1.23MiB/s ETA 00:42
    if let Some(progress) = self.try_parse_standard_format(line) {
        return Some(progress);
    }
    
    // Pattern 2: [download] Destination: filename.mp4
    if line.contains("[download] Destination:") {
        return Some(DownloadProgress {
            percentage: 0.0,
            downloaded_bytes: 0,
            total_bytes: 0,
            speed: 0.0,
            eta: 0,
        });
    }
    
    // Pattern 3: [download] 100% of 123.45MiB
    if line.contains("[download] 100%") {
        return Some(DownloadProgress {
            percentage: 100.0,
            downloaded_bytes: 0,
            total_bytes: 0,
            speed: 0.0,
            eta: 0,
        });
    }
    
    // Log unparseable lines for debugging
    if line.contains("[download]") {
        println!("[yt-dlp] Unparseable download line: {}", line);
    }
    
    None
}
```

### 5. Diagnostic Command

Add a test command to verify yt-dlp works:

```rust
pub async fn test_download(&self, url: &str) -> Result<String> {
    let output = Command::new(&self.ytdlp_path)
        .args(&[
            "--no-warnings",
            "--print", "title",
            url,
        ])
        .output()
        .await?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(DownloadError::DownloadFailed(
            String::from_utf8_lossy(&output.stderr).to_string()
        ))
    }
}
```

### 6. Timeout Detection

Add timeout to detect truly hung downloads:

```rust
// In execute_download
let timeout_duration = Duration::from_secs(300); // 5 minutes
let download_future = provider.download_video(...);

match tokio::time::timeout(timeout_duration, download_future).await {
    Ok(Ok(_)) => {
        // Success
    }
    Ok(Err(e)) => {
        // Download error
    }
    Err(_) => {
        // Timeout
        return Err(DownloadError::DownloadFailed(
            "Download timed out".to_string()
        ));
    }
}
```

## Error Handling

### Error Categories

1. **yt-dlp Not Found**: Clear message with installation instructions
2. **Network Errors**: Retry logic with exponential backoff
3. **Video Unavailable**: Clear message to user
4. **Timeout**: Indicate download took too long
5. **Parse Errors**: Continue download, log for debugging

### Error Messages

All errors should include:
- What went wrong
- What the user can do about it
- Relevant diagnostic information (command, output, etc.)

## Testing Strategy

### Manual Testing

1. Test with a known working video URL
2. Test with invalid URL
3. Test with unavailable video
4. Test with slow network
5. Test with yt-dlp not installed

### Diagnostic Output

When a download is initiated, log:
```
[yt-dlp] Version: 2023.xx.xx
[ffmpeg] Version: 6.x.x
[yt-dlp] Command: /path/to/yt-dlp --newline --no-warnings ...
[yt-dlp] URL: https://youtube.com/watch?v=...
[yt-dlp] Save path: /Users/xxx/Downloads/video.mp4
```

## Implementation Notes

1. All changes should be backward compatible
2. Logging should be controlled by log level (use `println!` for now, migrate to proper logging later)
3. Progress parsing should be defensive - never panic
4. Always emit final status (completed/failed) regardless of progress updates

## Alternative Considered: Not Using yt-dlp

**Pros:**
- No external dependency
- Full control over implementation

**Cons:**
- YouTube actively fights scrapers - constant maintenance required
- Need to implement format selection, audio/video merging, etc.
- Would take weeks/months to implement reliably
- yt-dlp has years of community development

**Decision:** Keep yt-dlp, fix the integration issues. The problem is not yt-dlp itself, but how we're using it.
