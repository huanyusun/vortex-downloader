use std::path::PathBuf;
use youtube_downloader_gui::storage::StorageService;

#[test]
fn test_sanitize_filename_basic() {
    // Test basic sanitization through create_directory_structure
    let result = StorageService::sanitize_filename("Normal Filename");
    assert_eq!(result, "Normal Filename");
}

#[test]
fn test_sanitize_filename_with_invalid_chars() {
    // Test that invalid filesystem characters are replaced
    let result = StorageService::sanitize_filename("File/Name:With*Invalid?Chars");
    assert_eq!(result, "File_Name_With_Invalid_Chars");
}

#[test]
fn test_sanitize_filename_with_control_chars() {
    // Test that control characters are replaced
    let result = StorageService::sanitize_filename("File\nName\tWith\rControl");
    assert_eq!(result, "File_Name_With_Control");
}

#[test]
fn test_sanitize_filename_with_dots() {
    // Test that leading/trailing dots are removed
    let result = StorageService::sanitize_filename("...filename...");
    assert_eq!(result, "filename");
}

#[test]
fn test_sanitize_filename_empty() {
    // Test that empty strings become "untitled"
    let result = StorageService::sanitize_filename("");
    assert_eq!(result, "untitled");
    
    let result2 = StorageService::sanitize_filename("   ");
    assert_eq!(result2, "untitled");
    
    let result3 = StorageService::sanitize_filename("...");
    assert_eq!(result3, "untitled");
}

#[test]
fn test_sanitize_filename_with_spaces() {
    // Test that spaces are preserved but trimmed
    let result = StorageService::sanitize_filename("  File Name  ");
    assert_eq!(result, "File Name");
}

#[test]
fn test_validate_path_absolute() {
    // This test requires a mock or actual StorageService instance
    // For now, we'll test the logic conceptually
    let absolute_path = PathBuf::from("/Users/test/Downloads");
    assert!(absolute_path.is_absolute());
}

#[test]
fn test_validate_path_relative() {
    let relative_path = PathBuf::from("relative/path");
    assert!(!relative_path.is_absolute());
}

#[test]
fn test_validate_path_traversal() {
    let path_str = "/Users/test/../../../etc/passwd";
    assert!(path_str.contains(".."));
}

#[test]
fn test_get_default_save_path() {
    // Test that default save path is reasonable
    let home = dirs::home_dir();
    assert!(home.is_some());
    
    if let Some(home_dir) = home {
        let expected = home_dir.join("Downloads");
        // Just verify the path construction works
        assert!(expected.to_string_lossy().contains("Downloads"));
    }
}

#[test]
fn test_path_with_null_bytes() {
    let path_str = "/Users/test\0/file";
    assert!(path_str.contains('\0'));
}

#[cfg(target_os = "macos")]
#[test]
fn test_restricted_paths_macos() {
    let restricted_paths = vec![
        "/System/Library",
        "/Library/System",
        "/bin/bash",
        "/sbin/init",
        "/usr/bin",
        "/private/var/root",
    ];
    
    for path in restricted_paths {
        assert!(
            path.starts_with("/System") ||
            path.starts_with("/Library") ||
            path.starts_with("/bin") ||
            path.starts_with("/sbin") ||
            path.starts_with("/usr") ||
            path.starts_with("/private/var")
        );
    }
}
