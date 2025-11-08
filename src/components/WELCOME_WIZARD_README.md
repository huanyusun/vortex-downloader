# Welcome Wizard Implementation

## Overview
This document describes the implementation of the first-launch welcome wizard for the YouTube Downloader GUI application.

## Features Implemented

### Task 10.1: Welcome Wizard (创建欢迎向导)

#### Components Created
- **WelcomeWizard.tsx**: A multi-step wizard component that guides users through initial setup

#### Wizard Steps

1. **Welcome Screen**
   - Introduces the application
   - Shows overview of setup steps
   - Provides a "Get Started" button

2. **Dependencies Check**
   - Automatically checks for required dependencies (yt-dlp, ffmpeg)
   - Displays installation status with visual indicators (green checkmark/red X)
   - Shows version information for installed dependencies
   - Provides expandable installation instructions for missing dependencies
   - Includes "Recheck" button to verify installations
   - Blocks progression until all dependencies are installed

3. **Save Path Configuration**
   - Allows users to select a default download directory
   - Uses native file picker dialog
   - Validates that a path is selected before continuing
   - Shows selected path for confirmation

4. **Quick Tutorial**
   - Provides a 4-step guide on how to use the application
   - Covers: URL input, preview/selection, queue management, and download controls
   - Includes a pro tip about channel organization
   - "Start Using App" button to complete setup

#### Integration Points

**Frontend Changes:**
- Added `firstLaunchCompleted` field to `AppSettings` type
- Updated `App.tsx` to show wizard on first launch
- Added `checkDependencies` API function
- Created `Dependency` TypeScript type

**Backend Changes:**
- Added `first_launch_completed` field to Rust `AppSettings` struct
- Implemented `check_dependencies` command in `commands.rs`
- Added support for checking platform-specific dependencies

### Task 10.2: Automatic Dependency Installation (实现依赖自动安装)

#### Features

1. **Homebrew Detection**
   - Automatically detects if Homebrew is installed on macOS
   - Command: `check_homebrew_installed()`

2. **One-Click Installation**
   - Provides automatic yt-dlp installation via Homebrew
   - Command: `install_ytdlp_via_homebrew()`
   - Shows real-time installation progress
   - Automatically rechecks dependencies after installation

3. **Smart UI**
   - Shows "Quick Install Available" panel when Homebrew is detected
   - Shows "Homebrew Not Found" warning with installation link when not detected
   - Displays installation progress with spinner and status messages
   - Disables buttons during installation to prevent conflicts

#### Backend Commands Added

```rust
// Check if Homebrew is installed
#[tauri::command]
pub async fn check_homebrew_installed() -> Result<bool, String>

// Install yt-dlp via Homebrew
#[tauri::command]
pub async fn install_ytdlp_via_homebrew(app_handle: tauri::AppHandle) -> Result<(), String>
```

#### Events
- `install:progress`: Emitted during installation to update UI with progress messages

## User Flow

1. User launches app for the first time
2. Welcome wizard automatically appears (modal overlay)
3. User proceeds through dependency check
   - If dependencies missing and Homebrew available: one-click install option
   - If dependencies missing and no Homebrew: manual installation instructions
4. User selects download directory
5. User reviews quick tutorial
6. Setup completes, `firstLaunchCompleted` flag is set to `true`
7. User can now use the main application
8. Wizard won't show again on subsequent launches

## Technical Details

### State Management
- Wizard state managed locally within component
- Settings persisted to backend via `saveSettings` API
- First launch flag prevents wizard from showing again

### Error Handling
- Graceful fallback if dependency check fails
- Clear error messages for installation failures
- User can manually proceed or retry operations

### Styling
- Consistent with existing Tailwind CSS design system
- Dark theme matching the main application
- Responsive layout with proper spacing
- Accessible color contrast and interactive elements

## Requirements Satisfied

- ✅ Requirement 1.1: macOS compatibility
- ✅ Requirement 1.2: Native GUI with proper initialization
- ✅ Requirement 1.3: Quick startup (wizard only shows once)
- ✅ Requirement 6.2: Settings configuration (save path)
- ✅ Requirement 7.1: Dependency checking
- ✅ Requirement 7.2: Clear error messages and installation guidance

## Future Enhancements

Potential improvements for future iterations:
- Support for other package managers (MacPorts, manual download)
- Automatic ffmpeg installation
- Skip wizard option for advanced users
- Wizard reset option in settings
- Multi-language support for wizard content
