use async_trait::async_trait;
use regex::Regex;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use super::provider::*;
use crate::error::{DownloadError, Result};

/// YouTube platform provider using yt-dlp
pub struct YouTubeProvider {
    ytdlp_path: PathBuf,
    ffmpeg_path: PathBuf,
    url_patterns: Vec<Regex>,
}

impl YouTubeProvider {
    pub fn new() -> Self {
        // Compile URL patterns for efficient matching
        let url_patterns = vec![
            // Standard video URLs
            Regex::new(r"^https?://(www\.)?youtube\.com/watch\?v=[\w-]+").unwrap(),
            // Short URLs
            Regex::new(r"^https?://youtu\.be/[\w-]+").unwrap(),
            // Playlist URLs
            Regex::new(r"^https?://(www\.)?youtube\.com/playlist\?list=[\w-]+").unwrap(),
            // Channel URLs (new format with @)
            Regex::new(r"^https?://(www\.)?youtube\.com/@[\w-]+").unwrap(),
            // Channel URLs (old format)
            Regex::new(r"^https?://(www\.)?youtube\.com/channel/[\w-]+").unwrap(),
            // User URLs
            Regex::new(r"^https?://(www\.)?youtube\.com/user/[\w-]+").unwrap(),
            // Channel custom URLs
            Regex::new(r"^https?://(www\.)?youtube\.com/c/[\w-]+").unwrap(),
        ];
        
        Self {
            ytdlp_path: PathBuf::from("yt-dlp"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            url_patterns,
        }
    }
    
    /// Create a new YouTubeProvider with custom executable paths
    pub fn with_executables(ytdlp_path: PathBuf, ffmpeg_path: PathBuf) -> Self {
        // Compile URL patterns for efficient matching
        let url_patterns = vec![
            // Standard video URLs
            Regex::new(r"^https?://(www\.)?youtube\.com/watch\?v=[\w-]+").unwrap(),
            // Short URLs
            Regex::new(r"^https?://youtu\.be/[\w-]+").unwrap(),
            // Playlist URLs
            Regex::new(r"^https?://(www\.)?youtube\.com/playlist\?list=[\w-]+").unwrap(),
            // Channel URLs (new format with @)
            Regex::new(r"^https?://(www\.)?youtube\.com/@[\w-]+").unwrap(),
            // Channel URLs (old format)
            Regex::new(r"^https?://(www\.)?youtube\.com/channel/[\w-]+").unwrap(),
            // User URLs
            Regex::new(r"^https?://(www\.)?youtube\.com/user/[\w-]+").unwrap(),
            // Channel custom URLs
            Regex::new(r"^https?://(www\.)?youtube\.com/c/[\w-]+").unwrap(),
        ];
        
        Self {
            ytdlp_path,
            ffmpeg_path,
            url_patterns,
        }
    }
    
    /// Check if yt-dlp is installed
    pub async fn check_installation(&self) -> bool {
        match Command::new(&self.ytdlp_path)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
        {
            Ok(status) => status.success(),
            Err(_) => false,
        }
    }
    
    /// Update yt-dlp to latest version
    pub async fn update_ytdlp(&self) -> Result<()> {
        let output = Command::new(&self.ytdlp_path)
            .arg("-U")
            .output()
            .await
            .map_err(|e| DownloadError::DownloadFailed(format!("Failed to update yt-dlp: {}", e)))?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(DownloadError::DownloadFailed(format!("yt-dlp update failed: {}", error)));
        }
        
        Ok(())
    }
    
