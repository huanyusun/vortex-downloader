use crate::error::{DownloadError, Result};
use regex::Regex;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// Retry a fallible async operation with exponential backoff
pub async fn retry_with_backoff<F, Fut, T>(
    operation: F,
    config: RetryConfig,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay;
    
    loop {
        attempt += 1;
        
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt >= config.max_attempts => {
                return Err(e);
            }
            Err(e) if !e.is_retryable() => {
                return Err(e);
            }
            Err(_) => {
                // Wait before retrying
                sleep(delay).await;
                
                // Calculate next delay with exponential backoff
                delay = Duration::from_secs_f64(
                    (delay.as_secs_f64() * config.backoff_multiplier).min(config.max_delay.as_secs_f64())
                );
            }
        }
    }
}

/// URL validator for YouTube URLs
pub struct UrlValidator {
    youtube_patterns: Vec<Regex>,
}

impl UrlValidator {
    pub fn new() -> Self {
        let youtube_patterns = vec![
            Regex::new(r"^https?://(www\.)?youtube\.com/watch\?v=[\w-]+").unwrap(),
            Regex::new(r"^https?://youtu\.be/[\w-]+").unwrap(),
            Regex::new(r"^https?://(www\.)?youtube\.com/playlist\?list=[\w-]+").unwrap(),
            Regex::new(r"^https?://(www\.)?youtube\.com/@[\w-]+").unwrap(),
            Regex::new(r"^https?://(www\.)?youtube\.com/channel/[\w-]+").unwrap(),
            Regex::new(r"^https?://(www\.)?youtube\.com/user/[\w-]+").unwrap(),
            Regex::new(r"^https?://(www\.)?youtube\.com/c/[\w-]+").unwrap(),
        ];
        
        Self { youtube_patterns }
    }
    
    /// Validate a YouTube URL
    pub fn validate_youtube_url(&self, url: &str) -> Result<String> {
        let trimmed = url.trim();
        
        // Check if empty
        if trimmed.is_empty() {
            return Err(DownloadError::InvalidUrl("URL cannot be empty".to_string()));
        }
        
        // Check if it's a valid URL format
        if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
            return Err(DownloadError::InvalidUrl(
                "URL must start with http:// or https://".to_string()
            ));
        }
        
        // Check if it matches YouTube patterns
        let matches = self.youtube_patterns.iter().any(|pattern| pattern.is_match(trimmed));
        
        if !matches {
            return Err(DownloadError::InvalidUrl(
                "URL does not match any supported YouTube format".to_string()
            ));
        }
        
        Ok(trimmed.to_string())
    }
    
    /// Validate and normalize URL
    pub fn validate_and_normalize(&self, url: &str) -> Result<String> {
        let validated = self.validate_youtube_url(url)?;
        
        // Remove tracking parameters
        let cleaned = self.remove_tracking_params(&validated);
        
        Ok(cleaned)
    }
    
    /// Remove tracking parameters from URL
    fn remove_tracking_params(&self, url: &str) -> String {
        // Remove common tracking parameters
        let tracking_params = ["&feature=", "&t=", "&list=", "&index="];
        let mut cleaned = url.to_string();
        
        for param in &tracking_params {
            if let Some(pos) = cleaned.find(param) {
                // Keep only the part before the tracking parameter
                // unless it's a playlist URL and we're removing &list=
                if !(*param == "&list=" && cleaned.contains("playlist?list=")) {
                    cleaned = cleaned[..pos].to_string();
                }
            }
        }
        
        cleaned
    }
}

impl Default for UrlValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Disk space checker with pre-validation
pub struct DiskSpaceChecker;

impl DiskSpaceChecker {
    /// Check disk space before starting download
    pub async fn check_before_download(
        path: &Path,
        estimated_size: Option<u64>,
    ) -> Result<()> {
        // If we don't have an estimated size, use a conservative default (1GB)
        let required_bytes = estimated_size.unwrap_or(1024 * 1024 * 1024);
        
        #[cfg(target_os = "macos")]
        {
            use nix::sys::statvfs::statvfs;
            
            let check_path = if path.exists() {
                path
            } else if let Some(parent) = path.parent() {
                parent
            } else {
                return Err(DownloadError::PermissionDenied("Invalid path".to_string()));
            };
            
            let stat = statvfs(check_path)
                .map_err(|e| DownloadError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to get disk space: {}", e)
                )))?;
            
            let available_bytes = stat.blocks_available() as u64 * stat.block_size();
            
            // Add 10% buffer to required space
            let required_with_buffer = required_bytes + (required_bytes / 10);
            
            if available_bytes < required_with_buffer {
                return Err(DownloadError::InsufficientSpace {
                    required: required_with_buffer,
                    available: available_bytes,
                });
            }
        }
        
        Ok(())
    }
    
    /// Format bytes to human-readable string
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Generate user-friendly error messages
pub struct ErrorMessageGenerator;

