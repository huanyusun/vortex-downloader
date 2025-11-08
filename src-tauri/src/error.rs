use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Main error type for download operations
#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Video unavailable: {0}")]
    VideoUnavailable(String),
    
    #[error("Insufficient disk space: required {required} bytes, available {available} bytes")]
    InsufficientSpace { required: u64, available: u64 },
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    #[error("yt-dlp not found")]
    YtdlpNotFound,
    
    #[error("Download failed: {0}")]
    DownloadFailed(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),
    
    #[error("Dependency missing: {0}")]
    DependencyMissing(String),
    
    #[error("Cancelled by user")]
    Cancelled,
    
    #[error("Timeout: operation took too long")]
    Timeout,
}

/// Error type enum for categorization (serializable for frontend)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ErrorType {
    NetworkError,
    VideoUnavailable,
    InsufficientSpace,
    InvalidUrl,
    YtdlpNotFound,
    DownloadFailed,
    PermissionDenied,
    PlatformNotSupported,
    DependencyMissing,
    Cancelled,
    Timeout,
    Unknown,
}

/// Serializable error response for frontend
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorResponse {
    /// Error type for categorization
    pub error_type: ErrorType,
    /// Human-readable error message
    pub message: String,
    /// Optional detailed error information
    pub details: Option<String>,
    /// Whether the operation can be retried
    pub retryable: bool,
    /// Suggested action for the user
    pub suggested_action: Option<String>,
}

impl DownloadError {
    /// Convert error to ErrorType for categorization
    pub fn error_type(&self) -> ErrorType {
        match self {
            DownloadError::Network(_) => ErrorType::NetworkError,
            DownloadError::VideoUnavailable(_) => ErrorType::VideoUnavailable,
            DownloadError::InsufficientSpace { .. } => ErrorType::InsufficientSpace,
            DownloadError::InvalidUrl(_) => ErrorType::InvalidUrl,
            DownloadError::YtdlpNotFound => ErrorType::YtdlpNotFound,
            DownloadError::DownloadFailed(_) => ErrorType::DownloadFailed,
            DownloadError::PermissionDenied(_) => ErrorType::PermissionDenied,
            DownloadError::PlatformNotSupported(_) => ErrorType::PlatformNotSupported,
            DownloadError::DependencyMissing(_) => ErrorType::DependencyMissing,
            DownloadError::Cancelled => ErrorType::Cancelled,
            DownloadError::Timeout => ErrorType::Timeout,
            DownloadError::Io(_) | DownloadError::Serialization(_) => ErrorType::Unknown,
        }
    }
    
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            DownloadError::Network(_) | DownloadError::Timeout | DownloadError::DownloadFailed(_)
        )
    }
    
    /// Get suggested action for the user
    pub fn suggested_action(&self) -> Option<String> {
        match self {
            DownloadError::Network(_) => Some("Check your internet connection and try again.".to_string()),
            DownloadError::VideoUnavailable(_) => Some("The video may be private, deleted, or region-restricted.".to_string()),
            DownloadError::InsufficientSpace { .. } => Some("Free up disk space and try again.".to_string()),
            DownloadError::InvalidUrl(_) => Some("Please enter a valid YouTube URL.".to_string()),
            DownloadError::YtdlpNotFound => Some("Install yt-dlp using: brew install yt-dlp".to_string()),
            DownloadError::PermissionDenied(_) => Some("Choose a different save location with write permissions.".to_string()),
            DownloadError::PlatformNotSupported(_) => Some("This platform is not yet supported.".to_string()),
            DownloadError::DependencyMissing(dep) => Some(format!("Install the required dependency: {}", dep)),
            DownloadError::Timeout => Some("The operation took too long. Try again later.".to_string()),
            _ => None,
        }
    }
    
    /// Convert to ErrorResponse for frontend
    pub fn to_response(&self) -> ErrorResponse {
        ErrorResponse {
            error_type: self.error_type(),
            message: self.to_string(),
            details: None,
            retryable: self.is_retryable(),
            suggested_action: self.suggested_action(),
        }
    }
    
    /// Convert to ErrorResponse with additional details
    pub fn to_response_with_details(&self, details: String) -> ErrorResponse {
        ErrorResponse {
            error_type: self.error_type(),
            message: self.to_string(),
            details: Some(details),
            retryable: self.is_retryable(),
            suggested_action: self.suggested_action(),
        }
    }
}

impl From<DownloadError> for String {
    fn from(error: DownloadError) -> Self {
        error.to_string()
    }
}

impl From<DownloadError> for ErrorResponse {
    fn from(error: DownloadError) -> Self {
        error.to_response()
    }
}

pub type Result<T> = std::result::Result<T, DownloadError>;
