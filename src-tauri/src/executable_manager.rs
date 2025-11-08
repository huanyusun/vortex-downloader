use std::path::{Path, PathBuf};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use sha2::{Sha256, Digest};
use tauri::api::path::resource_dir;
use tauri::PackageInfo;
use crate::error::{DownloadError, Result};

/// Manages bundled executable files (yt-dlp and ffmpeg)
pub struct ExecutableManager {
    resource_dir: PathBuf,
    arch: Architecture,
}

/// System architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86_64,
    Aarch64,
}

impl Architecture {
    /// Detect the current system architecture
    pub fn detect() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            Architecture::X86_64
        }
        #[cfg(target_arch = "aarch64")]
        {
            Architecture::Aarch64
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            // Default to x86_64 for unsupported architectures
            Architecture::X86_64
        }
    }
    
    /// Get the directory name for this architecture
    pub fn dir_name(&self) -> &str {
        match self {
            Architecture::X86_64 => "x86_64",
            Architecture::Aarch64 => "aarch64",
        }
    }
}

impl ExecutableManager {
    /// Create a new ExecutableManager
    pub fn new(package_info: &PackageInfo) -> Result<Self> {
        let resource_dir = resource_dir(package_info, &tauri::Env::default())
            .ok_or_else(|| DownloadError::DownloadFailed("Failed to get resource directory".to_string()))?;
        
        let arch = Architecture::detect();
        
        // In development mode, the resource_dir might not include the resources yet
        // Check if the bin directory exists, if not, try the src-tauri/resources path
        let bin_dir = resource_dir.join("bin");
        
        let final_resource_dir = if !bin_dir.exists() {
            // Try to find the resources in the source tree (development mode)
            // First try: current_dir/resources (if we're already in src-tauri)
            // Second try: current_dir/src-tauri/resources (if we're in the project root)
            let current = std::env::current_dir().unwrap_or(resource_dir.clone());
            
            let dev_resources_1 = current.join("resources");
            let dev_resources_2 = current.join("src-tauri").join("resources");
            
            if dev_resources_1.join("bin").exists() {
                dev_resources_1
            } else if dev_resources_2.join("bin").exists() {
                dev_resources_2
            } else {
                resource_dir
            }
        } else {
            resource_dir
        };
        
        Ok(Self {
            resource_dir: final_resource_dir,
            arch,
        })
    }
    
    /// Get the path to the bundled yt-dlp executable
    pub fn get_ytdlp_path(&self) -> PathBuf {
        self.resource_dir
            .join("bin")
            .join(self.arch.dir_name())
            .join("yt-dlp")
    }
    
    /// Get the path to the bundled ffmpeg executable
    pub fn get_ffmpeg_path(&self) -> PathBuf {
        self.resource_dir
            .join("bin")
            .join(self.arch.dir_name())
            .join("ffmpeg")
    }
    
    /// Verify the integrity of a file using SHA256 checksum
    pub fn verify_checksum(&self, file_path: &Path, expected_checksum: &str) -> Result<bool> {
        let contents = fs::read(file_path)
            .map_err(|e| DownloadError::DownloadFailed(format!("Failed to read file for checksum: {}", e)))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&contents);
        let result = hasher.finalize();
        let actual_checksum = format!("{:x}", result);
        
        Ok(actual_checksum == expected_checksum)
    }
    
    /// Verify all bundled executables
    pub fn verify_all_executables(&self) -> Result<()> {
        // Load checksums from the bundled CHECKSUMS.txt file
        let checksums_path = self.resource_dir.join("bin").join("CHECKSUMS.txt");
        let checksums_content = fs::read_to_string(&checksums_path)
            .map_err(|e| DownloadError::DownloadFailed(format!("Failed to read checksums file: {}", e)))?;
        
        // Parse checksums
        let mut checksums = std::collections::HashMap::new();
        for line in checksums_content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 {
                checksums.insert(parts[1].to_string(), parts[0].to_string());
            }
        }
        
        // Verify yt-dlp
        let ytdlp_path = self.get_ytdlp_path();
        let ytdlp_key = format!("{}/yt-dlp", self.arch.dir_name());
        if let Some(expected_checksum) = checksums.get(&ytdlp_key) {
            if !self.verify_checksum(&ytdlp_path, expected_checksum)? {
                return Err(DownloadError::DownloadFailed(
                    format!("yt-dlp checksum verification failed for {}", self.arch.dir_name())
                ));
            }
        } else {
            return Err(DownloadError::DownloadFailed(
                format!("No checksum found for yt-dlp ({})", self.arch.dir_name())
            ));
        }
        
        // Verify ffmpeg
        let ffmpeg_path = self.get_ffmpeg_path();
        let ffmpeg_key = format!("{}/ffmpeg", self.arch.dir_name());
        if let Some(expected_checksum) = checksums.get(&ffmpeg_key) {
            if !self.verify_checksum(&ffmpeg_path, expected_checksum)? {
                return Err(DownloadError::DownloadFailed(
                    format!("ffmpeg checksum verification failed for {}", self.arch.dir_name())
                ));
            }
        } else {
            return Err(DownloadError::DownloadFailed(
                format!("No checksum found for ffmpeg ({})", self.arch.dir_name())
            ));
        }
        
        Ok(())
    }
    
    /// Ensure executable permissions are set on the bundled executables
    pub fn set_executable_permissions(&self) -> Result<()> {
        let ytdlp_path = self.get_ytdlp_path();
        let ffmpeg_path = self.get_ffmpeg_path();
        
        // Set executable permissions (0o755 = rwxr-xr-x)
        self.set_permissions(&ytdlp_path, 0o755)?;
        self.set_permissions(&ffmpeg_path, 0o755)?;
        
        Ok(())
    }
    
    /// Set file permissions (Unix-specific)
    fn set_permissions(&self, path: &Path, mode: u32) -> Result<()> {
        let metadata = fs::metadata(path)
            .map_err(|e| DownloadError::DownloadFailed(format!("Failed to get metadata: {}", e)))?;
        
        let mut permissions = metadata.permissions();
        permissions.set_mode(mode);
        
        fs::set_permissions(path, permissions)
            .map_err(|e| DownloadError::DownloadFailed(format!("Failed to set permissions: {}", e)))?;
        
        Ok(())
    }
    
    /// Initialize the executable manager (verify and set permissions)
    pub fn initialize(&self) -> Result<()> {
        // Verify checksums
        self.verify_all_executables()?;
        
        // Set executable permissions
        self.set_executable_permissions()?;
        
        Ok(())
    }
    
    /// Get the current architecture
    pub fn architecture(&self) -> Architecture {
        self.arch
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_architecture_detection() {
        let arch = Architecture::detect();
        
        #[cfg(target_arch = "x86_64")]
        assert_eq!(arch, Architecture::X86_64);
        
        #[cfg(target_arch = "aarch64")]
        assert_eq!(arch, Architecture::Aarch64);
    }
    
    #[test]
    fn test_architecture_dir_name() {
        assert_eq!(Architecture::X86_64.dir_name(), "x86_64");
        assert_eq!(Architecture::Aarch64.dir_name(), "aarch64");
    }
}
