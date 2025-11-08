# Project Overview

This is a Vortex Downloader desktop application for macOS, built with the Tauri framework. The application provides a graphical user interface for downloading videos, playlists, and channels from YouTube.

**Key Technologies:**

*   **Frontend:** React, TypeScript, Vite, Tailwind CSS
*   **Backend:** Rust, Tauri
*   **State Management:** Zustand
*   **Download Engine:** yt-dlp
*   **Video Processing:** ffmpeg

**Architecture:**

The application uses a plugin-based platform system, making it extensible for other video platforms. The core components include:

*   **`PlatformProvider` Trait:** An interface for all platform implementations.
*   **`PlatformRegistry`:** Manages providers and routes URLs to the correct handler.
*   **`DownloadManager`:** Handles the download queue, concurrency, and state persistence.
*   **`StorageService`:** Manages file operations and settings.
*   **`MetadataCache`:** Caches video and playlist information.
*   **`ProgressThrottler`:** Throttles UI updates to prevent performance issues.

# Building and Running

**Prerequisites:**

*   macOS 10.15 (Catalina) or later
*   Node.js v16 or later
*   Rust (latest stable)
*   yt-dlp
*   ffmpeg

**Development:**

1.  Install dependencies:
    ```bash
    npm install
    ```
2.  Run in development mode:
    ```bash
    npm run tauri:dev
    ```

**Building:**

*   Build for production (universal binary):
    ```bash
    npm run tauri:build:universal
    ```
*   Build for a specific architecture:
    ```bash
    # Intel Macs
    npm run tauri:build:intel

    # Apple Silicon Macs
    npm run tauri:build:arm
    ```

# Development Conventions

*   The frontend code is located in the `src` directory and follows standard React and TypeScript conventions.
*   The backend code is in the `src-tauri` directory and is written in Rust.
*   State management is handled by Zustand.
*   Styling is done with Tailwind CSS.
*   Tauri is used for the application framework and native API access.
*   Rust tests can be run with `cargo test` in the `src-tauri` directory.

# Disclaimer

This application is for personal and educational use only. By using this application, you agree to the following terms:

- You will not use this application for any purpose that violates copyright laws.
- You will only download content for which you have the legal right to do so.
- The developers of this application are not responsible for any misuse of this software.

Please respect the terms of service of YouTube and other video platforms.