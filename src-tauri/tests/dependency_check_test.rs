#[tokio::test]
async fn test_youtube_provider_check_dependencies() {
    // This test requires the actual binaries to be installed
    // It's more of an integration test
    
    // Import the necessary types
    use youtube_downloader_gui::platform::{YouTubeProvider, PlatformProvider};
    
    let provider = YouTubeProvider::new();
    
    // Check dependencies
    let result = provider.check_dependencies().await;
    
    // Should return Ok with a list of dependencies
    assert!(result.is_ok(), "check_dependencies should return Ok");
    
    let dependencies = result.unwrap();
    
    // Should have at least 2 dependencies: yt-dlp and ffmpeg
    assert_eq!(dependencies.len(), 2, "Should have 2 dependencies");
    
    // Check yt-dlp dependency
    let ytdlp_dep = dependencies.iter().find(|d| d.name == "yt-dlp");
    assert!(ytdlp_dep.is_some(), "Should have yt-dlp dependency");
    
    let ytdlp = ytdlp_dep.unwrap();
    println!("yt-dlp installed: {}", ytdlp.installed);
    if let Some(ref version) = ytdlp.version {
        println!("yt-dlp version: {}", version);
    }
    println!("yt-dlp install instructions: {}", ytdlp.install_instructions);
    
    // Check ffmpeg dependency
    let ffmpeg_dep = dependencies.iter().find(|d| d.name == "ffmpeg");
    assert!(ffmpeg_dep.is_some(), "Should have ffmpeg dependency");
    
    let ffmpeg = ffmpeg_dep.unwrap();
    println!("ffmpeg installed: {}", ffmpeg.installed);
    if let Some(ref version) = ffmpeg.version {
        println!("ffmpeg version: {}", version);
    }
    println!("ffmpeg install instructions: {}", ffmpeg.install_instructions);
}