    /// Execute yt-dlp command and return stdout
    async fn execute_ytdlp(&self, args: &[&str]) -> Result<String> {
        let output = Command::new(&self.ytdlp_path)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    DownloadError::YtdlpNotFound
                } else {
                    DownloadError::DownloadFailed(format!("Failed to execute yt-dlp: {}", e))
                }
            })?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            
            // Parse common error messages
            if error.contains("Video unavailable") || error.contains("Private video") {
                return Err(DownloadError::VideoUnavailable(error.to_string()));
            } else if error.contains("network") || error.contains("timeout") {
                return Err(DownloadError::Network(error.to_string()));
            } else {
                return Err(DownloadError::DownloadFailed(error.to_string()));
            }
        }
        
        String::from_utf8(output.stdout)
            .map_err(|e| DownloadError::DownloadFailed(format!("Invalid UTF-8 output: {}", e)))
    }
    
    /// Parse video info from yt-dlp JSON output
    fn parse_video_info(&self, json: &Value, url: &str) -> Result<VideoInfo> {
        Ok(VideoInfo {
            id: json["id"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            title: json["title"]
                .as_str()
                .unwrap_or("Unknown Title")
                .to_string(),
            description: json["description"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            thumbnail: json["thumbnail"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            duration: json["duration"]
                .as_u64()
                .unwrap_or(0),
            uploader: json["uploader"]
                .as_str()
                .or_else(|| json["channel"].as_str())
                .unwrap_or("Unknown")
                .to_string(),
            upload_date: json["upload_date"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            view_count: json["view_count"]
                .as_u64()
                .unwrap_or(0),
            available_formats: self.parse_formats(json),
            platform: "YouTube".to_string(),
            url: url.to_string(),
        })
    }
    
    /// Parse available formats from yt-dlp JSON
    fn parse_formats(&self, json: &Value) -> Vec<FormatInfo> {
        let mut formats = Vec::new();
        
        if let Some(formats_array) = json["formats"].as_array() {
            for format in formats_array {
                if let Some(format_id) = format["format_id"].as_str() {
                    formats.push(FormatInfo {
                        format_id: format_id.to_string(),
                        ext: format["ext"]
                            .as_str()
                            .unwrap_or("mp4")
                            .to_string(),
                        resolution: format["resolution"]
                            .as_str()
                            .map(|s| s.to_string()),
                        filesize: format["filesize"]
                            .as_u64()
                            .or_else(|| format["filesize_approx"].as_u64()),
                    });
                }
            }
        }
        
        formats
    }
    
    /// Extract playlist ID from channel info to get all uploads
    fn extract_uploads_playlist_id(&self, json: &Value) -> Option<String> {
        // Try to get the uploads playlist ID from channel info
        json["channel_id"]
            .as_str()
            .map(|channel_id| {
                // YouTube uploads playlist ID is "UU" + channel_id[2..]
                if channel_id.starts_with("UC") {
                    format!("UU{}", &channel_id[2..])
                } else {
                    channel_id.to_string()
                }
            })
    }
    
    /// Internal download implementation with cancellation support
    async fn download_video_impl(
        &self,
        url: &str,
        options: DownloadOptions,
        save_path: &Path,
        progress_callback: Box<dyn Fn(DownloadProgress) + Send>,
        cancel_token: Option<CancellationToken>,
    ) -> Result<()> {
        // Ensure save_path is properly handled (yt-dlp handles escaping internally)
        let save_path_str = save_path.to_str()
            .ok_or_else(|| DownloadError::DownloadFailed(
                format!("Invalid save path: {:?}", save_path)
            ))?;
        
        // Validate ffmpeg path exists before starting download
        if !self.ffmpeg_path.exists() {
            return Err(DownloadError::DownloadFailed(
                format!("ffmpeg not found at: {:?}", self.ffmpeg_path)
            ));
        }
        
        // Get ffmpeg location and handle paths with spaces
        let ffmpeg_location = self.ffmpeg_path.to_str()
            .ok_or_else(|| DownloadError::DownloadFailed(
                format!("Invalid ffmpeg path: {:?}", self.ffmpeg_path)
            ))?;
        
        // Build yt-dlp command arguments
        let mut args = vec![
            "--newline",      // Output progress on new lines for easier parsing
            "--no-color",     // Prevent ANSI color codes
            "--progress",     // Force progress output
            "--no-warnings",  // Reduce noise in output
            "--no-playlist",  // Don't download playlists
            "-o", save_path_str,  // Output template (yt-dlp handles special characters)
        ];
        
        // Specify ffmpeg location (yt-dlp handles quoting internally)
        args.push("--ffmpeg-location");
        args.push(ffmpeg_location);
        
        // Add format selection based on options
        let format_arg = self.build_format_string(&options);
        args.push("-f");
        args.push(&format_arg);
        
        // Add audio-only flag if needed
        if options.audio_only {
            args.push("-x");  // Extract audio
            args.push("--audio-format");
            args.push(&options.format);
        }
        
        // Add URL
        args.push(url);
        
        // Log the complete command before execution
        println!("[yt-dlp] Executing command: {:?} {:?}", self.ytdlp_path, args);
        println!("[yt-dlp] URL: {}", url);
        println!("[yt-dlp] Save path: {}", save_path.display());
        println!("[yt-dlp] Format: {}", format_arg);
        println!("[yt-dlp] Audio only: {}", options.audio_only);
        
        // Spawn yt-dlp process with piped stdout for progress
        let mut child = Command::new(&self.ytdlp_path)
            .args(&args)
            .env("PYTHONIOENCODING", "utf-8")  // Force UTF-8 encoding
            .env("LANG", "en_US.UTF-8")        // Set English locale
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    println!("[yt-dlp] ERROR: yt-dlp executable not found at {:?}", self.ytdlp_path);
                    DownloadError::YtdlpNotFound
                } else {
                    println!("[yt-dlp] ERROR: Failed to spawn yt-dlp: {}", e);
                    DownloadError::DownloadFailed(format!("Failed to spawn yt-dlp: {}", e))
                }
            })?;
        
        // Get stdout for progress monitoring (yt-dlp outputs progress to stdout with --newline)
        let stdout = child.stdout.take().ok_or_else(|| {
            println!("[yt-dlp] ERROR: Failed to capture yt-dlp stdout");
            DownloadError::DownloadFailed("Failed to capture yt-dlp stdout".to_string())
        })?;
        
        // Also capture stderr for error messages
        let stderr = child.stderr.take().ok_or_else(|| {
            println!("[yt-dlp] WARNING: Failed to capture yt-dlp stderr");
            DownloadError::DownloadFailed("Failed to capture yt-dlp stderr".to_string())
        })?;
        
        let stdout_reader = BufReader::new(stdout);
        let mut stdout_lines = stdout_reader.lines();
        
        let stderr_reader = BufReader::new(stderr);
        let mut stderr_lines = stderr_reader.lines();
        
        // Wrap child in Arc<Mutex> for shared access
        let child = Arc::new(Mutex::new(child));
        let child_clone = child.clone();
        
        // Spawn task to monitor cancellation
        if let Some(token) = cancel_token {
            let child_for_cancel = child_clone.clone();
            tokio::spawn(async move {
                token.cancelled().await;
                println!("[yt-dlp] Cancellation requested, killing process");
                // Kill the process when cancelled
                if let Ok(mut child) = child_for_cancel.try_lock() {
                    let _ = child.kill().await;
                }
            });
        }
        
        // Spawn task to read and log stderr in real-time
        tokio::spawn(async move {
            while let Ok(Some(line)) = stderr_lines.next_line().await {
                println!("[yt-dlp stderr] {}", line);
            }
        });
        
        // Parse progress from stdout
        println!("[yt-dlp] Starting to monitor download progress...");
        while let Ok(Some(line)) = stdout_lines.next_line().await {
            // Log all stdout output in real-time
            println!("[yt-dlp stdout] {}", line);
            
            // Attempt to parse progress from the line
            if let Some(progress) = self.parse_progress_line(&line) {
                println!("[yt-dlp] ✓ Parsed progress: {:.1}% (downloaded: {} bytes, total: {} bytes, speed: {:.2} MB/s, ETA: {}s)", 
                         progress.percentage, 
                         progress.downloaded_bytes,
                         progress.total_bytes,
                         progress.speed / (1024.0 * 1024.0), 
                         progress.eta);
                progress_callback(progress);
            } else if line.contains("[download]") {
                // Log when we encounter a download line that we couldn't parse
                println!("[yt-dlp] ✗ Could not parse progress from download line: {}", line);
            }
        }
        
        // Wait for process to complete
        println!("[yt-dlp] Waiting for process to complete...");
        let status = child.lock().await.wait().await
            .map_err(|e| {
                println!("[yt-dlp] ERROR: Failed to wait for yt-dlp process: {}", e);
                DownloadError::DownloadFailed(format!("Failed to wait for yt-dlp: {}", e))
            })?;
        
        if !status.success() {
            println!("[yt-dlp] ✗ Download FAILED with exit status: {}", status);
            let error_msg = format!("yt-dlp exited with status: {} (check stderr output above for details)", status);
            return Err(DownloadError::DownloadFailed(error_msg));
        }
        
        println!("[yt-dlp] ✓ Download completed successfully");
        
        // Always send 100% progress when yt-dlp exits successfully
        // This ensures completion is reported even if progress updates were not received
        println!("[yt-dlp] Sending final 100% completion progress");
        progress_callback(DownloadProgress {
            percentage: 100.0,
            downloaded_bytes: 0,
            total_bytes: 0,
            speed: 0.0,
            eta: 0,
        });
        
        println!("[yt-dlp] Final status: SUCCESS");
        println!("[yt-dlp] Output file: {}", save_path.display());
        
        Ok(())
    }
    
    /// Build format string for yt-dlp based on download options
    fn build_format_string(&self, options: &DownloadOptions) -> String {
        if options.audio_only {
            // Best audio quality
            return "bestaudio".to_string();
        }
        
        // Parse quality preference
        let quality = &options.quality;
        let format = &options.format;
        
        match quality.as_str() {
            "best" => format!("bestvideo[ext={}]+bestaudio/best[ext={}]/best", format, format),
            "2160p" | "4k" => format!("bestvideo[height<=2160][ext={}]+bestaudio/best[height<=2160]/best", format),
            "1440p" => format!("bestvideo[height<=1440][ext={}]+bestaudio/best[height<=1440]/best", format),
            "1080p" => format!("bestvideo[height<=1080][ext={}]+bestaudio/best[height<=1080]/best", format),
            "720p" => format!("bestvideo[height<=720][ext={}]+bestaudio/best[height<=720]/best", format),
            "480p" => format!("bestvideo[height<=480][ext={}]+bestaudio/best[height<=480]/best", format),
            "360p" => format!("bestvideo[height<=360][ext={}]+bestaudio/best[height<=360]/best", format),
            _ => format!("bestvideo[ext={}]+bestaudio/best[ext={}]/best", format, format),
        }
    }
    
    /// Parse progress information from yt-dlp output line
    fn parse_progress_line(&self, line: &str) -> Option<DownloadProgress> {
        // Only process lines that contain [download]
        if !line.contains("[download]") {
            return None;
        }
        
        // Pattern 1: [download] Destination: filename.mp4 (indicates download start - 0% progress)
        if line.contains("[download] Destination:") {
            return Some(DownloadProgress {
                percentage: 0.0,
                downloaded_bytes: 0,
                total_bytes: 0,
                speed: 0.0,
                eta: 0,
            });
        }
        
        // Pattern 2: [download] has already been downloaded (indicates 100% - file exists)
        if line.contains("has already been downloaded") {
            return Some(DownloadProgress {
                percentage: 100.0,
                downloaded_bytes: 0,
                total_bytes: 0,
                speed: 0.0,
                eta: 0,
            });
        }
        
        // Pattern 3: [download] 100% of X.XXMiB (completion line)
        if line.contains("[download] 100%") || line.contains("[download]  100%") {
            return Some(DownloadProgress {
                percentage: 100.0,
                downloaded_bytes: 0,
                total_bytes: 0,
                speed: 0.0,
                eta: 0,
            });
        }
        
        // Pattern 4: Standard format - [download]  45.8% of 123.45MiB at 1.23MiB/s ETA 00:42
        // Try to extract percentage first - if this fails, the line is unparseable
        println!("[yt-dlp] Parsing progress line: {}", line);
        match self.extract_percentage(line) {
            Some(percentage) => {
                // Extract downloaded and total bytes
                let (downloaded_bytes, total_bytes) = self.extract_bytes(line).unwrap_or((0, 0));
                
                // Extract speed (bytes per second)
                let speed = self.extract_speed(line).unwrap_or(0.0);
                
                // Extract ETA (seconds)
                let eta = self.extract_eta(line).unwrap_or(0);
                
                Some(DownloadProgress {
                    percentage,
                    downloaded_bytes,
                    total_bytes,
                    speed,
                    eta,
                })
            }
            None => {
                // Log unparseable lines that contain "[download]" for debugging
                println!("[yt-dlp] ⚠ Unparseable download line: {}", line);
                None
            }
        }
    }
    
    /// Extract percentage from progress line
    fn extract_percentage(&self, line: &str) -> Option<f64> {
        // Wrap regex operations in error handling
        match Regex::new(r"(\d+\.?\d*)%") {
            Ok(re) => {
                match re.captures(line) {
                    Some(caps) => {
                        match caps.get(1) {
                            Some(m) => m.as_str().parse().ok(),
                            None => None,
                        }
                    }
                    None => None,
                }
            }
            Err(_) => None,
        }
    }
    
    /// Extract downloaded and total bytes from progress line
    fn extract_bytes(&self, line: &str) -> Option<(u64, u64)> {
        // Pattern: "45.8% of 123.45MiB" or "45.8% of ~123.45MiB"
        // Wrap all regex operations in error handling
        let re = match Regex::new(r"(\d+\.?\d*)\s*%\s+of\s+~?(\d+\.?\d*)(KiB|MiB|GiB|B)") {
            Ok(r) => r,
            Err(_) => return None,
        };
        
        let caps = match re.captures(line) {
            Some(c) => c,
            None => return None,
        };
        
        let percentage: f64 = match caps.get(1) {
            Some(m) => match m.as_str().parse() {
                Ok(p) => p,
                Err(_) => return None,
            },
            None => return None,
        };
        
        let total_value: f64 = match caps.get(2) {
            Some(m) => match m.as_str().parse() {
                Ok(v) => v,
                Err(_) => return None,
            },
            None => return None,
        };
        
        let unit = match caps.get(3) {
            Some(m) => m.as_str(),
            None => return None,
        };
        
        // Convert to bytes
        let multiplier = match unit {
            "B" => 1.0,
            "KiB" => 1024.0,
            "MiB" => 1024.0 * 1024.0,
            "GiB" => 1024.0 * 1024.0 * 1024.0,
            _ => 1.0,
        };
        
        let total_bytes = (total_value * multiplier) as u64;
        let downloaded_bytes = ((percentage / 100.0) * total_bytes as f64) as u64;
        
        Some((downloaded_bytes, total_bytes))
    }
    
    /// Extract download speed from progress line (returns bytes per second)
    fn extract_speed(&self, line: &str) -> Option<f64> {
        // Pattern: "at 1.23MiB/s" or "at 123.45KiB/s"
        // Wrap all regex operations in error handling
        let re = match Regex::new(r"at\s+(\d+\.?\d*)(KiB|MiB|GiB|B)/s") {
            Ok(r) => r,
            Err(_) => return None,
        };
        
        let caps = match re.captures(line) {
            Some(c) => c,
            None => return None,
        };
        
        let speed_value: f64 = match caps.get(1) {
            Some(m) => match m.as_str().parse() {
                Ok(v) => v,
                Err(_) => return None,
            },
            None => return None,
        };
        
        let unit = match caps.get(2) {
            Some(m) => m.as_str(),
            None => return None,
        };
        
        // Convert to bytes per second
        let multiplier = match unit {
            "B" => 1.0,
            "KiB" => 1024.0,
            "MiB" => 1024.0 * 1024.0,
            "GiB" => 1024.0 * 1024.0 * 1024.0,
            _ => 1.0,
        };
        
        Some(speed_value * multiplier)
    }
    
    /// Extract ETA from progress line (returns seconds)
    fn extract_eta(&self, line: &str) -> Option<u64> {
        // Pattern: "ETA 00:42" or "ETA 01:23:45"
        // Wrap all regex operations in error handling
        let re = match Regex::new(r"ETA\s+(\d+):(\d+)(?::(\d+))?") {
            Ok(r) => r,
            Err(_) => return None,
        };
        
        let caps = match re.captures(line) {
            Some(c) => c,
            None => return None,
        };
        
        let hours: u64 = if caps.get(3).is_some() {
            // Format is HH:MM:SS
            match caps.get(1) {
                Some(m) => match m.as_str().parse() {
                    Ok(h) => h,
                    Err(_) => return None,
                },
                None => return None,
            }
        } else {
            0
        };
        
        let minutes: u64 = if caps.get(3).is_some() {
            match caps.get(2) {
                Some(m) => match m.as_str().parse() {
                    Ok(min) => min,
                    Err(_) => return None,
                },
                None => return None,
            }
        } else {
            match caps.get(1) {
                Some(m) => match m.as_str().parse() {
                    Ok(min) => min,
                    Err(_) => return None,
                },
                None => return None,
            }
        };
        
        let seconds: u64 = if let Some(s) = caps.get(3) {
            match s.as_str().parse() {
                Ok(sec) => sec,
                Err(_) => return None,
            }
        } else {
            match caps.get(2) {
                Some(m) => match m.as_str().parse() {
                    Ok(sec) => sec,
                    Err(_) => return None,
                },
                None => return None,
            }
        };
        
        Some(hours * 3600 + minutes * 60 + seconds)
    }
    
    /// Download video with cancellation support (public method for download manager)
    pub async fn download_with_cancellation(
        &self,
        url: &str,
        options: DownloadOptions,
        save_path: &Path,
        progress_callback: Box<dyn Fn(DownloadProgress) + Send>,
        cancel_token: CancellationToken,
    ) -> Result<()> {
        self.download_video_impl(url, options, save_path, progress_callback, Some(cancel_token)).await
    }
    
    /// Test yt-dlp installation by fetching video title
    /// This is a lightweight test that verifies yt-dlp can communicate with YouTube
    pub async fn test_download(&self, url: &str) -> Result<String> {
        println!("[yt-dlp test] Testing yt-dlp with URL: {}", url);
        println!("[yt-dlp test] yt-dlp path: {:?}", self.ytdlp_path);
        
        // Check if yt-dlp executable exists
        if !self.ytdlp_path.exists() {
            let error_msg = format!("yt-dlp executable not found at: {:?}", self.ytdlp_path);
            println!("[yt-dlp test] ERROR: {}", error_msg);
            return Err(DownloadError::YtdlpNotFound);
        }
        
        // Try to fetch video title using yt-dlp
        let output = Command::new(&self.ytdlp_path)
            .args(&[
                "--no-warnings",
                "--print", "title",
                url,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                let error_msg = format!("Failed to execute yt-dlp: {}", e);
                println!("[yt-dlp test] ERROR: {}", error_msg);
                if e.kind() == std::io::ErrorKind::NotFound {
                    DownloadError::YtdlpNotFound
                } else {
                    DownloadError::DownloadFailed(error_msg)
                }
            })?;
        
        if output.status.success() {
            let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("[yt-dlp test] ✓ SUCCESS: Retrieved video title: {}", title);
            Ok(title)
        } else {
            let error = String::from_utf8_lossy(&output.stderr).to_string();
            println!("[yt-dlp test] ✗ FAILED: {}", error);
            
            // Provide clear error messages based on common issues
            if error.contains("Video unavailable") || error.contains("Private video") {
                Err(DownloadError::VideoUnavailable(
                    "Test failed: Video is unavailable or private. Try a different URL.".to_string()
                ))
            } else if error.contains("network") || error.contains("timeout") || error.contains("Unable to download") {
                Err(DownloadError::Network(
                    "Test failed: Network error. Check your internet connection.".to_string()
                ))
            } else if error.is_empty() {
                Err(DownloadError::DownloadFailed(
                    "yt-dlp test failed with no error output. yt-dlp may not be working correctly.".to_string()
                ))
            } else {
                Err(DownloadError::DownloadFailed(format!(
                    "yt-dlp test failed: {}",
                    error
                )))
            }
        }
    }
    
    /// Get yt-dlp version
    pub async fn get_ytdlp_version(&self) -> Result<String> {
        if !self.ytdlp_path.exists() {
            return Err(DownloadError::YtdlpNotFound);
        }
        
        let output = Command::new(&self.ytdlp_path)
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    DownloadError::YtdlpNotFound
                } else {
                    DownloadError::DownloadFailed(format!("Failed to get yt-dlp version: {}", e))
                }
            })?;
        
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(version)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(DownloadError::DownloadFailed(format!("Failed to get yt-dlp version: {}", error)))
        }
    }
    
    /// Get ffmpeg version
    pub async fn get_ffmpeg_version(&self) -> Result<String> {
        if !self.ffmpeg_path.exists() {
            return Err(DownloadError::DownloadFailed(
                format!("ffmpeg not found at: {:?}", self.ffmpeg_path)
            ));
        }
        
        let output = Command::new(&self.ffmpeg_path)
            .arg("-version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                DownloadError::DownloadFailed(format!("Failed to get ffmpeg version: {}", e))
            })?;
        
        if output.status.success() {
            let version_output = String::from_utf8_lossy(&output.stdout);
            // Extract version from first line: "ffmpeg version X.X.X ..."
            let version = version_output
                .lines()
                .next()
                .and_then(|line| {
                    // Try to extract version number after "ffmpeg version "
                    if let Some(start) = line.find("version ") {
                        let version_part = &line[start + 8..];
                        // Take until first space
                        version_part.split_whitespace().next().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "unknown".to_string());
            Ok(version)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(DownloadError::DownloadFailed(format!("Failed to get ffmpeg version: {}", error)))
        }
    }
    
    /// Log versions of yt-dlp and ffmpeg at startup
    pub async fn log_versions(&self) {
        println!("[YouTubeProvider] Logging dependency versions...");
        
        match self.get_ytdlp_version().await {
            Ok(version) => println!("[YouTubeProvider] yt-dlp version: {}", version),
            Err(e) => println!("[YouTubeProvider] Failed to get yt-dlp version: {:?}", e),
        }
        
        match self.get_ffmpeg_version().await {
            Ok(version) => println!("[YouTubeProvider] ffmpeg version: {}", version),
            Err(e) => println!("[YouTubeProvider] Failed to get ffmpeg version: {:?}", e),
        }
    }
}

