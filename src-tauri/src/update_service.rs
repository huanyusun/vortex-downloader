use std::path::{Path, PathBuf};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tokio::process::Command;
use tokio::io::AsyncWriteExt;
use sha2::{Sha256, Digest};
use crate::error::{DownloadError, Result};
use crate::executable_manager::Architecture;

/// Service for managing yt-dlp updates
pub struct UpdateService {
    ytdlp_path: PathBuf,
    arch: Architecture,
}

impl UpdateService {
    /// Create a new UpdateService
    pub fn new(ytdlp_path: PathBuf, arch: Architecture) -> Self {
        Self {
            ytdlp_path,
            arch,
        }
    }
    
    /// Get the current version of yt-dlp
    pub async fn get_current_version(&self) -> Result<String> {
        let output = Command::new(&self.ytdlp_path)
            .arg("--version")
            .output()
            .await
            .map_err(|e| DownloadError::DownloadFailed(format!("Failed to get yt-dlp version: {}", e)))?;
        
        if !output.status.success() {
            return Err(DownloadError::DownloadFailed("Failed to get yt-dlp version".to_string()));
        }
        
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    }
    
    /// Get the latest version available from GitHub
    pub async fn get_latest_version(&self) -> Result<String> {
        // Use GitHub API to get the latest release
        let client = reqwest::Client::builder()
            .user_agent("YouTube-Downloader-GUI")
            .build()
            .map_err(|e| DownloadError::Network(format!("Failed to create HTTP client: {}", e)))?;
        
        let response = client
            .get("https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest")
            .send()
            .await
            .map_err(|e| DownloadError::Network(format!("Failed to fetch latest version: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(DownloadError::Network(format!("GitHub API returned status: {}", response.status())));
        }
        
        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| DownloadError::Network(format!("Failed to parse GitHub API response: {}", e)))?;
        
        let version = json["tag_name"]
            .as_str()
            .ok_or_else(|| DownloadError::DownloadFailed("No tag_name in GitHub API response".to_string()))?
            .to_string();
        
        Ok(version)
    }
    
    /// Check if an update is available
    pub async fn check_for_update(&self) -> Result<Option<String>> {
        let current = self.get_current_version().await?;
        let latest = self.get_latest_version().await?;
        
        if current != latest {
            Ok(Some(latest))
        } else {
            Ok(None)
        }
    }
    
    /// Download the latest version of yt-dlp
    async fn download_latest(&self, temp_path: &Path) -> Result<()> {
        let download_url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos";
        
        let client = reqwest::Client::builder()
            .user_agent("YouTube-Downloader-GUI")
            .build()
            .map_err(|e| DownloadError::Network(format!("Failed to create HTTP client: {}", e)))?;
        
        let response = client
            .get(download_url)
            .send()
            .await
            .map_err(|e| DownloadError::Network(format!("Failed to download yt-dlp: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(DownloadError::Network(format!("Download failed with status: {}", response.status())));
        }
        
        let bytes = response
            .bytes()
            .await
            .map_err(|e| DownloadError::Network(format!("Failed to read download: {}", e)))?;
        
        // Write to temp file
        let mut file = tokio::fs::File::create(temp_path)
            .await
            .map_err(|e| DownloadError::DownloadFailed(format!("Failed to create temp file: {}", e)))?;
        
        file.write_all(&bytes)
            .await
            .map_err(|e| DownloadError::DownloadFailed(format!("Failed to write temp file: {}", e)))?;
        
        file.flush()
            .await
            .map_err(|e| DownloadError::DownloadFailed(format!("Failed to flush temp file: {}", e)))?;
        
        Ok(())
    }
    
    /// Calculate SHA256 checksum of a file
    fn calculate_checksum(&self, path: &Path) -> Result<String> {
        let contents = fs::read(path)
            .map_err(|e| DownloadError::DownloadFailed(format!("Failed to read file for checksum: {}", e)))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&contents);
        let result = hasher.finalize();
        
        Ok(format!("{:x}", result))
    }
    
    /// Set executable permissions on a file
    fn set_executable(&self, path: &Path) -> Result<()> {
        let metadata = fs::metadata(path)
            .map_err(|e| DownloadError::DownloadFailed(format!("Failed to get metadata: {}", e)))?;
        
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);
        
        fs::set_permissions(path, permissions)
            .map_err(|e| DownloadError::DownloadFailed(format!("Failed to set permissions: {}", e)))?;
        
        Ok(())
    }
    
    /// Update yt-dlp to the latest version
    pub async fn update(&self) -> Result<String> {
        // Check if update is available
        let new_version = match self.check_for_update().await? {
            Some(version) => version,
            None => return Ok("Already up to date".to_string()),
        };
        
        // Create backup path
        let backup_path = self.ytdlp_path.with_extension("backup");
        
        // Create temp path for download
        let temp_path = self.ytdlp_path.with_extension("tmp");
        
        // Download new version
        self.download_latest(&temp_path).await?;
        
        // Set executable permissions on temp file
        self.set_executable(&temp_path)?;
        
        // Verify the downloaded file works
        let test_output = Command::new(&temp_path)
            .arg("--version")
            .output()
            .await
            .map_err(|e| {
                // Clean up temp file on error
                let _ = fs::remove_file(&temp_path);
                DownloadError::DownloadFailed(format!("Downloaded yt-dlp failed verification: {}", e))
            })?;
        
        if !test_output.status.success() {
            // Clean up temp file
            let _ = fs::remove_file(&temp_path);
            return Err(DownloadError::DownloadFailed("Downloaded yt-dlp failed to run".to_string()));
        }
        
        // Backup current version
        if self.ytdlp_path.exists() {
            fs::copy(&self.ytdlp_path, &backup_path)
                .map_err(|e| {
                    // Clean up temp file on error
                    let _ = fs::remove_file(&temp_path);
                    DownloadError::DownloadFailed(format!("Failed to create backup: {}", e))
                })?;
        }
        
        // Atomically replace old version with new version
        fs::rename(&temp_path, &self.ytdlp_path)
            .map_err(|e| {
                // Try to restore from backup
                if backup_path.exists() {
                    let _ = fs::rename(&backup_path, &self.ytdlp_path);
                }
                // Clean up temp file
                let _ = fs::remove_file(&temp_path);
                DownloadError::DownloadFailed(format!("Failed to replace yt-dlp: {}", e))
            })?;
        
        // Remove backup on success
        if backup_path.exists() {
            let _ = fs::remove_file(&backup_path);
        }
        
        // Update checksums file
        self.update_checksums_file().await?;
        
        Ok(format!("Updated to version {}", new_version))
    }
    
    /// Update the CHECKSUMS.txt file with the new yt-dlp checksum
    async fn update_checksums_file(&self) -> Result<()> {
        // Get the parent directory (resources/bin)
        let bin_dir = self.ytdlp_path
            .parent()
            .and_then(|p| p.parent())
            .ok_or_else(|| DownloadError::DownloadFailed("Invalid yt-dlp path".to_string()))?;
        
        let checksums_path = bin_dir.join("CHECKSUMS.txt");
        
        // Calculate new checksum
        let new_checksum = self.calculate_checksum(&self.ytdlp_path)?;
        
        // Read existing checksums
        let checksums_content = fs::read_to_string(&checksums_path)
            .map_err(|e| DownloadError::DownloadFailed(format!("Failed to read checksums file: {}", e)))?;
        
        // Update the appropriate line
        let arch_dir = self.arch.dir_name();
        let ytdlp_key = format!("{}/yt-dlp", arch_dir);
        
        let mut new_content = String::new();
        for line in checksums_content.lines() {
            if line.contains(&ytdlp_key) {
                // Replace with new checksum
                new_content.push_str(&format!("{}  {}\n", new_checksum, ytdlp_key));
            } else {
                new_content.push_str(line);
                new_content.push('\n');
            }
        }
        
        // Write updated checksums
        fs::write(&checksums_path, new_content)
            .map_err(|e| DownloadError::DownloadFailed(format!("Failed to write checksums file: {}", e)))?;
        
        Ok(())
    }
    
    /// Rollback to backup version if available
    pub fn rollback(&self) -> Result<()> {
        let backup_path = self.ytdlp_path.with_extension("backup");
        
        if !backup_path.exists() {
            return Err(DownloadError::DownloadFailed("No backup available".to_string()));
        }
        
        fs::rename(&backup_path, &self.ytdlp_path)
            .map_err(|e| DownloadError::DownloadFailed(format!("Failed to rollback: {}", e)))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_architecture_dir_name() {
        assert_eq!(Architecture::X86_64.dir_name(), "x86_64");
        assert_eq!(Architecture::Aarch64.dir_name(), "aarch64");
    }
}
