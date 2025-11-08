use youtube_downloader_gui::error::{DownloadError, ErrorType, ErrorResponse};

#[test]
fn test_error_type_network() {
    let error = DownloadError::Network("Connection failed".to_string());
    assert_eq!(error.error_type(), ErrorType::NetworkError);
}

#[test]
fn test_error_type_video_unavailable() {
    let error = DownloadError::VideoUnavailable("Video is private".to_string());
    assert_eq!(error.error_type(), ErrorType::VideoUnavailable);
}

#[test]
fn test_error_type_insufficient_space() {
    let error = DownloadError::InsufficientSpace {
        required: 1000000,
        available: 500000,
    };
    assert_eq!(error.error_type(), ErrorType::InsufficientSpace);
}

#[test]
fn test_error_type_invalid_url() {
    let error = DownloadError::InvalidUrl("Not a valid URL".to_string());
    assert_eq!(error.error_type(), ErrorType::InvalidUrl);
}

#[test]
fn test_error_type_ytdlp_not_found() {
    let error = DownloadError::YtdlpNotFound;
    assert_eq!(error.error_type(), ErrorType::YtdlpNotFound);
}

#[test]
fn test_error_type_download_failed() {
    let error = DownloadError::DownloadFailed("Unknown error".to_string());
    assert_eq!(error.error_type(), ErrorType::DownloadFailed);
}

#[test]
fn test_error_type_permission_denied() {
    let error = DownloadError::PermissionDenied("No write access".to_string());
    assert_eq!(error.error_type(), ErrorType::PermissionDenied);
}

#[test]
fn test_error_type_cancelled() {
    let error = DownloadError::Cancelled;
    assert_eq!(error.error_type(), ErrorType::Cancelled);
}

#[test]
fn test_error_type_timeout() {
    let error = DownloadError::Timeout;
    assert_eq!(error.error_type(), ErrorType::Timeout);
}

#[test]
fn test_error_is_retryable_network() {
    let error = DownloadError::Network("Connection failed".to_string());
    assert!(error.is_retryable());
}

#[test]
fn test_error_is_retryable_timeout() {
    let error = DownloadError::Timeout;
    assert!(error.is_retryable());
}

#[test]
fn test_error_is_retryable_download_failed() {
    let error = DownloadError::DownloadFailed("Unknown error".to_string());
    assert!(error.is_retryable());
}

#[test]
fn test_error_not_retryable_video_unavailable() {
    let error = DownloadError::VideoUnavailable("Video is private".to_string());
    assert!(!error.is_retryable());
}

#[test]
fn test_error_not_retryable_invalid_url() {
    let error = DownloadError::InvalidUrl("Not a valid URL".to_string());
    assert!(!error.is_retryable());
}

#[test]
fn test_error_not_retryable_ytdlp_not_found() {
    let error = DownloadError::YtdlpNotFound;
    assert!(!error.is_retryable());
}

#[test]
fn test_error_suggested_action_network() {
    let error = DownloadError::Network("Connection failed".to_string());
    let action = error.suggested_action();
    assert!(action.is_some());
    assert!(action.unwrap().contains("internet connection"));
}

#[test]
fn test_error_suggested_action_video_unavailable() {
    let error = DownloadError::VideoUnavailable("Video is private".to_string());
    let action = error.suggested_action();
    assert!(action.is_some());
    assert!(action.unwrap().contains("private"));
}

#[test]
fn test_error_suggested_action_insufficient_space() {
    let error = DownloadError::InsufficientSpace {
        required: 1000000,
        available: 500000,
    };
    let action = error.suggested_action();
    assert!(action.is_some());
    assert!(action.unwrap().contains("disk space"));
}

#[test]
fn test_error_suggested_action_invalid_url() {
    let error = DownloadError::InvalidUrl("Not a valid URL".to_string());
    let action = error.suggested_action();
    assert!(action.is_some());
    assert!(action.unwrap().contains("valid"));
}

#[test]
fn test_error_suggested_action_ytdlp_not_found() {
    let error = DownloadError::YtdlpNotFound;
    let action = error.suggested_action();
    assert!(action.is_some());
    assert!(action.unwrap().contains("brew install yt-dlp"));
}

#[test]
fn test_error_suggested_action_permission_denied() {
    let error = DownloadError::PermissionDenied("No write access".to_string());
    let action = error.suggested_action();
    assert!(action.is_some());
    assert!(action.unwrap().contains("permissions"));
}

#[test]
fn test_error_to_response() {
    let error = DownloadError::Network("Connection failed".to_string());
    let response = error.to_response();
    
    assert_eq!(response.error_type, ErrorType::NetworkError);
    assert!(response.message.contains("Connection failed"));
    assert!(response.retryable);
    assert!(response.suggested_action.is_some());
}

#[test]
fn test_error_to_response_with_details() {
    let error = DownloadError::DownloadFailed("Unknown error".to_string());
    let response = error.to_response_with_details("Additional context".to_string());
    
    assert_eq!(response.error_type, ErrorType::DownloadFailed);
    assert!(response.message.contains("Unknown error"));
    assert_eq!(response.details, Some("Additional context".to_string()));
    assert!(response.retryable);
}

#[test]
fn test_error_display_network() {
    let error = DownloadError::Network("Connection failed".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Network error"));
    assert!(display.contains("Connection failed"));
}

#[test]
fn test_error_display_insufficient_space() {
    let error = DownloadError::InsufficientSpace {
        required: 1000000,
        available: 500000,
    };
    let display = format!("{}", error);
    assert!(display.contains("Insufficient disk space"));
    assert!(display.contains("1000000"));
    assert!(display.contains("500000"));
}

#[test]
fn test_error_from_io_error() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let download_error: DownloadError = io_error.into();
    
    match download_error {
        DownloadError::Io(_) => (),
        _ => panic!("Expected Io error variant"),
    }
}

#[test]
fn test_error_from_serde_error() {
    let json = "{ invalid json }";
    let serde_error = serde_json::from_str::<serde_json::Value>(json).unwrap_err();
    let download_error: DownloadError = serde_error.into();
    
    match download_error {
        DownloadError::Serialization(_) => (),
        _ => panic!("Expected Serialization error variant"),
    }
}

#[test]
fn test_error_response_serialization() {
    let response = ErrorResponse {
        error_type: ErrorType::NetworkError,
        message: "Test error".to_string(),
        details: Some("Test details".to_string()),
        retryable: true,
        suggested_action: Some("Test action".to_string()),
    };
    
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("NetworkError"));
    assert!(json.contains("Test error"));
    assert!(json.contains("Test details"));
    assert!(json.contains("Test action"));
}

#[test]
fn test_error_type_equality() {
    assert_eq!(ErrorType::NetworkError, ErrorType::NetworkError);
    assert_ne!(ErrorType::NetworkError, ErrorType::VideoUnavailable);
}
