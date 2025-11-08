# Manual Testing Checklist

This document provides a comprehensive checklist for manual testing of the YouTube Downloader GUI application.

## Prerequisites

- macOS 10.15 (Catalina) or later
- yt-dlp installed (`brew install yt-dlp`)
- ffmpeg installed (`brew install ffmpeg`)
- Internet connection for testing downloads

## Test Environment Setup

1. Build the application: `npm run tauri build`
2. Install the built application
3. Prepare test URLs:
   - Single video: `https://www.youtube.com/watch?v=dQw4w9WgXcQ`
   - Small playlist (5-10 videos): `https://www.youtube.com/playlist?list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf`
   - Large playlist (100+ videos): Find a suitable test playlist
   - Channel: `https://www.youtube.com/@LinusTechTips`

---

## 1. macOS Version Compatibility Testing

### Test on macOS 10.15 (Catalina)
- [ ] Application launches successfully
- [ ] All UI elements render correctly
- [ ] Basic download functionality works
- [ ] Settings can be saved and loaded

### Test on macOS 11 (Big Sur)
- [ ] Application launches successfully
- [ ] All UI elements render correctly
- [ ] Basic download functionality works
- [ ] Settings can be saved and loaded

### Test on macOS 12 (Monterey)
- [ ] Application launches successfully
- [ ] All UI elements render correctly
- [ ] Basic download functionality works
- [ ] Settings can be saved and loaded

### Test on macOS 13 (Ventura)
- [ ] Application launches successfully
- [ ] All UI elements render correctly
- [ ] Basic download functionality works
- [ ] Settings can be saved and loaded

### Test on macOS 14 (Sonoma)
- [ ] Application launches successfully
- [ ] All UI elements render correctly
- [ ] Basic download functionality works
- [ ] Settings can be saved and loaded

**Notes:**
- Document any version-specific issues
- Check for UI rendering differences
- Verify file system permissions work correctly

---

## 2. Large Playlist Performance Testing

### Test with 100+ Video Playlist

#### Initial Load
- [ ] Playlist information loads within reasonable time (< 30 seconds)
- [ ] UI remains responsive during loading
- [ ] Progress indicator shows loading state
- [ ] Memory usage stays reasonable (< 500 MB)

#### Video Selection
- [ ] Can scroll through entire video list smoothly
- [ ] Checkboxes respond immediately
- [ ] "Select All" works without freezing
- [ ] "Deselect All" works without freezing

#### Adding to Queue
- [ ] Adding 100+ videos to queue completes successfully
- [ ] Queue UI updates correctly
- [ ] Application doesn't crash or freeze
- [ ] Memory usage remains stable

#### Download Processing
- [ ] Downloads start automatically
- [ ] Concurrent download limit is respected (default: 3)
- [ ] Progress updates for all active downloads
- [ ] Queue processes all items eventually
- [ ] No memory leaks during long-running downloads

**Performance Metrics to Record:**
- Time to load playlist info: _____ seconds
- Memory usage during load: _____ MB
- Memory usage during downloads: _____ MB
- CPU usage during downloads: _____ %

---

## 3. Network Exception Scenarios

### Test Network Interruption During Download

#### Scenario 1: WiFi Disconnection
- [ ] Start a download
- [ ] Disconnect WiFi during download
- [ ] Verify error message appears
- [ ] Verify download status changes to "Failed"
- [ ] Reconnect WiFi
- [ ] Verify retry functionality works
- [ ] Verify download can be resumed

#### Scenario 2: Slow Network
- [ ] Throttle network speed (use Network Link Conditioner)
- [ ] Start a download
- [ ] Verify download continues (slowly)
- [ ] Verify progress updates correctly
- [ ] Verify ETA calculation is reasonable
- [ ] Verify download completes successfully

#### Scenario 3: Network Timeout
- [ ] Start a download
- [ ] Simulate network timeout (block YouTube domains)
- [ ] Verify timeout error is caught
- [ ] Verify friendly error message is shown
- [ ] Verify retry option is available

#### Scenario 4: DNS Failure
- [ ] Configure invalid DNS server
- [ ] Attempt to fetch video info
- [ ] Verify appropriate error message
- [ ] Restore DNS settings
- [ ] Verify functionality returns

**Error Messages to Verify:**
- [ ] Network errors are user-friendly
- [ ] Suggested actions are helpful
- [ ] Retry button is available for retryable errors

---

## 4. Application Crash Recovery

### Test Queue Persistence After Crash

