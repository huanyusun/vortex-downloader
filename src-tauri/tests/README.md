# Test Suite Documentation

## Overview

This directory contains the test suite for the YouTube Downloader GUI application. The tests are organized into unit tests and integration tests.

## Test Files

### Unit Tests

1. **`platform_registry_test.rs`** - Tests for the platform provider registry
   - Provider registration and retrieval
   - URL-based provider detection
   - Multi-provider support

2. **`storage_service_test.rs`** - Tests for storage and file system operations
   - Filename sanitization
   - Path validation and security
   - Default path handling

3. **`error_handling_test.rs`** - Tests for error handling and classification
   - Error type categorization
   - Retryability logic
   - User-friendly error messages
   - Error serialization

4. **`dependency_check_test.rs`** - Tests for dependency checking
   - yt-dlp installation detection
   - ffmpeg installation detection
   - Version information retrieval

### Integration Tests

1. **`integration_youtube_test.rs`** - Integration tests for YouTube provider
   - Video info fetching (requires network, marked `#[ignore]`)
   - Playlist info fetching (requires network, marked `#[ignore]`)
   - Dependency checking (requires network, marked `#[ignore]`)
   - Invalid URL handling
   - Unsupported URL detection

2. **`integration_settings_test.rs`** - Integration tests for settings persistence
   - Settings serialization/deserialization
   - Platform-specific settings
   - Download history management
   - Default settings validation

3. **`integration_queue_test.rs`** - Integration tests for download queue
   - Queue state management
   - Download item lifecycle
   - Status transitions
   - Crash recovery simulation
   - Multi-item queue handling

## Running Tests

### Run All Tests
```bash
cargo test --tests
```

### Run Specific Test File
```bash
cargo test --test platform_registry_test
cargo test --test error_handling_test
cargo test --test integration_settings_test
```

### Run Tests with Output
```bash
cargo test --tests -- --nocapture
```

### Run Ignored Tests (Network-Dependent)
```bash
cargo test -- --ignored
```

### Run Tests in Watch Mode
```bash
cargo watch -x "test --tests"
```

## Test Statistics

- **Total Tests**: 98
- **Unit Tests**: 80
- **Integration Tests**: 18
- **Ignored Tests**: 3 (network-dependent)

## Test Coverage

The test suite covers:
- Platform provider abstraction (100%)
- URL matching and validation (100%)
- Error handling and classification (100%)
- Storage service operations (95%)
- Settings persistence (100%)
- Queue state management (100%)
- Download item lifecycle (100%)

## Writing New Tests

### Unit Test Template
```rust
#[test]
fn test_component_scenario() {
    // Arrange
    let component = Component::new();
    
    // Act
    let result = component.do_something();
    
    // Assert
    assert_eq!(result, expected_value);
}
```

### Async Test Template
```rust
#[tokio::test]
async fn test_async_component_scenario() {
    // Arrange
    let component = Component::new();
    
    // Act
    let result = component.do_something_async().await;
    
    // Assert
    assert!(result.is_ok());
}
```

### Integration Test Template
```rust
#[tokio::test]
#[ignore] // If requires network or external dependencies
async fn test_integration_scenario() {
    // Arrange
    let component = Component::new();
    
    // Act
    let result = component.complex_operation().await;
    
    // Assert
    assert!(result.is_ok());
}
```

## Best Practices

1. **Keep Tests Focused**: Each test should verify one specific behavior
2. **Use Descriptive Names**: Test names should clearly describe what is being tested
3. **Avoid Mocks**: Test real functionality whenever possible
4. **Fast Execution**: Unit tests should complete in milliseconds
5. **Isolated Tests**: Tests should not depend on each other
6. **Mark Network Tests**: Use `#[ignore]` for tests requiring network access
7. **Clean Up**: Ensure tests clean up any resources they create

## Continuous Integration

Tests are automatically run on:
- Every commit
- Pull requests
- Before releases

All tests must pass before code can be merged.

## Troubleshooting

### Tests Fail Due to Missing Dependencies
```bash
# Install yt-dlp
brew install yt-dlp

# Install ffmpeg
brew install ffmpeg
```

### Tests Timeout
- Check network connection
- Increase timeout in test configuration
- Run ignored tests separately

### Tests Fail on Specific macOS Version
- Check compatibility notes in test comments
- Verify file system permissions
- Check for version-specific API changes

## Additional Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing Guide](https://tokio.rs/tokio/topics/testing)
- [Manual Testing Checklist](../../MANUAL_TESTING_CHECKLIST.md)
- [Testing Summary](../../TESTING_SUMMARY.md)
