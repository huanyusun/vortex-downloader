use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use tauri_plugin_store::{Store, StoreBuilder};
use tauri::Wry;
use super::settings::{AppSettings, DownloadHistory, QueueState};
use crate::error::{DownloadError, Result};

/// Storage service for file system operations and configuration
pub struct StorageService {
    app_handle: AppHandle,
    store: Arc<Mutex<Store<Wry>>>,
}

impl StorageService {
    /// Create a new StorageService instance
    pub fn new(app_handle: AppHandle) -> Result<Self> {
        // Initialize the store with a JSON file
        let store = StoreBuilder::new(app_handle.clone(), "settings.json".parse().unwrap())
            .build();
        
        Ok(Self {
            app_handle,
            store: Arc::new(Mutex::new(store)),
        })
    }
    
    /// Create directory structure for downloads
    /// Creates nested directories for channel/playlist organization
    pub async fn create_directory_structure(
        &self,
        base_path: &Path,
        channel_name: Option<&str>,
        playlist_name: Option<&str>,
    ) -> Result<PathBuf> {
        // Validate base path is safe
        self.validate_path(base_path)?;
        
        let mut path = base_path.to_path_buf();
        
        // Add channel subdirectory if provided
        if let Some(channel) = channel_name {
            let sanitized = Self::sanitize_filename(channel);
            if sanitized.is_empty() {
                return Err(DownloadError::PermissionDenied(
                    "Invalid channel name".to_string()
                ));
            }
            path.push(sanitized);
        }
        
        // Add playlist subdirectory if provided
        if let Some(playlist) = playlist_name {
            let sanitized = Self::sanitize_filename(playlist);
            if sanitized.is_empty() {
                return Err(DownloadError::PermissionDenied(
                    "Invalid playlist name".to_string()
                ));
            }
            path.push(sanitized);
        }
        
        // Create all directories in the path
        tokio::fs::create_dir_all(&path).await.map_err(|e| {
            DownloadError::PermissionDenied(format!("Failed to create directory: {}", e))
        })?;
        
        Ok(path)
    }
    
    /// Check if there's enough disk space available
    /// Returns true if sufficient space is available
    pub async fn check_disk_space(&self, path: &Path, required_bytes: u64) -> Result<bool> {
        // Get the actual path to check (resolve parent if file doesn't exist)
        let check_path = if path.exists() {
            path.to_path_buf()
        } else {
            path.parent()
                .ok_or_else(|| DownloadError::PermissionDenied("Invalid path".to_string()))?
                .to_path_buf()
        };
        
        // Use statvfs on Unix systems to get disk space info
        #[cfg(target_os = "macos")]
        {
            let _metadata = tokio::fs::metadata(&check_path).await?;
            let stat = nix::sys::statvfs::statvfs(&check_path)
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
            
            Ok(true)
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            // Fallback for non-macOS systems (shouldn't happen in this app)
            Ok(true)
        }
    }
    
    /// Validate that a path is safe to use
    /// Prevents path traversal attacks and ensures path is absolute
    pub fn validate_path(&self, path: &Path) -> Result<()> {
        // Check if path is absolute
        if !path.is_absolute() {
            return Err(DownloadError::PermissionDenied(
                "Path must be absolute".to_string()
            ));
        }
        
        // Check for path traversal attempts
        let path_str = path.to_string_lossy();
        if path_str.contains("..") {
            return Err(DownloadError::PermissionDenied(
                "Path traversal not allowed".to_string()
            ));
        }
        
        // Ensure path doesn't contain null bytes
        if path_str.contains('\0') {
            return Err(DownloadError::PermissionDenied(
                "Invalid path characters".to_string()
            ));
        }
        
        // On macOS, ensure we're not trying to write to system directories
        #[cfg(target_os = "macos")]
        {
            let restricted_prefixes = [
                "/System",
                "/Library",
                "/bin",
                "/sbin",
                "/usr",
                "/private/var",
            ];
            
            for prefix in &restricted_prefixes {
                if path_str.starts_with(prefix) {
                    return Err(DownloadError::PermissionDenied(
                        format!("Cannot write to system directory: {}", prefix)
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Save application settings to persistent storage
    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        let mut store = self.store.lock().map_err(|e| DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to lock store: {}", e)
        )))?;
        
        store.insert(
            "app_settings".to_string(),
            serde_json::to_value(settings).map_err(|e| DownloadError::Serialization(e))?
        ).map_err(|e| DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to save settings: {}", e)
        )))?;
        
        store.save().map_err(|e| DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to persist settings: {}", e)
        )))?;
        