#### Scenario 1: Force Quit During Download
- [ ] Add multiple videos to download queue
- [ ] Start downloads
- [ ] Force quit application (Cmd+Q or Force Quit)
- [ ] Relaunch application
- [ ] Verify queue is restored
- [ ] Verify downloading items are reset to "Queued"
- [ ] Verify completed items remain "Completed"
- [ ] Verify downloads can be resumed

#### Scenario 2: System Crash Simulation
- [ ] Add multiple videos to download queue
- [ ] Start downloads
- [ ] Kill application process (`kill -9 <pid>`)
- [ ] Relaunch application
- [ ] Verify queue is restored
- [ ] Verify no data corruption
- [ ] Verify application is stable after recovery

#### Scenario 3: Disk Full During Download
- [ ] Start a large download
- [ ] Fill up disk space during download
- [ ] Verify error is caught gracefully
- [ ] Verify application doesn't crash
- [ ] Verify error message is clear
- [ ] Free up disk space
- [ ] Verify retry works

**Recovery Verification:**
- [ ] Queue state file exists and is valid JSON
- [ ] Settings are preserved
- [ ] Download history is intact
- [ ] No orphaned files in download directory

---

## 5. First Launch Experience

### Test Welcome Wizard
- [ ] Delete application data directory
- [ ] Launch application
- [ ] Verify welcome wizard appears
- [ ] Verify dependency check runs
- [ ] Verify yt-dlp status is shown correctly
- [ ] Verify ffmpeg status is shown correctly
- [ ] Verify installation instructions are clear
- [ ] Complete wizard
- [ ] Verify default settings are applied
- [ ] Verify wizard doesn't show on subsequent launches

---

## 6. Settings Persistence

### Test Settings Save/Load
- [ ] Change default save path
- [ ] Change default quality
- [ ] Change max concurrent downloads
- [ ] Save settings
- [ ] Quit application
- [ ] Relaunch application
- [ ] Verify all settings are preserved

---

## 7. Download Functionality

### Test Single Video Download
- [ ] Enter valid YouTube video URL
- [ ] Fetch video info
- [ ] Verify video metadata is displayed
- [ ] Select quality and format
- [ ] Add to download queue
- [ ] Verify download starts
- [ ] Verify progress updates
- [ ] Verify download completes
- [ ] Verify file exists in save location
- [ ] Verify file is playable

### Test Playlist Download
- [ ] Enter valid playlist URL
- [ ] Fetch playlist info
- [ ] Verify all videos are listed
- [ ] Select specific videos
- [ ] Add to download queue
- [ ] Verify downloads process correctly
- [ ] Verify playlist folder is created
- [ ] Verify all files are in correct location

### Test Channel Download
- [ ] Enter valid channel URL
- [ ] Fetch channel info
- [ ] Verify channel structure is shown
- [ ] Verify playlists are grouped correctly
- [ ] Select videos from different playlists
- [ ] Add to download queue
- [ ] Verify folder structure is created correctly
- [ ] Verify downloads complete successfully

---

## 8. Queue Management

### Test Queue Operations
- [ ] Add multiple items to queue
- [ ] Pause a download
- [ ] Verify download pauses
- [ ] Resume download
- [ ] Verify download resumes
- [ ] Cancel a download
- [ ] Verify download is cancelled
- [ ] Reorder queue items (drag and drop)
- [ ] Verify order changes take effect

---

## 9. Error Handling

### Test Various Error Scenarios
- [ ] Invalid URL
- [ ] Private video
- [ ] Deleted video
- [ ] Age-restricted video
- [ ] Region-restricted video
- [ ] Insufficient disk space
- [ ] Invalid save path
- [ ] No write permissions

**For Each Error:**
- [ ] Verify error is caught
- [ ] Verify error message is user-friendly
- [ ] Verify suggested action is helpful
- [ ] Verify application remains stable

---

## 10. UI/UX Testing

### Test User Interface
- [ ] All buttons are clickable
- [ ] All text is readable
- [ ] No UI elements overlap
- [ ] Responsive to window resizing
- [ ] Dark mode support (if applicable)
- [ ] Tooltips are helpful
- [ ] Loading states are clear
- [ ] Progress bars animate smoothly

---

## Test Results Summary

### Overall Results
- Total Tests: _____
- Passed: _____
- Failed: _____
- Blocked: _____

### Critical Issues Found
1. 
2. 
3. 

### Minor Issues Found
1. 
2. 
3. 

### Performance Notes
- 

### Compatibility Notes
- 

### Recommendations
- 

---

## Sign-off

**Tester Name:** _____________________
**Date:** _____________________
**macOS Version Tested:** _____________________
**Application Version:** _____________________
**Signature:** _____________________