impl ErrorMessageGenerator {
    /// Generate a friendly error message from a DownloadError
    pub fn generate_friendly_message(error: &DownloadError) -> String {
        match error {
            DownloadError::Network(msg) => {
                if msg.contains("timeout") {
                    "Connection timed out. Please check your internet connection.".to_string()
                } else if msg.contains("DNS") || msg.contains("resolve") {
                    "Could not resolve the server address. Check your DNS settings.".to_string()
                } else {
                    format!("Network error: {}", Self::simplify_technical_message(msg))
                }
            }
            DownloadError::VideoUnavailable(msg) => {
                if msg.contains("Private video") {
                    "This video is private and cannot be downloaded.".to_string()
                } else if msg.contains("removed") || msg.contains("deleted") {
                    "This video has been removed or deleted.".to_string()
                } else if msg.contains("region") || msg.contains("country") {
                    "This video is not available in your region.".to_string()
                } else if msg.contains("age") {
                    "This video has age restrictions.".to_string()
                } else {
                    "This video is not available for download.".to_string()
                }
            }
            DownloadError::InsufficientSpace { required, available } => {
                format!(
                    "Not enough disk space. Required: {}, Available: {}",
                    DiskSpaceChecker::format_bytes(*required),
                    DiskSpaceChecker::format_bytes(*available)
                )
            }
            DownloadError::InvalidUrl(msg) => {
                format!("Invalid URL: {}", msg)
            }
            DownloadError::YtdlpNotFound => {
                "yt-dlp is not installed. Please install it using: brew install yt-dlp".to_string()
            }
            DownloadError::DownloadFailed(msg) => {
                format!("Download failed: {}", Self::simplify_technical_message(msg))
            }
            DownloadError::PermissionDenied(msg) => {
                format!("Permission denied: {}. Please choose a different location.", msg)
            }
            DownloadError::PlatformNotSupported(platform) => {
                format!("The platform '{}' is not yet supported.", platform)
            }
            DownloadError::DependencyMissing(dep) => {
                format!("Required dependency '{}' is missing. Please install it first.", dep)
            }
            DownloadError::Cancelled => {
                "Download was cancelled.".to_string()
            }
            DownloadError::Timeout => {
                "The operation timed out. Please try again.".to_string()
            }
            DownloadError::Io(e) => {
                format!("File system error: {}", e)
            }
            DownloadError::Serialization(e) => {
                format!("Data processing error: {}", e)
            }
        }
    }
    
    /// Simplify technical error messages for end users
    fn simplify_technical_message(msg: &str) -> String {
        // Remove common technical prefixes
        let msg = msg
            .trim_start_matches("ERROR: ")
            .trim_start_matches("Error: ")
            .trim_start_matches("error: ");
        
        // Truncate very long messages
        if msg.len() > 200 {
            format!("{}...", &msg[..197])
        } else {
            msg.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_validator_valid_urls() {
        let validator = UrlValidator::new();
        
        assert!(validator.validate_youtube_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ").is_ok());
        assert!(validator.validate_youtube_url("https://youtu.be/dQw4w9WgXcQ").is_ok());
        assert!(validator.validate_youtube_url("https://www.youtube.com/playlist?list=PLtest").is_ok());
        assert!(validator.validate_youtube_url("https://www.youtube.com/@channel").is_ok());
    }

    #[test]
    fn test_url_validator_invalid_urls() {
        let validator = UrlValidator::new();
        
        assert!(validator.validate_youtube_url("").is_err());
        assert!(validator.validate_youtube_url("not a url").is_err());
        assert!(validator.validate_youtube_url("https://vimeo.com/123456").is_err());
        assert!(validator.validate_youtube_url("www.youtube.com/watch?v=test").is_err());
    }

    #[test]
    fn test_url_validator_normalize() {
        let validator = UrlValidator::new();
        
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&feature=share";
        let normalized = validator.validate_and_normalize(url).unwrap();
        assert_eq!(normalized, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(DiskSpaceChecker::format_bytes(1024), "1.00 KB");
        assert_eq!(DiskSpaceChecker::format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(DiskSpaceChecker::format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(DiskSpaceChecker::format_bytes(1536 * 1024 * 1024), "1.50 GB");
    }

    #[test]
    fn test_friendly_message_generation() {
        let error = DownloadError::YtdlpNotFound;
        let msg = ErrorMessageGenerator::generate_friendly_message(&error);
        assert!(msg.contains("yt-dlp"));
        assert!(msg.contains("brew install"));
    }

    #[test]
    fn test_friendly_message_network_timeout() {
        let error = DownloadError::Network("Connection timeout after 30s".to_string());
        let msg = ErrorMessageGenerator::generate_friendly_message(&error);
        assert!(msg.contains("timed out"));
    }

    #[test]
    fn test_friendly_message_insufficient_space() {
        let error = DownloadError::InsufficientSpace {
            required: 1024 * 1024 * 1024,
            available: 512 * 1024 * 1024,
        };
        let msg = ErrorMessageGenerator::generate_friendly_message(&error);
        assert!(msg.contains("Not enough disk space"));
        assert!(msg.contains("GB"));
    }

    #[tokio::test]
    async fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay, Duration::from_secs(1));
    }
}
