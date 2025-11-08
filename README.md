# Vortex Downloader

A modern, user-friendly YouTube video downloader for macOS. Download individual videos, entire playlists, or channels with this high-performance desktop application, built with Tauri and React.

![macOS](https://img.shields.io/badge/macOS-10.15+-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Version](https://img.shields.io/badge/version-0.1.0-orange)

## ✨ Features

- 🎥 **Video Downloads**: Download individual YouTube videos in various qualities
- 📋 **Playlist Support**: Download entire playlists with automatic organization
- 📺 **Channel Downloads**: Download all videos from a channel, organized by playlists
- ⚡ **Queue Management**: Manage multiple downloads with pause/resume/cancel controls
- 🎯 **Smart Organization**: Automatically organize downloads by channel and playlist
- ⚙️ **Configurable Settings**: Customize quality, format, concurrent downloads, and more
- 🔄 **Queue Persistence**: Resume interrupted downloads after app restart
- 🎨 **Modern UI**: Clean, intuitive interface built with React and Tailwind CSS
- 🚀 **High Performance**: Native performance with Rust backend and optimized caching

## 🛠 Technology Stack

- **Frontend**: React 18 + TypeScript + Tailwind CSS
- **Backend**: Rust (Tauri Framework)
- **State Management**: Zustand
- **Download Engine**: yt-dlp
- **Video Processing**: ffmpeg

## 📋 Prerequisites

### Required

- **macOS 10.15 (Catalina)** or later
- **Node.js** v16 or later
- **Rust** (latest stable)
- **yt-dlp** - Will be checked on first launch with installation guidance
- **ffmpeg** - Required by yt-dlp for video processing

### Installation

```bash
# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install yt-dlp and ffmpeg
brew install yt-dlp ffmpeg

# Install Node.js (if needed)
brew install node

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## 🚀 Quick Start

### Development

```bash
# Clone the repository
git clone <repository-url>
cd vortex-downloader

# Install dependencies
npm install

# Run in development mode
npm run tauri:dev
```

### Building

```bash
# Build for production (universal binary)
npm run tauri:build:universal

# Or build for specific architecture
npm run tauri:build:intel    # Intel Macs
npm run tauri:build:arm      # Apple Silicon Macs
```

See [BUILD.md](BUILD.md) for detailed build instructions, code signing, and distribution.

## 📁 Project Structure

```
.
├── src/                        # React frontend
│   ├── components/            # UI components
│   │   ├── URLInputPanel.tsx
│   │   ├── VideoPreviewPanel.tsx
│   │   ├── DownloadQueuePanel.tsx
│   │   ├── SettingsPanel.tsx
│   │   └── WelcomeWizard.tsx
│   ├── hooks/                 # Custom React hooks
│   ├── stores/                # Zustand state stores
│   ├── api/                   # Tauri API wrappers
│   ├── types/                 # TypeScript type definitions
│   └── App.tsx                # Main application component
│
├── src-tauri/                 # Rust backend
│   ├── src/
│   │   ├── main.rs           # Application entry point
│   │   ├── commands.rs       # Tauri command handlers
│   │   ├── platform/         # Platform provider system
│   │   │   ├── provider.rs   # Provider trait definition
│   │   │   ├── registry.rs   # Provider registry
│   │   │   ├── youtube.rs    # YouTube implementation
│   │   │   └── cache.rs      # Metadata caching
│   │   ├── download/         # Download management
│   │   │   ├── manager.rs    # Queue and concurrency
│   │   │   ├── task.rs       # Download task types
│   │   │   └── throttle.rs   # Progress throttling
│   │   ├── storage/          # Persistence layer
│   │   │   ├── service.rs    # Storage operations
│   │   │   └── settings.rs   # Settings types
│   │   └── error.rs          # Error handling
│   ├── Cargo.toml            # Rust dependencies
│   └── tauri.conf.json       # Tauri configuration
│
├── BUILD.md                   # Detailed build instructions
├── package.json              # Node.js dependencies
└── README.md                 # This file
```

## 🏗 Architecture

### Plugin-Based Platform System

The application uses an extensible plugin architecture that makes it easy to add support for new video platforms:

```rust
// Define a new platform provider
pub struct NewPlatformProvider { }

#[async_trait]
impl PlatformProvider for NewPlatformProvider {
    fn name(&self) -> &str { "NewPlatform" }
    fn matches_url(&self, url: &str) -> bool { /* ... */ }
    async fn get_video_info(&self, url: &str) -> Result<VideoInfo> { /* ... */ }
    // ... implement other methods
}

// Register the provider
platform_registry.register(Arc::new(NewPlatformProvider::new()));
```

### Key Components

- **PlatformProvider Trait**: Interface for all platform implementations
- **PlatformRegistry**: Manages providers and routes URLs to the correct handler
- **DownloadManager**: Handles queue, concurrency, and state persistence
- **StorageService**: Manages file operations and settings
- **MetadataCache**: Caches video/playlist info (5-minute TTL)
- **ProgressThrottler**: Throttles UI updates to 500ms intervals

### Performance Optimizations

- ✅ **Progress Throttling**: Updates limited to 500ms intervals
- ✅ **Metadata Caching**: 5-minute TTL for video/playlist info
- ✅ **Async I/O**: Non-blocking file and network operations
- ✅ **Queue Persistence**: Automatic save/restore of download state
- ✅ **Pagination Support**: Efficient handling of large playlists

## 🎯 Usage

1. **Launch the application**
2. **First-time setup**: Follow the welcome wizard to check dependencies and set default save location
3. **Enter a YouTube URL**: Video, playlist, or channel
4. **Preview content**: Review videos before downloading
5. **Select videos**: Choose which videos to download
6. **Add to queue**: Videos are added to the download queue
7. **Manage downloads**: Pause, resume, cancel, or reorder as needed

## ⚙️ Configuration

Settings are accessible via the gear icon in the top-right corner:

- **Default Save Path**: Where videos are saved
- **Default Quality**: Video quality preference (best, 1080p, 720p, 480p)
- **Default Format**: Video format (mp4, webm, mkv)
- **Max Concurrent Downloads**: 1-5 simultaneous downloads
- **Auto Retry**: Automatically retry failed downloads
- **Platform Settings**: Platform-specific options

## 🔧 Development

### Available Scripts

```bash
npm run dev              # Start Vite dev server
npm run build            # Build frontend
npm run tauri:dev        # Run Tauri in development mode
npm run tauri:build      # Build production app
npm run tauri:build:universal  # Build universal binary
```

### Testing

```bash
# Run Rust tests
cd src-tauri
cargo test

# Run with logging
RUST_LOG=debug npm run tauri:dev
```

## 🐛 Troubleshooting

### Common Issues

**"yt-dlp not found"**
```bash
brew install yt-dlp
```

**"ffmpeg not found"**
```bash
brew install ffmpeg
```

**Build fails**
```bash
# Clean and rebuild
cd src-tauri
cargo clean
cd ..
npm run tauri:build
```

**App won't open (Gatekeeper)**
- Right-click the app and select "Open"
- Or: System Preferences → Security & Privacy → Allow

## 🗺 Roadmap

- [x] Core download functionality
- [x] Queue management
- [x] Settings persistence
- [x] Welcome wizard
- [x] Performance optimizations
- [ ] Batch operations
- [ ] Download history
- [ ] Subtitle support
- [ ] Additional platform support (Vimeo, etc.)
- [ ] Auto-update functionality

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚖️ Disclaimer

This application is for personal and educational use only. By using this application, you agree to the following terms:

- You will not use this application for any purpose that violates copyright laws.
- You will only download content for which you have the legal right to do so.
- The developers of this application are not responsible for any misuse of this software.

Please respect the terms of service of YouTube and other video platforms.


## 🙏 Acknowledgments

- [Tauri](https://tauri.app/) - Desktop application framework
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) - Video download engine
- [React](https://react.dev/) - UI framework
- [Tailwind CSS](https://tailwindcss.com/) - Styling

## 📞 Support

For issues, questions, or contributions, please open an issue on GitHub.

---

**Note**: This application is for personal use only. Please respect YouTube's Terms of Service and copyright laws.