#[async_trait]
impl PlatformProvider for YouTubeProvider {
    fn name(&self) -> &str {
        "YouTube"
    }
    
    fn matches_url(&self, url: &str) -> bool {
        // Trim whitespace and convert to lowercase for comparison
        let url = url.trim();
        
        // Check against all compiled regex patterns
        self.url_patterns.iter().any(|pattern| pattern.is_match(url))
    }
    
    fn supported_patterns(&self) -> Vec<String> {
        vec![
            "https://www.youtube.com/watch?v=VIDEO_ID".to_string(),
            "https://youtu.be/VIDEO_ID".to_string(),
            "https://www.youtube.com/playlist?list=PLAYLIST_ID".to_string(),
            "https://www.youtube.com/@CHANNEL_NAME".to_string(),
            "https://www.youtube.com/channel/CHANNEL_ID".to_string(),
            "https://www.youtube.com/user/USERNAME".to_string(),
            "https://www.youtube.com/c/CUSTOM_NAME".to_string(),
        ]
    }
    
    async fn get_video_info(&self, url: &str) -> Result<VideoInfo> {
        // Use yt-dlp to extract video information in JSON format
        let json_output = self.execute_ytdlp(&[
            "--dump-json",
            "--no-playlist",
            "--skip-download",
            url,
        ]).await?;
        
        let json: Value = serde_json::from_str(&json_output)
            .map_err(|e| DownloadError::DownloadFailed(format!("Failed to parse video info: {}", e)))?;
        
        self.parse_video_info(&json, url)
    }
    
