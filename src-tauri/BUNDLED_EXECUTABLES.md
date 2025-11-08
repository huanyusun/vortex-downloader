# Bundled Executables Implementation

This document describes the implementation of bundled executables for the YouTube Downloader application.

## Overview

The application now bundles yt-dlp and ffmpeg executables directly within the application package, eliminating the need for users to manually install these dependencies.

## Architecture

### Components

1. **ExecutableManager** (`src/executable_manager.rs`)
   - Manages bundled executable files
   - Detects system architecture (x86_64 vs aarch64)
   - Verifies executable integrity using SHA256 checksums
   - Sets proper file permissions

2. **UpdateService** (`src/update_service.rs`)
   - Checks for yt-dlp updates via GitHub API
   - Downloads and installs updates
   - Implements atomic replacement with rollback capability
   - Updates checksums file after successful update

3. **YouTubeProvider** (updated)
   - Now uses bundled executables instead of system PATH
   - Accepts custom executable paths via `with_executables()` constructor
   - Passes ffmpeg location to yt-dlp commands

### Directory Structure

```
src-tauri/resources/bin/
├── x86_64/
│   ├── yt-dlp
│   └── ffmpeg
├── aarch64/
│   ├── yt-dlp
│   └── ffmpeg
├── CHECKSUMS.txt
└── README.md
```

## Implementation Details

### 1. Executable Download and Preparation (Task 13.1)

- Downloaded yt-dlp for macOS (universal binary works for both architectures)
- Downloaded ffmpeg for both x86_64 and aarch64 from evermeet.cx
- Calculated SHA256 checksums for all executables
- Organized files into architecture-specific directories
- Created CHECKSUMS.txt for integrity verification

**Checksums:**
- x86_64/yt-dlp: `27c0dfb1d9439ba52f642ca16517d9c56408e8ac922ab594403750c74300e2bc`
- x86_64/ffmpeg: `63c7b7eb8bb473c8f24a26a0bdc481765ff9e9078ba3488c64e9faf6ccafaa04`
- aarch64/yt-dlp: `27c0dfb1d9439ba52f642ca16517d9c56408e8ac922ab594403750c74300e2bc`
- aarch64/ffmpeg: `63c7b7eb8bb473c8f24a26a0bdc481765ff9e9078ba3488c64e9faf6ccafaa04`

### 2. Tauri Resource Configuration (Task 13.2)

Updated `tauri.conf.json` to include bundled resources:

```json
"resources": [
  "resources/bin/x86_64/*",
  "resources/bin/aarch64/*",
  "resources/bin/CHECKSUMS.txt"
]
```

### 3. ExecutableManager Implementation (Task 13.3)

**Key Features:**
- Architecture detection at runtime
- Path resolution for bundled executables
- SHA256 checksum verification
- Automatic permission setting (chmod +x)
- Initialization method that verifies and prepares executables

**API:**
```rust
let exec_manager = ExecutableManager::new(package_info)?;
exec_manager.initialize()?;  // Verify and set permissions
let ytdlp_path = exec_manager.get_ytdlp_path();
let ffmpeg_path = exec_manager.get_ffmpeg_path();
```

### 4. YouTubeProvider Integration (Task 13.4)

**Changes:**
- Added `ffmpeg_path` field to YouTubeProvider
- New constructor: `with_executables(ytdlp_path, ffmpeg_path)`
- Updated download commands to use `--ffmpeg-location` flag
- Modified dependency checking to verify bundled executables
- Updated main.rs to initialize provider with bundled paths

**Initialization Flow:**
```rust
// In main.rs
let executable_manager = ExecutableManager::new(package_info)?;
executable_manager.initialize()?;

let ytdlp_path = executable_manager.get_ytdlp_path();
let ffmpeg_path = executable_manager.get_ffmpeg_path();

let provider = YouTubeProvider::with_executables(ytdlp_path, ffmpeg_path);
```

### 5. Auto-Update Service (Task 13.5)

**Features:**
- Version checking via GitHub API
- Automatic download of latest yt-dlp
- Atomic replacement with backup
- Rollback capability on failure
- Checksum file updates
- Progress events for UI feedback

**API:**
```rust
let update_service = UpdateService::new(ytdlp_path, arch);

// Check for updates
let update_info = update_service.check_for_update().await?;

// Perform update
let result = update_service.update().await?;

// Rollback if needed
update_service.rollback()?;
```

**Tauri Commands:**
- `check_ytdlp_update()` - Returns current and latest version info
- `update_ytdlp()` - Performs the update and emits progress events

## Dependencies Added

```toml
sha2 = "0.10"
reqwest = { version = "0.11", features = ["json"] }
```

## Benefits

1. **User Experience**
   - No manual installation required
   - Works out of the box
   - Automatic updates for yt-dlp
   - Consistent behavior across installations

2. **Reliability**
   - Verified executable integrity
   - Known working versions
   - Isolated from system PATH issues
   - Atomic updates with rollback

3. **Maintenance**
   - Easy to update yt-dlp
   - ffmpeg version controlled by app releases
   - Checksums ensure integrity
   - Clear error messages if executables are corrupted

## Testing

All components have been tested:
- ExecutableManager unit tests pass
- Architecture detection works correctly
- Checksum verification functions properly
- Application compiles without errors

## Future Improvements

1. Add UI for manual update checks
2. Implement automatic background update checks (weekly)
3. Add progress indicators for downloads
4. Support for custom executable paths (advanced users)
5. Implement ffmpeg updates (currently only yt-dlp)

## Migration Notes

For existing users:
- The application will now use bundled executables
- System-installed yt-dlp/ffmpeg will be ignored
- No action required from users
- Welcome wizard will be simplified (no dependency checks needed)

## Security Considerations

1. **Checksum Verification**: All executables verified on startup
2. **Atomic Updates**: Updates are atomic with rollback capability
3. **Secure Downloads**: HTTPS for all downloads
4. **Permission Management**: Proper file permissions set automatically
5. **Integrity Checks**: Failed verification prevents app startup

## Troubleshooting

If executable verification fails:
1. Check CHECKSUMS.txt exists
2. Verify executable files are present
3. Ensure proper file permissions
4. Reinstall application if corruption detected

Error messages will guide users to reinstall if executables are corrupted or missing.
