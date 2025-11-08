use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{sleep, Duration};
use tauri::{AppHandle, Manager};
use super::task::{DownloadItem, DownloadTask, DownloadStatus};
use super::throttle::ProgressThrottler;
use crate::platform::{PlatformRegistry, DownloadOptions, DownloadProgress};
use crate::error::{Result, DownloadError};

/// Download manager for handling queue and concurrent downloads
pub struct DownloadManager {
    queue: Arc<RwLock<Vec<DownloadItem>>>,
    active_downloads: Arc<Mutex<HashMap<String, Arc<DownloadTask>>>>,
    max_concurrent: Arc<RwLock<usize>>,
    app_handle: AppHandle,
    platform_registry: Arc<PlatformRegistry>,
    processing: Arc<Mutex<bool>>,
}

impl DownloadManager {
    pub fn new(app_handle: AppHandle, platform_registry: Arc<PlatformRegistry>) -> Self {
        Self {
            queue: Arc::new(RwLock::new(Vec::new())),
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
            max_concurrent: Arc::new(RwLock::new(3)),
            app_handle,
            platform_registry,
            processing: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Set maximum concurrent downloads
    pub async fn set_max_concurrent(&self, max: usize) {
        let mut max_concurrent = self.max_concurrent.write().await;
        *max_concurrent = max.max(1).min(5);
    }
    
    /// Add download tasks to queue
    pub async fn add_to_queue(&self, items: Vec<DownloadItem>) -> Result<()> {
        println!("[DownloadManager::add_to_queue] Adding {} items to queue", items.len());
        
        for (idx, item) in items.iter().enumerate() {
            println!("[DownloadManager::add_to_queue] Item {}: id={}, title={}, status={:?}, url={}", 
                     idx, item.id, item.title, item.status, item.url);
        }
        
        let mut queue = self.queue.write().await;
        queue.extend(items);
        println!("[DownloadManager::add_to_queue] Queue now has {} items", queue.len());
        drop(queue); // Release lock before emitting events
        
        // Emit queue update event
        self.emit_queue_update().await;
        
        // Start processing if not already running
        println!("[DownloadManager::add_to_queue] Starting processing...");
        self.start_processing().await;
        println!("[DownloadManager::add_to_queue] Processing started");
        
        Ok(())
    }
    
    /// Start queue processing loop
    async fn start_processing(&self) {
        let mut processing = self.processing.lock().await;
        if *processing {
            println!("[start_processing] Already processing, skipping");
            return;
        }
        println!("[start_processing] Starting new processing loop");
        *processing = true;
        drop(processing);
        
        let manager = self.clone_arc();
        tokio::spawn(async move {
            println!("[process_queue_loop] Spawned processing task");
            manager.process_queue_loop().await;
            println!("[process_queue_loop] Processing task completed");
        });
    }
    
    /// Process download queue in a loop
    async fn process_queue_loop(&self) {
        println!("[process_queue_loop] Starting queue processing loop");
        loop {
            // Check if there are items to process
            let has_work = {
                let queue = self.queue.read().await;
                let active = self.active_downloads.lock().await;
                let max_concurrent = *self.max_concurrent.read().await;
                
                let queued_count = queue.iter().filter(|item| item.status == DownloadStatus::Queued).count();
                let has_work = queued_count > 0 && active.len() < max_concurrent;
                
                println!("[process_queue_loop] Queue check: {} queued, {} active, {} max, has_work={}", 
                         queued_count, active.len(), max_concurrent, has_work);
                
                has_work
            };
            
            if !has_work {
                // Check if we should stop processing
                let queue = self.queue.read().await;
                let active = self.active_downloads.lock().await;
                
                println!("[process_queue_loop] No work: queue.len()={}, active.len()={}", 
                         queue.len(), active.len());
                
                if queue.is_empty() && active.is_empty() {
                    println!("[process_queue_loop] Queue and active both empty, stopping");
                    let mut processing = self.processing.lock().await;
                    *processing = false;
                    break;
                }
                
                // Wait before checking again
                println!("[process_queue_loop] Waiting 500ms before next check");
                sleep(Duration::from_millis(500)).await;
                continue;
            }
            
            // Process next item
            println!("[process_queue_loop] Processing next item");
            if let Err(e) = self.process_next_item().await {
                eprintln!("[process_queue_loop] Error processing queue item: {}", e);
            }
            
            // Small delay to prevent tight loop
            sleep(Duration::from_millis(100)).await;
        }
        println!("[process_queue_loop] Exiting processing loop");
    }
    
    /// Process next queued item
    async fn process_next_item(&self) -> Result<()> {
        // Find next queued item
        let item_to_download = {
            let mut queue = self.queue.write().await;
            let active = self.active_downloads.lock().await;
            let max_concurrent = *self.max_concurrent.read().await;
            
            println!("[process_next_item] Active downloads: {}/{}", active.len(), max_concurrent);
            
            if active.len() >= max_concurrent {
                println!("[process_next_item] Max concurrent downloads reached");
                return Ok(());
            }
            
            let queued_count = queue.iter().filter(|item| item.status == DownloadStatus::Queued).count();
            println!("[process_next_item] Found {} queued items", queued_count);
            
            queue.iter_mut()
                .find(|item| item.status == DownloadStatus::Queued)
                .map(|item| {
                    println!("[process_next_item] Starting download for: {} ({})", item.title, item.id);
                    item.status = DownloadStatus::Downloading;
                    item.clone()
                })
        };
        
        if let Some(item) = item_to_download {
            let task = Arc::new(DownloadTask::new(item.clone()));
            
            // Add to active downloads
            {
                let mut active = self.active_downloads.lock().await;
                active.insert(item.id.clone(), Arc::clone(&task));
            }
            
            // Emit status change
            self.emit_status_change(&item.id, DownloadStatus::Downloading).await;
            
            // Start download in background
            let manager = self.clone_arc();
            let item_id = item.id.clone();
            tokio::spawn(async move {
                if let Err(e) = manager.execute_download(task).await {
                    eprintln!("[execute_download] Download failed for {}: {}", item_id, e);
                }
            });
        } else {
            println!("[process_next_item] No queued items found to process");
        }
        
        Ok(())
    }
    
    /// Execute a download task
    async fn execute_download(&self, task: Arc<DownloadTask>) -> Result<()> {
        let item = &task.item;
        let item_id = item.id.clone();
        
        println!("[execute_download] Starting download for: {} ({})", item.title, item_id);
        println!("[execute_download] URL: {}", item.url);
        println!("[execute_download] Save path: {}", item.save_path);
        
        // Get platform provider
        let url = &item.url;
        
        let provider = self.platform_registry.detect_provider(url)
            .ok_or_else(|| {
                println!("[execute_download] Failed to detect platform for URL: {}", url);
                DownloadError::InvalidUrl("Unsupported platform".to_string())
            })?;
        
        println!("[execute_download] Detected platform: {}", provider.name());
        
        // Prepare download options
        let options = DownloadOptions {
            quality: "best".to_string(),
            format: "mp4".to_string(),
            audio_only: false,
        };
        
        let save_path = PathBuf::from(&item.save_path);
        
        // Create progress callback with throttling (500ms)
        let manager = self.clone_arc();
        let item_id_clone = item_id.clone();
        let throttler = Arc::new(ProgressThrottler::with_default_interval());
        let progress_callback = Box::new(move |progress: DownloadProgress| {
            let manager = manager.clone();
            let item_id = item_id_clone.clone();
            let throttler = Arc::clone(&throttler);
            tokio::spawn(async move {
                // Only update if throttle allows or if download is complete
                if throttler.should_update().await || progress.percentage >= 100.0 {
                    manager.update_progress(&item_id, progress).await;
                }
            });
        });
        
        println!("[execute_download] Starting download with provider: {}", provider.name());
        
        // Execute download with timeout (30 minutes for large videos)
        let timeout_duration = Duration::from_secs(30 * 60); // 30 minutes
        let download_future = provider.download_video(
            url,
            options,
            &save_path,
            progress_callback,
        );
        
        println!("[execute_download] Download timeout set to {} seconds", timeout_duration.as_secs());
        
        let result = tokio::time::timeout(timeout_duration, download_future).await;
        
        // Update status based on result
        match result {
            Ok(Ok(_)) => {
                println!("[execute_download] Download completed successfully: {}", item_id);
                if task.is_cancelled() {
                    println!("[execute_download] Download was cancelled: {}", item_id);
                    self.update_item_status(&item_id, DownloadStatus::Cancelled, None).await;
                } else {
                    self.update_item_status(&item_id, DownloadStatus::Completed, None).await;
                    self.emit_download_complete(&item_id).await;
                }
            }
            Ok(Err(e)) => {
                println!("[execute_download] Download failed for {}: {}", item_id, e);
                self.update_item_status(&item_id, DownloadStatus::Failed, Some(e.to_string())).await;
                self.emit_error(&item_id, &e.to_string()).await;
            }
            Err(_) => {
                let timeout_msg = format!(
                    "Download timed out after {} minutes. The video may be too large or the connection too slow. Please try again or check your network connection.",
                    timeout_duration.as_secs() / 60
                );
                println!("[execute_download] Download timed out for {}: {}", item_id, timeout_msg);
                self.update_item_status(&item_id, DownloadStatus::Failed, Some(timeout_msg.clone())).await;
                self.emit_error(&item_id, &timeout_msg).await;
            }
        }
        
        // Remove from active downloads
        {
            let mut active = self.active_downloads.lock().await;
            active.remove(&item_id);
            println!("[execute_download] Removed from active downloads: {}", item_id);
        }
        
        Ok(())
    }
    
    /// Update download progress
    async fn update_progress(&self, id: &str, progress: DownloadProgress) {
        let mut queue = self.queue.write().await;
        if let Some(item) = queue.iter_mut().find(|i| i.id == id) {
            item.progress = progress.percentage;
            item.speed = progress.speed;
            item.eta = progress.eta;
        }
        drop(queue);
        
        // Emit progress event
        let _ = self.app_handle.emit_all("download:progress", serde_json::json!({
            "id": id,
            "progress": progress,
        }));
    }
    
    /// Update item status
    async fn update_item_status(&self, id: &str, status: DownloadStatus, error: Option<String>) {
        let mut queue = self.queue.write().await;
        if let Some(item) = queue.iter_mut().find(|i| i.id == id) {
            item.status = status.clone();
            if let Some(err) = error {
                item.error = Some(err);
            }
        }
        drop(queue);
        
        self.emit_status_change(id, status).await;
        self.emit_queue_update().await;
    }
    
    /// Pause download
    pub async fn pause_download(&self, id: &str) -> Result<()> {
        // Cancel the active download
        {
            let active = self.active_downloads.lock().await;
            if let Some(task) = active.get(id) {
                task.cancel();
            }
        }
        
        // Update status
        self.update_item_status(id, DownloadStatus::Paused, None).await;
        
        Ok(())
    }
    
    /// Resume download
    pub async fn resume_download(&self, id: &str) -> Result<()> {
        // Update status to queued
        {
            let mut queue = self.queue.write().await;
            if let Some(item) = queue.iter_mut().find(|i| i.id == id) {
                if item.status == DownloadStatus::Paused {
                    item.status = DownloadStatus::Queued;
                    item.progress = 0.0;
                    item.speed = 0.0;
                    item.eta = 0;
                }
            }
        }
        
        self.emit_queue_update().await;
        
        // Start processing if not already running
        self.start_processing().await;
        
        Ok(())
    }
    
    /// Cancel download
    pub async fn cancel_download(&self, id: &str) -> Result<()> {
        // Cancel the active download
        {
            let active = self.active_downloads.lock().await;
            if let Some(task) = active.get(id) {
                task.cancel();
            }
        }
        
        // Update status
        self.update_item_status(id, DownloadStatus::Cancelled, None).await;
        
        Ok(())
    }
    
    /// Reorder queue
    pub async fn reorder_queue(&self, from_index: usize, to_index: usize) -> Result<()> {
        let mut queue = self.queue.write().await;
        if from_index < queue.len() && to_index < queue.len() {
            let item = queue.remove(from_index);
            queue.insert(to_index, item);
            drop(queue);
            self.emit_queue_update().await;
        }
        Ok(())
    }
    
    /// Get queue status
    pub async fn get_queue_status(&self) -> Vec<DownloadItem> {
        let queue = self.queue.read().await;
        queue.clone()
    }
    
    /// Save queue state to disk
    pub async fn save_queue_state(&self) -> Result<()> {
        let queue = self.queue.read().await;
        let app_dir = self.app_handle.path_resolver()
            .app_data_dir()
            .ok_or_else(|| DownloadError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find app data directory"
            )))?;
        
        tokio::fs::create_dir_all(&app_dir).await?;
        
        let queue_file = app_dir.join("queue.json");
        let json = serde_json::to_string_pretty(&*queue)?;
        tokio::fs::write(queue_file, json).await?;
        
        Ok(())
    }
    