    async fn get_playlist_info(&self, url: &str) -> Result<PlaylistInfo> {
        // First, get playlist metadata
        let json_output = self.execute_ytdlp(&[
            "--dump-json",
            "--flat-playlist",
            "--skip-download",
            url,
        ]).await?;
        
        // Parse each line as a separate JSON object (one per video)
        let mut videos = Vec::new();
        let mut playlist_title = String::new();
        let mut playlist_id = String::new();
        let mut playlist_description = String::new();
        let mut uploader = String::new();
        
        for line in json_output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            let json: Value = serde_json::from_str(line)
                .map_err(|e| DownloadError::DownloadFailed(format!("Failed to parse playlist entry: {}", e)))?;
            
            // Extract playlist metadata from first entry
            if playlist_title.is_empty() {
                playlist_title = json["playlist_title"]
                    .as_str()
                    .or_else(|| json["playlist"].as_str())
                    .unwrap_or("Unknown Playlist")
                    .to_string();
                
                playlist_id = json["playlist_id"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                
                uploader = json["playlist_uploader"]
                    .as_str()
                    .or_else(|| json["uploader"].as_str())
                    .or_else(|| json["channel"].as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                
                playlist_description = json["playlist_description"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
            }
            
            // Parse video entry
            if let Some(video_id) = json["id"].as_str() {
                let video_url = format!("https://www.youtube.com/watch?v={}", video_id);
                videos.push(VideoInfo {
                    id: video_id.to_string(),
                    title: json["title"]
                        .as_str()
                        .unwrap_or("Unknown Title")
                        .to_string(),
                    description: json["description"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    thumbnail: json["thumbnail"]
                        .as_str()
                        .or_else(|| json["thumbnails"].as_array()
                            .and_then(|arr| arr.last())
                            .and_then(|t| t["url"].as_str()))
                        .unwrap_or("")
                        .to_string(),
                    duration: json["duration"]
                        .as_u64()
                        .unwrap_or(0),
                    uploader: json["uploader"]
                        .as_str()
                        .or_else(|| json["channel"].as_str())
                        .unwrap_or(&uploader)
                        .to_string(),
                    upload_date: json["upload_date"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    view_count: json["view_count"]
                        .as_u64()
                        .unwrap_or(0),
                    available_formats: Vec::new(), // Formats not available in flat playlist
                    platform: "YouTube".to_string(),
                    url: video_url,
                });
            }
        }
        
        Ok(PlaylistInfo {
            id: playlist_id,
            title: playlist_title,
            description: playlist_description,
            uploader,
            video_count: videos.len(),
            videos,
            platform: "YouTube".to_string(),
            url: url.to_string(),
            has_more: false,
            page: 0,
            page_size: 0,
        })
    }
    
    async fn get_channel_info(&self, url: &str) -> Result<ChannelInfo> {
        // First, get channel metadata
        let json_output = self.execute_ytdlp(&[
            "--dump-json",
            "--flat-playlist",
            "--skip-download",
            url,
        ]).await?;
        
        let mut channel_name = String::new();
        let mut channel_id = String::new();
        let mut channel_description = String::new();
        let mut all_videos = Vec::new();
        
        // Parse channel videos
        for line in json_output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            let json: Value = serde_json::from_str(line)
                .map_err(|e| DownloadError::DownloadFailed(format!("Failed to parse channel entry: {}", e)))?;
            
            // Extract channel metadata from first entry
            if channel_name.is_empty() {
                channel_name = json["channel"]
                    .as_str()
                    .or_else(|| json["uploader"].as_str())
                    .unwrap_or("Unknown Channel")
                    .to_string();
                
                channel_id = json["channel_id"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                
                channel_description = json["description"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
            }
            
            // Parse video entry
            if let Some(video_id) = json["id"].as_str() {
                let video_url = format!("https://www.youtube.com/watch?v={}", video_id);
                all_videos.push(VideoInfo {
                    id: video_id.to_string(),
                    title: json["title"]
                        .as_str()
                        .unwrap_or("Unknown Title")
                        .to_string(),
                    description: json["description"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    thumbnail: json["thumbnail"]
                        .as_str()
                        .or_else(|| json["thumbnails"].as_array()
                            .and_then(|arr| arr.last())
                            .and_then(|t| t["url"].as_str()))
                        .unwrap_or("")
                        .to_string(),
                    duration: json["duration"]
                        .as_u64()
                        .unwrap_or(0),
                    uploader: channel_name.clone(),
                    upload_date: json["upload_date"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    view_count: json["view_count"]
                        .as_u64()
                        .unwrap_or(0),
                    available_formats: Vec::new(),
                    platform: "YouTube".to_string(),
                    url: video_url,
                });
            }
        }
        
        // Try to get channel playlists
        let mut playlists = Vec::new();
        
        // Attempt to get playlists tab (this may not always work)
        let playlists_url = if url.contains("/@") {
            format!("{}/playlists", url.trim_end_matches('/'))
        } else if url.contains("/channel/") {
            format!("{}/playlists", url.trim_end_matches('/'))
        } else {
            url.to_string()
        };
        
        // Try to fetch playlists (may fail if channel has no playlists tab)
        if let Ok(playlists_output) = self.execute_ytdlp(&[
            "--dump-json",
            "--flat-playlist",
            "--skip-download",
            &playlists_url,
        ]).await {
            let mut current_playlist: Option<PlaylistInfo> = None;
            let mut playlist_videos: Vec<VideoInfo> = Vec::new();
            
            for line in playlists_output.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                
                if let Ok(json) = serde_json::from_str::<Value>(line) {
                    // Check if this is a playlist entry
                    if let Some(playlist_id) = json["playlist_id"].as_str() {
                        // Save previous playlist if exists
                        if let Some(mut playlist) = current_playlist.take() {
                            let video_count = playlist_videos.len();
                            playlist.videos = playlist_videos.clone();
                            playlist.video_count = video_count;
                            playlists.push(playlist);
                            playlist_videos.clear();
                        }
                        
                        // Start new playlist
                        current_playlist = Some(PlaylistInfo {
                            id: playlist_id.to_string(),
                            title: json["playlist_title"]
                                .as_str()
                                .or_else(|| json["playlist"].as_str())
                                .unwrap_or("Unknown Playlist")
                                .to_string(),
                            description: json["playlist_description"]
                                .as_str()
                                .unwrap_or("")
                                .to_string(),
                            uploader: channel_name.clone(),
                            video_count: 0,
                            videos: Vec::new(),
                            platform: "YouTube".to_string(),
                            url: format!("https://www.youtube.com/playlist?list={}", playlist_id),
                            has_more: false,
                            page: 0,
                            page_size: 0,
                        });
                    }
                    
                    // Add video to current playlist
                    if let Some(video_id) = json["id"].as_str() {
                        let video_url = format!("https://www.youtube.com/watch?v={}", video_id);
                        playlist_videos.push(VideoInfo {
                            id: video_id.to_string(),
                            title: json["title"]
                                .as_str()
                                .unwrap_or("Unknown Title")
                                .to_string(),
                            description: String::new(),
                            thumbnail: json["thumbnail"]
                                .as_str()
                                .unwrap_or("")
                                .to_string(),
                            duration: json["duration"]
                                .as_u64()
                                .unwrap_or(0),
                            uploader: channel_name.clone(),
                            upload_date: String::new(),
                            view_count: 0,
                            available_formats: Vec::new(),
                            platform: "YouTube".to_string(),
                            url: video_url,
                        });
                    }
                }
            }
            
            // Save last playlist
            if let Some(mut playlist) = current_playlist {
                let video_count = playlist_videos.len();
                playlist.videos = playlist_videos;
                playlist.video_count = video_count;
                playlists.push(playlist);
            }
        }
        
        Ok(ChannelInfo {
            id: channel_id,
            name: channel_name,
            description: channel_description,
            playlists,
            all_videos,
            platform: "YouTube".to_string(),
            url: url.to_string(),
        })
    }
    
    async fn download_video(
        &self,
        url: &str,
        options: DownloadOptions,
        save_path: &Path,
        progress_callback: Box<dyn Fn(DownloadProgress) + Send>,
    ) -> Result<()> {
        self.download_video_impl(url, options, save_path, progress_callback, None).await
    }
    
    async fn check_dependencies(&self) -> Result<Vec<Dependency>> {
        let mut dependencies = Vec::new();
        
        // Check bundled yt-dlp
        let ytdlp_installed = self.ytdlp_path.exists();
        let ytdlp_version = if ytdlp_installed {
            match self.execute_ytdlp(&["--version"]).await {
                Ok(version) => Some(version.trim().to_string()),
                Err(_) => None,
            }
        } else {
            None
        };
        
        dependencies.push(Dependency {
            name: "yt-dlp (bundled)".to_string(),
            installed: ytdlp_installed,
            version: ytdlp_version,
            install_instructions: "yt-dlp is bundled with the application. If missing, please reinstall the application.".to_string(),
        });
        
        // Check bundled ffmpeg
        let ffmpeg_installed = self.ffmpeg_path.exists();
        let ffmpeg_version = if ffmpeg_installed {
            match Command::new(&self.ffmpeg_path)
                .arg("-version")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
            {
                Ok(output) if output.status.success() => {
                    let version_output = String::from_utf8_lossy(&output.stdout);
                    version_output
                        .lines()
                        .next()
                        .and_then(|line| line.split_whitespace().nth(2))
                        .map(|v| v.to_string())
                }
                _ => None,
            }
        } else {
            None
        };
        
        dependencies.push(Dependency {
            name: "ffmpeg (bundled)".to_string(),
            installed: ffmpeg_installed,
            version: ffmpeg_version,
            install_instructions: "ffmpeg is bundled with the application. If missing, please reinstall the application.".to_string(),
        });
        
        Ok(dependencies)
    }
    
    fn get_platform_settings(&self) -> Vec<PlatformSetting> {
        vec![
            PlatformSetting {
                key: "youtube_prefer_av1".to_string(),
                label: "优先使用 AV1 编码".to_string(),
                setting_type: SettingType::Boolean,
                default_value: serde_json::json!(false),
            },
            PlatformSetting {
                key: "youtube_skip_ads".to_string(),
                label: "跳过赞助片段 (SponsorBlock)".to_string(),
                setting_type: SettingType::Boolean,
                default_value: serde_json::json!(true),
            },
            PlatformSetting {
                key: "youtube_subtitle_language".to_string(),
                label: "字幕语言".to_string(),
                setting_type: SettingType::Select {
                    options: vec![
                        "none".to_string(),
                        "zh-CN".to_string(),
                        "zh-TW".to_string(),
                        "en".to_string(),
                        "ja".to_string(),
                        "ko".to_string(),
                    ],
                },
                default_value: serde_json::json!("none"),
            },
            PlatformSetting {
                key: "youtube_embed_thumbnail".to_string(),
                label: "嵌入缩略图到视频文件".to_string(),
                setting_type: SettingType::Boolean,
                default_value: serde_json::json!(true),
            },
            PlatformSetting {
                key: "youtube_embed_metadata".to_string(),
                label: "嵌入元数据 (标题、描述等)".to_string(),
                setting_type: SettingType::Boolean,
                default_value: serde_json::json!(true),
            },
            PlatformSetting {
                key: "youtube_max_resolution".to_string(),
                label: "最大分辨率".to_string(),
                setting_type: SettingType::Select {
                    options: vec![
                        "best".to_string(),
                        "2160p".to_string(),
                        "1440p".to_string(),
                        "1080p".to_string(),
                        "720p".to_string(),
                        "480p".to_string(),
                    ],
                },
                default_value: serde_json::json!("1080p"),
            },
        ]
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Default for YouTubeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_standard_video_url() {
        let provider = YouTubeProvider::new();
        assert!(provider.matches_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ"));
        assert!(provider.matches_url("https://youtube.com/watch?v=dQw4w9WgXcQ"));
    }

    #[test]
    fn test_matches_short_url() {
        let provider = YouTubeProvider::new();
        assert!(provider.matches_url("https://youtu.be/dQw4w9WgXcQ"));
    }

    #[test]
    fn test_matches_playlist_url() {
        let provider = YouTubeProvider::new();
        assert!(provider.matches_url("https://www.youtube.com/playlist?list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf"));
    }

    #[test]
    fn test_matches_channel_url_new_format() {
        let provider = YouTubeProvider::new();
        assert!(provider.matches_url("https://www.youtube.com/@LinusTechTips"));
    }

    #[test]
    fn test_matches_channel_url_old_format() {
        let provider = YouTubeProvider::new();
        assert!(provider.matches_url("https://www.youtube.com/channel/UCXuqSBlHAE6Xw-yeJA0Tunw"));
    }

    #[test]
    fn test_matches_user_url() {
        let provider = YouTubeProvider::new();
        assert!(provider.matches_url("https://www.youtube.com/user/LinusTechTips"));
    }

    #[test]
    fn test_matches_custom_url() {
        let provider = YouTubeProvider::new();
        assert!(provider.matches_url("https://www.youtube.com/c/LinusTechTips"));
    }

    #[test]
    fn test_does_not_match_invalid_urls() {
        let provider = YouTubeProvider::new();
        assert!(!provider.matches_url("https://www.vimeo.com/123456"));
        assert!(!provider.matches_url("https://www.bilibili.com/video/BV1xx411c7mD"));
        assert!(!provider.matches_url("not a url"));
        assert!(!provider.matches_url(""));
    }

    #[test]
    fn test_matches_url_with_whitespace() {
        let provider = YouTubeProvider::new();
        assert!(provider.matches_url("  https://www.youtube.com/watch?v=dQw4w9WgXcQ  "));
    }

    #[test]
    fn test_provider_name() {
        let provider = YouTubeProvider::new();
        assert_eq!(provider.name(), "YouTube");
    }

    #[test]
    fn test_supported_patterns() {
        let provider = YouTubeProvider::new();
        let patterns = provider.supported_patterns();
        assert_eq!(patterns.len(), 7);
        assert!(patterns.contains(&"https://www.youtube.com/watch?v=VIDEO_ID".to_string()));
    }

    #[test]
    fn test_platform_settings() {
        let provider = YouTubeProvider::new();
        let settings = provider.get_platform_settings();
        assert_eq!(settings.len(), 6);
        
        // Check that key settings exist
        assert!(settings.iter().any(|s| s.key == "youtube_prefer_av1"));
        assert!(settings.iter().any(|s| s.key == "youtube_skip_ads"));
        assert!(settings.iter().any(|s| s.key == "youtube_subtitle_language"));
        assert!(settings.iter().any(|s| s.key == "youtube_max_resolution"));
    }

    #[test]
    fn test_build_format_string_best() {
        let provider = YouTubeProvider::new();
        let options = DownloadOptions {
            quality: "best".to_string(),
            format: "mp4".to_string(),
            audio_only: false,
        };
        let format = provider.build_format_string(&options);
        assert!(format.contains("bestvideo"));
        assert!(format.contains("mp4"));
    }

    #[test]
    fn test_build_format_string_1080p() {
        let provider = YouTubeProvider::new();
        let options = DownloadOptions {
            quality: "1080p".to_string(),
            format: "mp4".to_string(),
            audio_only: false,
        };
        let format = provider.build_format_string(&options);
        assert!(format.contains("height<=1080"));
    }

    #[test]
    fn test_build_format_string_audio_only() {
        let provider = YouTubeProvider::new();
        let options = DownloadOptions {
            quality: "best".to_string(),
            format: "mp3".to_string(),
            audio_only: true,
        };
        let format = provider.build_format_string(&options);
        assert_eq!(format, "bestaudio");
    }

    #[test]
    fn test_extract_percentage() {
        let provider = YouTubeProvider::new();
        
        let line = "[download]  45.8% of 123.45MiB at 1.23MiB/s ETA 00:42";
        assert_eq!(provider.extract_percentage(line), Some(45.8));
        
        let line2 = "[download] 100.0% of 50.00MiB at 5.00MiB/s ETA 00:00";
        assert_eq!(provider.extract_percentage(line2), Some(100.0));
        
        let line3 = "[download]   0.5% of 1.00GiB at 100.00KiB/s ETA 02:30:00";
        assert_eq!(provider.extract_percentage(line3), Some(0.5));
    }

    #[test]
    fn test_extract_bytes() {
        let provider = YouTubeProvider::new();
        
        // Test MiB
        let line = "[download]  50.0% of 100.00MiB at 1.00MiB/s ETA 00:50";
        let (downloaded, total) = provider.extract_bytes(line).unwrap();
        assert_eq!(total, 100 * 1024 * 1024);
        assert_eq!(downloaded, 50 * 1024 * 1024);
        
        // Test GiB
        let line2 = "[download]  25.0% of 2.00GiB at 10.00MiB/s ETA 05:00";
        let (downloaded2, total2) = provider.extract_bytes(line2).unwrap();
        assert_eq!(total2, 2 * 1024 * 1024 * 1024);
        assert_eq!(downloaded2, (0.25 * 2.0 * 1024.0 * 1024.0 * 1024.0) as u64);
        
        // Test KiB
        let line3 = "[download]  10.0% of 500.00KiB at 50.00KiB/s ETA 00:09";
        let (downloaded3, total3) = provider.extract_bytes(line3).unwrap();
        assert_eq!(total3, 500 * 1024);
        assert_eq!(downloaded3, 50 * 1024);
    }

    #[test]
    fn test_extract_speed() {
        let provider = YouTubeProvider::new();
        
        // Test MiB/s
        let line = "[download]  50.0% of 100.00MiB at 5.50MiB/s ETA 00:09";
        let speed = provider.extract_speed(line).unwrap();
        assert_eq!(speed, 5.5 * 1024.0 * 1024.0);
        
        // Test KiB/s
        let line2 = "[download]  25.0% of 10.00MiB at 512.00KiB/s ETA 00:15";
        let speed2 = provider.extract_speed(line2).unwrap();
        assert_eq!(speed2, 512.0 * 1024.0);
        
        // Test GiB/s (unlikely but possible)
        let line3 = "[download]  75.0% of 100.00GiB at 1.00GiB/s ETA 00:25";
        let speed3 = provider.extract_speed(line3).unwrap();
        assert_eq!(speed3, 1.0 * 1024.0 * 1024.0 * 1024.0);
    }

    #[test]
    fn test_extract_eta() {
        let provider = YouTubeProvider::new();
        
        // Test MM:SS format
        let line = "[download]  50.0% of 100.00MiB at 1.00MiB/s ETA 00:50";
        assert_eq!(provider.extract_eta(line), Some(50));
        
        let line2 = "[download]  25.0% of 100.00MiB at 1.00MiB/s ETA 05:30";
        assert_eq!(provider.extract_eta(line2), Some(5 * 60 + 30));
        
        // Test HH:MM:SS format
        let line3 = "[download]   5.0% of 10.00GiB at 1.00MiB/s ETA 02:30:45";
        assert_eq!(provider.extract_eta(line3), Some(2 * 3600 + 30 * 60 + 45));
    }

    #[test]
    fn test_parse_progress_line() {
        let provider = YouTubeProvider::new();
        
        let line = "[download]  45.8% of 123.45MiB at 1.23MiB/s ETA 00:42";
        let progress = provider.parse_progress_line(line).unwrap();
        
        assert_eq!(progress.percentage, 45.8);
        assert!(progress.total_bytes > 0);
        assert!(progress.downloaded_bytes > 0);
        assert!(progress.speed > 0.0);
        assert_eq!(progress.eta, 42);
    }

    #[test]
    fn test_parse_progress_line_no_match() {
        let provider = YouTubeProvider::new();
        
        let line = "[youtube] Extracting URL: https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        assert!(provider.parse_progress_line(line).is_none());
        
        let line2 = "Some random output";
        assert!(provider.parse_progress_line(line2).is_none());
    }
}
