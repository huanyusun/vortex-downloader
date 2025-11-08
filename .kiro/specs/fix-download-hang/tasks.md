# Implementation Plan

- [x] 1. Add enhanced logging to YouTubeProvider
  - Add logging before yt-dlp command execution showing full command and arguments
  - Add real-time logging of all stdout and stderr output from yt-dlp
  - Add logging for progress parsing attempts (both successful and failed)
  - Add logging for final download status
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 2. Improve yt-dlp command construction
  - [x] 2.1 Update download_video_impl to use improved command flags
    - Add `--no-color` flag to prevent ANSI color codes
    - Add `--progress` flag to force progress output
    - Add `--no-warnings` flag to reduce noise
    - Set `PYTHONIOENCODING=utf-8` environment variable
    - Set `LANG=en_US.UTF-8` environment variable
    - _Requirements: 2.1, 2.2, 2.5_
  
  - [x] 2.2 Fix output path handling for special characters
    - Ensure save_path is properly escaped
    - Handle paths with spaces and special characters
    - _Requirements: 2.3_
  
  - [x] 2.3 Improve ffmpeg path handling
    - Quote ffmpeg path if it contains spaces
    - Validate ffmpeg path exists before starting download
    - _Requirements: 2.4_

- [x] 3. Make progress parsing more resilient
  - [x] 3.1 Add multiple progress parsing patterns
    - Keep existing standard format parser
    - Add parser for "[download] Destination:" lines (0% progress)
    - Add parser for "[download] 100%" completion lines
    - Add parser for "[download] has already been downloaded" lines
    - _Requirements: 3.1_
  
  - [x] 3.2 Add defensive error handling in parse_progress_line
    - Wrap all regex operations in error handling
    - Log unparseable lines that contain "[download]"
    - Return None instead of panicking on parse errors
    - _Requirements: 3.1_
  
  - [x] 3.3 Ensure completion progress is always sent
    - Send 100% progress when yt-dlp exits successfully
    - Send completion regardless of whether progress updates were received
    - _Requirements: 3.2, 3.3_

- [x] 4. Add diagnostic and testing utilities
  - [x] 4.1 Add test_download method to YouTubeProvider
    - Create method that tests yt-dlp with simple command
    - Test should verify yt-dlp can fetch video title
    - Return clear error if yt-dlp is not working
    - _Requirements: 4.1_
  
  - [x] 4.2 Add version checking methods
    - Add method to get yt-dlp version
    - Add method to get ffmpeg version
    - Log versions at startup
    - _Requirements: 4.2, 4.3_
  
  - [x] 4.3 Add command to expose test functionality to frontend
    - Create Tauri command for testing yt-dlp
    - Allow users to verify installation from UI
    - _Requirements: 4.1_

- [x] 5. Add timeout detection to DownloadManager
  - Wrap download execution in tokio::time::timeout
  - Set reasonable timeout (e.g., 30 minutes for large videos)
  - Report timeout errors clearly to user
  - _Requirements: 1.4_

- [x] 6. Read remaining lines of youtube.rs file
  - Use readFile with start_line parameter to read lines 777-1174
  - Verify complete implementation of download_video method
  - Check if there are any other methods that need updating
  - _Requirements: All_

- [x] 7. Test the fixes
  - [x] 7.1 Test with a known working YouTube URL
    - Verify download starts and completes
    - Verify progress updates are received
    - Verify file is saved correctly
    - _Requirements: All_
  
  - [x] 7.2 Test error scenarios
    - Test with invalid URL
    - Test with unavailable video
    - Test with yt-dlp not installed (if possible)
    - _Requirements: 1.3, 1.4_
  
  - [x] 7.3 Verify logging output
    - Check that command is logged before execution
    - Check that yt-dlp output is visible in logs
    - Check that progress parsing is logged
    - _Requirements: 1.1, 1.2_