        Ok(())
    }
    
    /// Load application settings from persistent storage
    pub fn load_settings(&self) -> Result<AppSettings> {
        let store = self.store.lock().map_err(|e| DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to lock store: {}", e)
        )))?;
        
        match store.get("app_settings") {
            Some(value) => {
                serde_json::from_value(value.clone())
                    .map_err(|e| DownloadError::Serialization(e))
            }
            None => {
                // Return default settings if none exist
                drop(store); // Release lock before recursive call
                let default_settings = AppSettings::default();
                // Save the defaults for next time
                self.save_settings(&default_settings)?;
                Ok(default_settings)
            }
        }
    }
    
    /// Save platform-specific settings
    pub fn save_platform_settings(
        &self,
        platform: &str,
        settings: &std::collections::HashMap<String, serde_json::Value>
    ) -> Result<()> {
        let mut store = self.store.lock().map_err(|e| DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to lock store: {}", e)
        )))?;
        
        let key = format!("platform_settings_{}", platform);
        store.insert(
            key,
            serde_json::to_value(settings).map_err(|e| DownloadError::Serialization(e))?
        ).map_err(|e| DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to save platform settings: {}", e)
        )))?;
        
        store.save().map_err(|e| DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to persist platform settings: {}", e)
        )))?;
        
        Ok(())
    }
    
    /// Load platform-specific settings
    pub fn load_platform_settings(
        &self,
        platform: &str
    ) -> Result<std::collections::HashMap<String, serde_json::Value>> {
        let store = self.store.lock().map_err(|e| DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to lock store: {}", e)
        )))?;
        
        let key = format!("platform_settings_{}", platform);
        match store.get(&key) {
            Some(value) => {
                serde_json::from_value(value.clone())
                    .map_err(|e| DownloadError::Serialization(e))
            }
            None => Ok(std::collections::HashMap::new())
        }
    }
    
    /// Save download history
    pub fn save_download_history(&self, history: &DownloadHistory) -> Result<()> {
        let mut store = self.store.lock().map_err(|e| DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to lock store: {}", e)
        )))?;
        
        store.insert(
            "download_history".to_string(),
            serde_json::to_value(history).map_err(|e| DownloadError::Serialization(e))?
        ).map_err(|e| DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to save download history: {}", e)
        )))?;
        
        store.save().map_err(|e| DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to persist download history: {}", e)
        )))?;
        
        Ok(())
    }
    
    /// Load download history
    pub fn load_download_history(&self) -> Result<DownloadHistory> {
        let store = self.store.lock().map_err(|e| DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to lock store: {}", e)
        )))?;
        
        match store.get("download_history") {
            Some(value) => {
                serde_json::from_value(value.clone())
                    .map_err(|e| DownloadError::Serialization(e))
            }
            None => Ok(DownloadHistory::default())
        }
    }
    
    /// Add a completed download to history
    pub fn add_to_history(&self, download: crate::storage::settings::CompletedDownload) -> Result<()> {
        let mut history = self.load_download_history()?;
        history.downloads.push(download);
        
        // Keep only the last 1000 downloads to prevent unbounded growth
        if history.downloads.len() > 1000 {
            history.downloads.drain(0..history.downloads.len() - 1000);
        }
        
        self.save_download_history(&history)
    }
    
    /// Save queue state
    pub fn save_queue_state(&self, queue: &QueueState) -> Result<()> {
        let mut store = self.store.lock().map_err(|e| DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to lock store: {}", e)
        )))?;
        
        store.insert(
            "queue_state".to_string(),
            serde_json::to_value(queue).map_err(|e| DownloadError::Serialization(e))?
        ).map_err(|e| DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to save queue state: {}", e)
        )))?;
        
        store.save().map_err(|e| DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to persist queue state: {}", e)
        )))?;
        
        Ok(())
    }
    
    /// Load queue state
    pub fn load_queue_state(&self) -> Result<QueueState> {
        let store = self.store.lock().map_err(|e| DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to lock store: {}", e)
        )))?;
        
        match store.get("queue_state") {
            Some(value) => {
                serde_json::from_value(value.clone())
                    .map_err(|e| DownloadError::Serialization(e))
            }
            None => Ok(QueueState::default())
        }
    }
    
    /// Clear queue state
    pub fn clear_queue_state(&self) -> Result<()> {
        let mut store = self.store.lock().map_err(|e| DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to lock store: {}", e)
        )))?;
        
        store.delete("queue_state").map_err(|e| DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to clear queue state: {}", e)
        )))?;
        
        store.save().map_err(|e| DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to persist changes: {}", e)
        )))?;
        
        Ok(())
    }
    
    /// Get default save path (user's Downloads folder)
    pub fn get_default_save_path(&self) -> PathBuf {
        // Get user's home directory
        if let Some(home) = dirs::home_dir() {
            home.join("Downloads")
        } else {
            // Fallback to current directory if home can't be determined
            PathBuf::from(".")
        }
    }
    
    /// Sanitize filename to remove invalid characters
    /// Replaces filesystem-unsafe characters with underscores
    pub fn sanitize_filename(name: &str) -> String {
        // Trim whitespace
        let trimmed = name.trim();
        
        // Replace invalid characters
        let sanitized: String = trimmed
            .chars()
            .map(|c| match c {
                // Filesystem-unsafe characters
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                // Control characters
                c if c.is_control() => '_',
                // Keep everything else
                _ => c,
            })
            .collect();
        
        // Remove leading/trailing dots and spaces (problematic on some filesystems)
        let sanitized = sanitized.trim_matches(|c| c == '.' || c == ' ');
        
        // Ensure the result is not empty
        if sanitized.is_empty() {
            "untitled".to_string()
        } else {
            sanitized.to_string()
        }
    }
}