    /// Restore queue state from disk
    pub async fn restore_queue_state(&self) -> Result<()> {
        let app_dir = self.app_handle.path_resolver()
            .app_data_dir()
            .ok_or_else(|| DownloadError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find app data directory"
            )))?;
        
        let queue_file = app_dir.join("queue.json");
        
        if !queue_file.exists() {
            return Ok(());
        }
        
        let json = tokio::fs::read_to_string(queue_file).await?;
        let mut items: Vec<DownloadItem> = serde_json::from_str(&json)?;
        
        // Reset downloading items to queued
        for item in &mut items {
            if item.status == DownloadStatus::Downloading {
                item.status = DownloadStatus::Queued;
                item.progress = 0.0;
                item.speed = 0.0;
                item.eta = 0;
            }
        }
        
        let mut queue = self.queue.write().await;
        *queue = items;
        drop(queue);
        
        self.emit_queue_update().await;
        
        // Start processing if there are queued items
        self.start_processing().await;
        
        Ok(())
    }
    
    /// Emit queue update event
    async fn emit_queue_update(&self) {
        let queue = self.get_queue_status().await;
        let _ = self.app_handle.emit_all("queue:update", queue);
    }
    
    /// Emit status change event
    async fn emit_status_change(&self, id: &str, status: DownloadStatus) {
        let _ = self.app_handle.emit_all("download:status_change", serde_json::json!({
            "id": id,
            "status": status,
        }));
    }
    
    /// Emit download complete event
    async fn emit_download_complete(&self, id: &str) {
        let _ = self.app_handle.emit_all("download:complete", serde_json::json!({
            "id": id,
        }));
    }
    
    /// Emit error event
    async fn emit_error(&self, id: &str, error: &str) {
        let _ = self.app_handle.emit_all("download:error", serde_json::json!({
            "id": id,
            "error": error,
        }));
    }
    
    /// Clone Arc references for spawning tasks
    fn clone_arc(&self) -> Arc<Self> {
        Arc::new(Self {
            queue: Arc::clone(&self.queue),
            active_downloads: Arc::clone(&self.active_downloads),
            max_concurrent: Arc::clone(&self.max_concurrent),
            app_handle: self.app_handle.clone(),
            platform_registry: Arc::clone(&self.platform_registry),
            processing: Arc::clone(&self.processing),
        })
    }
}
