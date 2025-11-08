use tauri::{State, Manager};
use crate::AppState;
use youtube_downloader_gui::platform::{VideoInfo, PlaylistInfo, ChannelInfo, Dependency};
use youtube_downloader_gui::download::DownloadItem;
use youtube_downloader_gui::storage::AppSettings;
use youtube_downloader_gui::error::{DownloadError, ErrorResponse};
use youtube_downloader_gui::error_handler::{UrlValidator, retry_with_backoff, RetryConfig};
use youtube_downloader_gui::update_service::UpdateService;
use youtube_downloader_gui::executable_manager::ExecutableManager;

#[tauri::command]
pub async fn detect_platform(url: String, state: State<'_, AppState>) -> Result<String, ErrorResponse> {
    // Validate URL first
    let validator = UrlValidator::new();
    let validated_url = validator.validate_and_normalize(&url)
        .map_err(|e| e.to_response())?;
    
    match state.platform_registry.detect_provider(&validated_url) {
        Some(provider) => Ok(provider.name().to_string()),
        None => Err(DownloadError::PlatformNotSupported(validated_url).to_response()),
    }
}

#[tauri::command]
pub async fn get_supported_platforms(state: State<'_, AppState>) -> Result<Vec<PlatformInfo>, String> {
    let providers = state.platform_registry.get_all_providers();
    let platforms = providers
        .iter()
        .map(|p| PlatformInfo {
            name: p.name().to_string(),
            supported_patterns: p.supported_patterns(),
        })
        .collect();
    Ok(platforms)
}

#[tauri::command]
pub async fn get_video_info(url: String, state: State<'_, AppState>) -> Result<VideoInfo, ErrorResponse> {
    // Validate URL first
    let validator = UrlValidator::new();
    let validated_url = validator.validate_and_normalize(&url)
        .map_err(|e| e.to_response())?;
    
    // Verify platform is supported
    let _provider = state
        .platform_registry
        .detect_provider(&validated_url)
        .ok_or_else(|| DownloadError::PlatformNotSupported(validated_url.clone()).to_response())?;
    
    // Retry with exponential backoff for network errors
    let state_clone = state.inner().clone();
    let url_clone = validated_url.clone();
    
    retry_with_backoff(
        || async {
            let provider = state_clone
                .platform_registry
                .detect_provider(&url_clone)
                .ok_or_else(|| DownloadError::PlatformNotSupported(url_clone.clone()))?;
            provider.get_video_info(&url_clone).await
        },
        RetryConfig::default(),
    )
    .await
    .map_err(|e| e.to_response())
}

#[tauri::command]
pub async fn get_playlist_info(url: String, state: State<'_, AppState>) -> Result<PlaylistInfo, ErrorResponse> {
    // Validate URL first
    let validator = UrlValidator::new();
    let validated_url = validator.validate_and_normalize(&url)
        .map_err(|e| e.to_response())?;
    
    // Verify platform is supported
    let _provider = state
        .platform_registry
        .detect_provider(&validated_url)
        .ok_or_else(|| DownloadError::PlatformNotSupported(validated_url.clone()).to_response())?;
    
    // Retry with exponential backoff for network errors
    let state_clone = state.inner().clone();
    let url_clone = validated_url.clone();
    
    retry_with_backoff(
        || async {
            let provider = state_clone
                .platform_registry
                .detect_provider(&url_clone)
                .ok_or_else(|| DownloadError::PlatformNotSupported(url_clone.clone()))?;
            provider.get_playlist_info(&url_clone).await
        },
        RetryConfig::default(),
    )
    .await
    .map_err(|e| e.to_response())
}

#[tauri::command]
pub async fn get_channel_info(url: String, state: State<'_, AppState>) -> Result<ChannelInfo, ErrorResponse> {
    // Validate URL first
    let validator = UrlValidator::new();
    let validated_url = validator.validate_and_normalize(&url)
        .map_err(|e| e.to_response())?;
    
    // Verify platform is supported
    let _provider = state
        .platform_registry
        .detect_provider(&validated_url)
        .ok_or_else(|| DownloadError::PlatformNotSupported(validated_url.clone()).to_response())?;
    
    // Retry with exponential backoff for network errors
    let state_clone = state.inner().clone();
    let url_clone = validated_url.clone();
    
    retry_with_backoff(
        || async {
            let provider = state_clone
                .platform_registry
                .detect_provider(&url_clone)
                .ok_or_else(|| DownloadError::PlatformNotSupported(url_clone.clone()))?;
            provider.get_channel_info(&url_clone).await
        },
        RetryConfig::default(),
    )
    .await
    .map_err(|e| e.to_response())
}

