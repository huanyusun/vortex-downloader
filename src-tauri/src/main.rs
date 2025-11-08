// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use youtube_downloader_gui::{platform, download, storage, executable_manager};

use std::sync::Arc;
use tauri::{AppHandle, Manager};
use platform::{PlatformRegistry, YouTubeProvider};
use download::DownloadManager;
use storage::StorageService;
use executable_manager::ExecutableManager;

#[derive(Clone)]
pub struct AppState {
    platform_registry: Arc<PlatformRegistry>,
    download_manager: Arc<DownloadManager>,
    storage_service: Arc<StorageService>,
    metadata_cache: Arc<platform::MetadataCache>,
}

/// Initialize the application with all required services and state
fn initialize_app(app_handle: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing YouTube Downloader application...");
    
    // Step 0: Initialize ExecutableManager and verify bundled executables
    println!("Initializing executable manager...");
    let package_info = app_handle.package_info();
    let executable_manager = ExecutableManager::new(package_info)
        .expect("Failed to initialize executable manager");
    
    println!("Verifying bundled executables...");
    match executable_manager.initialize() {
        Ok(_) => {
            println!("  ✓ Bundled executables verified and ready");
            println!("  ✓ Architecture: {:?}", executable_manager.architecture());
        }
        Err(e) => {
            eprintln!("ERROR: Failed to verify bundled executables: {}", e);
            eprintln!("Please reinstall the application.");
            return Err(Box::new(e));
        }
    }
    
    // Get paths to bundled executables
    let ytdlp_path = executable_manager.get_ytdlp_path();
    let ffmpeg_path = executable_manager.get_ffmpeg_path();
    println!("  ✓ yt-dlp path: {:?}", ytdlp_path);
    println!("  ✓ ffmpeg path: {:?}", ffmpeg_path);
    
    // Step 1: Initialize platform registry and register all providers
    println!("Registering platform providers...");
    let mut platform_registry = PlatformRegistry::new();
    
    // Register YouTube provider with bundled executables
    let youtube_provider = Arc::new(YouTubeProvider::with_executables(ytdlp_path, ffmpeg_path));
    
    // Log versions at startup
    let provider_clone = Arc::clone(&youtube_provider);
    tauri::async_runtime::spawn(async move {
        provider_clone.log_versions().await;
    });
    
    platform_registry.register(youtube_provider);
    println!("  ✓ YouTube provider registered");
    
    // Future providers can be registered here:
    // platform_registry.register(Arc::new(BilibiliProvider::new()));
    
    let platform_registry = Arc::new(platform_registry);
    
    // Step 2: Initialize storage service
    println!("Initializing storage service...");
    let storage_service = Arc::new(
        StorageService::new(app_handle.clone())
            .expect("Failed to initialize storage service")
    );
    println!("  ✓ Storage service initialized");
    
    // Step 3: Load user settings
    println!("Loading user settings...");
    let settings = storage_service.load_settings()
        .unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load settings, using defaults: {}", e);
            storage::AppSettings::default()
        });
    println!("  ✓ Settings loaded");
    
    // Step 4: Initialize download manager
    println!("Initializing download manager...");
    let download_manager = Arc::new(DownloadManager::new(
        app_handle.clone(),
        Arc::clone(&platform_registry),
    ));
    
    // Set max concurrent downloads from settings
    let max_concurrent = settings.max_concurrent_downloads;
    let dm_clone = Arc::clone(&download_manager);
    tauri::async_runtime::spawn(async move {
        dm_clone.set_max_concurrent(max_concurrent).await;
    });
    println!("  ✓ Download manager initialized (max concurrent: {})", max_concurrent);
    
    // Step 5: Restore previous queue state
    println!("Restoring download queue...");
    let dm_clone = Arc::clone(&download_manager);
    tauri::async_runtime::spawn(async move {
        match dm_clone.restore_queue_state().await {
            Ok(_) => {
                let queue = dm_clone.get_queue_status().await;
                println!("  ✓ Queue restored ({} items)", queue.len());
            }
            Err(e) => {
                eprintln!("Warning: Failed to restore queue state: {}", e);
            }
        }
    });
    
    // Step 6: Initialize metadata cache
    println!("Initializing metadata cache...");
    let metadata_cache = Arc::new(platform::MetadataCache::with_default_ttl());
    println!("  ✓ Metadata cache initialized (TTL: 5 minutes)");
    
    // Step 7: Store state in Tauri's managed state
    app_handle.manage(AppState {
        platform_registry,
        download_manager,
        storage_service,
        metadata_cache,
    });
    
    println!("✓ Application initialization complete");
    
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // Initialize application state
            initialize_app(app.handle())?;
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::detect_platform,
            commands::get_supported_platforms,
            commands::get_video_info,
            commands::get_playlist_info,
            commands::get_channel_info,
            commands::add_to_download_queue,
            commands::pause_download,
            commands::resume_download,
            commands::cancel_download,
            commands::reorder_queue,
            commands::get_settings,
            commands::save_settings,
            commands::select_directory,
            commands::check_dependencies,
            commands::verify_bundled_executables,
            commands::check_homebrew_installed,
            commands::install_ytdlp_via_homebrew,
            commands::check_ytdlp_update,
            commands::update_ytdlp,
            commands::test_ytdlp,
            commands::get_dependency_versions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
