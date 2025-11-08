use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use super::provider::{VideoInfo, PlaylistInfo, ChannelInfo};

/// Cache entry with expiration
#[derive(Clone)]
struct CacheEntry<T> {
    data: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            expires_at: Instant::now() + ttl,
        }
    }
    
    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// Metadata cache for videos, playlists, and channels
pub struct MetadataCache {
    video_cache: Arc<RwLock<HashMap<String, CacheEntry<VideoInfo>>>>,
    playlist_cache: Arc<RwLock<HashMap<String, CacheEntry<PlaylistInfo>>>>,
    channel_cache: Arc<RwLock<HashMap<String, CacheEntry<ChannelInfo>>>>,
    ttl: Duration,
}

impl MetadataCache {
    /// Create a new metadata cache with specified TTL
    pub fn new(ttl: Duration) -> Self {
        Self {
            video_cache: Arc::new(RwLock::new(HashMap::new())),
            playlist_cache: Arc::new(RwLock::new(HashMap::new())),
            channel_cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }
    
    /// Create a cache with default 5-minute TTL
    pub fn with_default_ttl() -> Self {
        Self::new(Duration::from_secs(300))
    }
    
    /// Get cached video info
    pub async fn get_video(&self, url: &str) -> Option<VideoInfo> {
        let cache = self.video_cache.read().await;
        cache.get(url).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }
    
    /// Cache video info
    pub async fn put_video(&self, url: String, info: VideoInfo) {
        let mut cache = self.video_cache.write().await;
        cache.insert(url, CacheEntry::new(info, self.ttl));
    }
    
    /// Get cached playlist info
    pub async fn get_playlist(&self, url: &str) -> Option<PlaylistInfo> {
        let cache = self.playlist_cache.read().await;
        cache.get(url).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }
    
    /// Cache playlist info
    pub async fn put_playlist(&self, url: String, info: PlaylistInfo) {
        let mut cache = self.playlist_cache.write().await;
        cache.insert(url, CacheEntry::new(info, self.ttl));
    }
    
    /// Get cached channel info
    pub async fn get_channel(&self, url: &str) -> Option<ChannelInfo> {
        let cache = self.channel_cache.read().await;
        cache.get(url).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }
    
    /// Cache channel info
    pub async fn put_channel(&self, url: String, info: ChannelInfo) {
        let mut cache = self.channel_cache.write().await;
        cache.insert(url, CacheEntry::new(info, self.ttl));
    }
    
    /// Clear all expired entries from all caches
    pub async fn cleanup_expired(&self) {
        // Clean video cache
        {
            let mut cache = self.video_cache.write().await;
            cache.retain(|_, entry| !entry.is_expired());
        }
        
        // Clean playlist cache
        {
            let mut cache = self.playlist_cache.write().await;
            cache.retain(|_, entry| !entry.is_expired());
        }
        
        // Clean channel cache
        {
            let mut cache = self.channel_cache.write().await;
            cache.retain(|_, entry| !entry.is_expired());
        }
    }
    
    /// Clear all caches
    pub async fn clear_all(&self) {
        self.video_cache.write().await.clear();
        self.playlist_cache.write().await.clear();
        self.channel_cache.write().await.clear();
    }
    
    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let video_count = self.video_cache.read().await.len();
        let playlist_count = self.playlist_cache.read().await.len();
        let channel_count = self.channel_cache.read().await.len();
        
        CacheStats {
            video_count,
            playlist_count,
            channel_count,
            total_count: video_count + playlist_count + channel_count,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub video_count: usize,
    pub playlist_count: usize,
    pub channel_count: usize,
    pub total_count: usize,
}

impl Default for MetadataCache {
    fn default() -> Self {
        Self::with_default_ttl()
    }
}