#[tauri::command]
pub async fn add_to_download_queue(
    items: Vec<DownloadItem>,
    state: State<'_, AppState>,
) -> Result<(), ErrorResponse> {
    println!("[add_to_download_queue] Received {} items", items.len());
    for (idx, item) in items.iter().enumerate() {
        println!("[add_to_download_queue] Item {}: id={}, title={}, status={:?}", 
                 idx, item.id, item.title, item.status);
    }
    
    state
        .download_manager
        .add_to_queue(items)
        .await
        .map_err(|e| {
            println!("[add_to_download_queue] Error: {:?}", e);
            e.to_response()
        })
}

#[tauri::command]
pub async fn pause_download(id: String, state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    state
        .download_manager
        .pause_download(&id)
        .await
        .map_err(|e| e.to_response())
}

#[tauri::command]
pub async fn resume_download(id: String, state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    state
        .download_manager
        .resume_download(&id)
        .await
        .map_err(|e| e.to_response())
}

#[tauri::command]
pub async fn cancel_download(id: String, state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    state
        .download_manager
        .cancel_download(&id)
        .await
        .map_err(|e| e.to_response())
}

#[tauri::command]
pub async fn reorder_queue(
    from_index: usize,
    to_index: usize,
    state: State<'_, AppState>,
) -> Result<(), ErrorResponse> {
    state
        .download_manager
        .reorder_queue(from_index, to_index)
        .await
        .map_err(|e| e.to_response())
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, ErrorResponse> {
    state
        .storage_service
        .load_settings()
        .map_err(|e| e.to_response())
}

#[tauri::command]
pub async fn save_settings(
    settings: AppSettings,
    state: State<'_, AppState>,
) -> Result<(), ErrorResponse> {
    state
        .storage_service
        .save_settings(&settings)
        .map_err(|e| e.to_response())
}

#[tauri::command]
pub async fn select_directory() -> Result<Option<String>, String> {
    use tauri::api::dialog::blocking::FileDialogBuilder;
    
    let path = FileDialogBuilder::new()
        .set_title("Select Download Directory")
        .pick_folder();
    
    Ok(path.map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn check_homebrew_installed() -> Result<bool, String> {
    use std::process::Command;
    
    match Command::new("which").arg("brew").output() {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
pub async fn install_ytdlp_via_homebrew(app_handle: tauri::AppHandle) -> Result<(), String> {
    use std::process::Command;
    use tauri::Manager;
    
    // Check if homebrew is installed
    let has_brew = check_homebrew_installed().await?;
    if !has_brew {
        return Err("Homebrew is not installed. Please install Homebrew first from https://brew.sh".to_string());
    }
    
    // Emit progress event
    let _ = app_handle.emit_all("install:progress", "Installing yt-dlp via Homebrew...");
    
    // Run brew install yt-dlp
    let output = Command::new("brew")
        .args(&["install", "yt-dlp"])
        .output()
        .map_err(|e| format!("Failed to execute brew command: {}", e))?;
    
    if output.status.success() {
        let _ = app_handle.emit_all("install:progress", "yt-dlp installed successfully!");
        Ok(())
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to install yt-dlp: {}", error_msg))
    }
}

#[tauri::command]
pub async fn check_dependencies(
    platform_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<Dependency>, ErrorResponse> {
    // If platform_name is provided, check dependencies for that specific platform
    if let Some(name) = platform_name {
        let provider = state
            .platform_registry
            .get_provider(&name)
            .ok_or_else(|| DownloadError::PlatformNotSupported(name.clone()).to_response())?;
        
        provider
            .check_dependencies()
            .await
            .map_err(|e| e.to_response())
    } else {
        // Check dependencies for all registered platforms
        let providers = state.platform_registry.get_all_providers();
        let mut all_dependencies = Vec::new();
        
        for provider in providers {
            match provider.check_dependencies().await {
                Ok(mut deps) => all_dependencies.append(&mut deps),
                Err(e) => {
                    // Log error but continue checking other platforms
                    eprintln!("Failed to check dependencies for {}: {}", provider.name(), e);
                }
            }
        }
        
        Ok(all_dependencies)
    }
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PlatformInfo {
    pub name: String,
    pub supported_patterns: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
}

#[tauri::command]
pub async fn verify_bundled_executables(app_handle: tauri::AppHandle) -> Result<bool, ErrorResponse> {
    let package_info = app_handle.package_info();
    let exec_manager = ExecutableManager::new(package_info)
        .map_err(|e| e.to_response())?;
    
    // Try to verify and initialize executables
    match exec_manager.initialize() {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
pub async fn check_ytdlp_update(app_handle: tauri::AppHandle) -> Result<UpdateInfo, ErrorResponse> {
    let package_info = app_handle.package_info();
    let exec_manager = ExecutableManager::new(package_info)
        .map_err(|e| e.to_response())?;
    
    let ytdlp_path = exec_manager.get_ytdlp_path();
    let arch = exec_manager.architecture();
    
    let update_service = UpdateService::new(ytdlp_path, arch);
    
    let current_version = update_service.get_current_version()
        .await
        .map_err(|e| e.to_response())?;
    
    let latest_version = update_service.check_for_update()
        .await
        .map_err(|e| e.to_response())?;
    
    let update_available = latest_version.is_some();
    
    Ok(UpdateInfo {
        current_version,
        latest_version,
        update_available,
    })
}

#[tauri::command]
pub async fn update_ytdlp(app_handle: tauri::AppHandle) -> Result<String, ErrorResponse> {
    let package_info = app_handle.package_info();
    let exec_manager = ExecutableManager::new(package_info)
        .map_err(|e| e.to_response())?;
    
    let ytdlp_path = exec_manager.get_ytdlp_path();
    let arch = exec_manager.architecture();
    
    let update_service = UpdateService::new(ytdlp_path, arch);
    
    // Emit progress event
    let _ = app_handle.emit_all("ytdlp:update:progress", "Checking for updates...");
    
    let result = update_service.update()
        .await
        .map_err(|e| e.to_response())?;
    
    // Emit completion event
    let _ = app_handle.emit_all("ytdlp:update:complete", &result);
    
    Ok(result)
}

#[derive(Serialize, Deserialize)]
pub struct DiagnosticInfo {
    pub ytdlp_version: Option<String>,
    pub ffmpeg_version: Option<String>,
    pub ytdlp_working: bool,
    pub test_result: Option<String>,
    pub test_error: Option<String>,
}

#[tauri::command]
pub async fn test_ytdlp(url: String, state: State<'_, AppState>) -> Result<DiagnosticInfo, ErrorResponse> {
    // Get the YouTube provider from the registry
    let provider = state
        .platform_registry
        .get_provider("YouTube")
        .ok_or_else(|| DownloadError::PlatformNotSupported("YouTube".to_string()).to_response())?;
    
    // Downcast to YouTubeProvider to access test methods
    let youtube_provider = provider
        .as_any()
        .downcast_ref::<youtube_downloader_gui::platform::youtube::YouTubeProvider>()
        .ok_or_else(|| DownloadError::DownloadFailed("Failed to access YouTube provider".to_string()).to_response())?;
    
    // Get versions
    let ytdlp_version = youtube_provider.get_ytdlp_version().await.ok();
    let ffmpeg_version = youtube_provider.get_ffmpeg_version().await.ok();
    
    // Test yt-dlp with the provided URL
    let (ytdlp_working, test_result, test_error) = match youtube_provider.test_download(&url).await {
        Ok(title) => (true, Some(title), None),
        Err(e) => (false, None, Some(format!("{:?}", e))),
    };
    
    Ok(DiagnosticInfo {
        ytdlp_version,
        ffmpeg_version,
        ytdlp_working,
        test_result,
        test_error,
    })
}

#[derive(Serialize, Deserialize)]
pub struct VersionInfo {
    pub ytdlp_version: Option<String>,
    pub ffmpeg_version: Option<String>,
}

#[tauri::command]
pub async fn get_dependency_versions(state: State<'_, AppState>) -> Result<VersionInfo, ErrorResponse> {
    // Get the YouTube provider from the registry
    let provider = state
        .platform_registry
        .get_provider("YouTube")
        .ok_or_else(|| DownloadError::PlatformNotSupported("YouTube".to_string()).to_response())?;
    
    // Downcast to YouTubeProvider to access version methods
    let youtube_provider = provider
        .as_any()
        .downcast_ref::<youtube_downloader_gui::platform::youtube::YouTubeProvider>()
        .ok_or_else(|| DownloadError::DownloadFailed("Failed to access YouTube provider".to_string()).to_response())?;
    
    // Get versions
    let ytdlp_version = youtube_provider.get_ytdlp_version().await.ok();
    let ffmpeg_version = youtube_provider.get_ffmpeg_version().await.ok();
    
    Ok(VersionInfo {
        ytdlp_version,
        ffmpeg_version,
    })
}
