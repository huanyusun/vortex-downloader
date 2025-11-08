# Requirements Document

## Introduction

This document outlines the requirements for diagnosing and fixing the download hang issue where downloads are queued but yt-dlp doesn't actually download content. The system needs to properly execute yt-dlp commands, capture output, and handle errors gracefully.

## Glossary

- **DownloadManager**: The Rust component responsible for managing download queue and execution
- **YouTubeProvider**: The platform-specific implementation that interfaces with yt-dlp
- **yt-dlp**: Command-line tool for downloading videos from YouTube and other platforms
- **Progress Callback**: Function that receives download progress updates

## Requirements

### Requirement 1

**User Story:** As a user, I want to see detailed diagnostic information when downloads fail, so that I can understand what went wrong

#### Acceptance Criteria

1. WHEN a download is initiated, THE DownloadManager SHALL log the complete yt-dlp command being executed
2. WHEN yt-dlp produces output, THE YouTubeProvider SHALL log both stdout and stderr in real-time
3. IF yt-dlp fails to start, THEN THE YouTubeProvider SHALL provide a clear error message indicating whether yt-dlp is installed
4. WHEN a download hangs without progress, THE DownloadManager SHALL detect the timeout and report it

### Requirement 2

**User Story:** As a developer, I want yt-dlp to execute with proper parameters, so that downloads work reliably

#### Acceptance Criteria

1. THE YouTubeProvider SHALL use the `--newline` flag to ensure progress is output on separate lines
2. THE YouTubeProvider SHALL use `--no-warnings` to reduce noise in output
3. THE YouTubeProvider SHALL specify output template with proper escaping for special characters
4. THE YouTubeProvider SHALL handle cases where ffmpeg path contains spaces or special characters
5. WHEN executing yt-dlp, THE YouTubeProvider SHALL set proper environment variables for locale

### Requirement 3

**User Story:** As a user, I want downloads to proceed even if progress parsing fails, so that I can still get my content

#### Acceptance Criteria

1. IF progress line parsing fails, THEN THE YouTubeProvider SHALL continue the download without emitting progress
2. THE YouTubeProvider SHALL emit at least one progress update when download completes
3. WHEN yt-dlp completes successfully, THE DownloadManager SHALL mark the download as completed regardless of progress updates received

### Requirement 4

**User Story:** As a user, I want to test if yt-dlp works correctly, so that I can verify my installation

#### Acceptance Criteria

1. THE YouTubeProvider SHALL provide a method to test yt-dlp with a simple command
2. WHEN testing yt-dlp, THE YouTubeProvider SHALL verify both yt-dlp and ffmpeg are accessible
3. THE YouTubeProvider SHALL log the versions of yt-dlp and ffmpeg being used
