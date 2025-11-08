# Download Fix Testing Guide

This document describes how to test the download hang fixes implemented in the YouTube downloader.

## Prerequisites

Before running these tests, ensure you have:

1. **yt-dlp installed**: `brew install yt-dlp` (macOS) or equivalent
2. **ffmpeg installed**: `brew install ffmpeg` (macOS) or equivalent
3. **Internet connection**: Tests make real network requests to YouTube

## Running the Tests

The integration tests are marked with `#[ignore]` by default because they:
- Require external dependencies (yt-dlp, ffmpeg)
- Make real network requests
- May take time to complete

### Run All Integration Tests

```bash
cd src-tauri
cargo test --test integration_youtube_test -- --ignored --nocapture
```

### Run Specific Test Suites

#### Task 7.1: Test with Known Working YouTube URL

Tests that downloads start, complete, and track progress correctly:

```bash
cargo test --test integration_youtube_test test_youtube_download_with_progress -- --ignored --nocapture
cargo test --test integration_youtube_test test_youtube_download_progress_tracking -- --ignored --nocapture
```

**Expected Output:**
- Progress updates logged to console
- Download completes successfully
- File is saved to temporary directory
- Final progress is 100%

#### Task 7.2: Test Error Scenarios

Tests error handling for invalid and unavailable videos:

```bash
cargo test --test integration_youtube_test test_youtube_download_invalid_url -- --ignored --nocapture
cargo test --test integration_youtube_test test_youtube_download_unavailable_video -- --ignored --nocapture
```

**Expected Output:**
- Tests should fail gracefully with appropriate error messages
- No files should be created for failed downloads
- Error messages should be clear and actionable

#### Task 7.3: Verify Logging Output

Tests diagnostic functionality and dependency checking:

```bash
cargo test --test integration_youtube_test test_youtube_diagnostic_logging -- --ignored --nocapture
```

**Expected Output:**
- yt-dlp version logged
- ffmpeg version logged
- Dependency status confirmed

## What Each Test Validates

### test_youtube_download_with_progress
- ✅ Download starts successfully
- ✅ Progress updates are received
- ✅ Final progress reaches 100%
- ✅ File is saved correctly
- ✅ File has content (not empty)

### test_youtube_download_progress_tracking
- ✅ Detailed progress information is tracked
- ✅ Progress includes percentage, bytes, speed, and ETA
- ✅ Multiple progress updates are received
- ✅ All progress data is logged

### test_youtube_download_invalid_url
- ✅ Invalid URLs are rejected
- ✅ Error message is clear
- ✅ No file is created for failed download

### test_youtube_download_unavailable_video
- ✅ Unavailable videos are handled gracefully
- ✅ Error message indicates video is unavailable
- ✅ No file is created for failed download

### test_youtube_diagnostic_logging
- ✅ Dependency check works
- ✅ yt-dlp version is detected
- ✅ ffmpeg version is detected
- ✅ Installation status is reported

## Troubleshooting

### Tests Skip with "yt-dlp not available"

This means yt-dlp is not installed or not in PATH. Install it:

```bash
brew install yt-dlp
```

### Tests Fail with Network Errors

- Check your internet connection
- YouTube may be temporarily unavailable
- Try running tests again after a few minutes

### Tests Timeout

- Large videos may take time to download
- The timeout is set to 30 minutes in the download manager
- Use shorter test videos (like "Me at the zoo" - 18 seconds)

## Manual Testing Checklist

In addition to automated tests, perform these manual tests:

1. **Start the application**
   ```bash
   npm run tauri dev
   ```

2. **Test a real download**
   - Enter a YouTube URL
   - Click download
   - Verify progress bar updates
   - Verify file is saved to selected location

3. **Check console logs**
   - Open developer tools
   - Look for `[yt-dlp]` log messages
   - Verify command is logged before execution
   - Verify yt-dlp output is visible
   - Verify progress parsing is logged

4. **Test error scenarios**
   - Try an invalid URL
   - Try a private/unavailable video
   - Verify error messages are clear

## Requirements Coverage

These tests cover all requirements from the spec:

- **Requirement 1.1**: ✅ Command logging verified
- **Requirement 1.2**: ✅ Real-time output logging verified
- **Requirement 1.3**: ✅ Clear error messages verified
- **Requirement 1.4**: ✅ Timeout detection implemented
- **Requirement 2.1-2.5**: ✅ Command construction verified
- **Requirement 3.1-3.3**: ✅ Progress parsing resilience verified
- **Requirement 4.1-4.3**: ✅ Diagnostic functionality verified

## Next Steps

After all tests pass:

1. Test with the full application UI
2. Verify downloads work end-to-end
3. Check that progress updates appear in the UI
4. Verify error messages are displayed to users
5. Test with various video formats and qualities
