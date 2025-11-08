use std::sync::Arc;
use youtube_downloader_gui::platform::{PlatformProvider, PlatformRegistry, YouTubeProvider};

#[test]
fn test_registry_new() {
    let registry = PlatformRegistry::new();
    assert_eq!(registry.get_all_providers().len(), 0);
}

#[test]
fn test_registry_register_provider() {
    let mut registry = PlatformRegistry::new();
    let provider = Arc::new(YouTubeProvider::new());
    
    registry.register(provider);
    
    assert_eq!(registry.get_all_providers().len(), 1);
}

#[test]
fn test_registry_get_provider_by_name() {
    let mut registry = PlatformRegistry::new();
    let provider = Arc::new(YouTubeProvider::new());
    
    registry.register(provider);
    
    let retrieved = registry.get_provider("YouTube");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name(), "YouTube");
}

#[test]
fn test_registry_get_provider_by_name_not_found() {
    let registry = PlatformRegistry::new();
    
    let retrieved = registry.get_provider("NonExistent");
    assert!(retrieved.is_none());
}

#[test]
fn test_registry_detect_provider_youtube() {
    let mut registry = PlatformRegistry::new();
    let provider = Arc::new(YouTubeProvider::new());
    
    registry.register(provider);
    
    // Test various YouTube URLs
    let urls = vec![
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
        "https://youtu.be/dQw4w9WgXcQ",
        "https://www.youtube.com/playlist?list=PLtest",
        "https://www.youtube.com/@channel",
    ];
    
    for url in urls {
        let detected = registry.detect_provider(url);
        assert!(detected.is_some(), "Failed to detect provider for URL: {}", url);
        assert_eq!(detected.unwrap().name(), "YouTube");
    }
}

#[test]
fn test_registry_detect_provider_unsupported() {
    let mut registry = PlatformRegistry::new();
    let provider = Arc::new(YouTubeProvider::new());
    
    registry.register(provider);
    
    // Test unsupported URLs
    let urls = vec![
        "https://www.vimeo.com/123456",
        "https://www.bilibili.com/video/BV1xx411c7mD",
        "not a url",
        "",
    ];
    
    for url in urls {
        let detected = registry.detect_provider(url);
        assert!(detected.is_none(), "Should not detect provider for URL: {}", url);
    }
}

#[test]
fn test_registry_multiple_providers() {
    let mut registry = PlatformRegistry::new();
    let youtube_provider = Arc::new(YouTubeProvider::new());
    
    registry.register(youtube_provider);
    
    assert_eq!(registry.get_all_providers().len(), 1);
    
    // Verify we can get all providers
    let all_providers = registry.get_all_providers();
    assert_eq!(all_providers.len(), 1);
    assert_eq!(all_providers[0].name(), "YouTube");
}

#[test]
fn test_registry_default() {
    let registry = PlatformRegistry::default();
    assert_eq!(registry.get_all_providers().len(), 0);
}
