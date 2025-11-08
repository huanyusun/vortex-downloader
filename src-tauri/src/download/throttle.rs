use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use crate::platform::DownloadProgress;

/// Throttles progress updates to prevent overwhelming the UI
pub struct ProgressThrottler {
    last_update: Arc<Mutex<Instant>>,
    min_interval: Duration,
}

impl ProgressThrottler {
    /// Create a new throttler with specified minimum interval
    pub fn new(min_interval: Duration) -> Self {
        Self {
            last_update: Arc::new(Mutex::new(Instant::now() - min_interval)),
            min_interval,
        }
    }
    
    /// Create a throttler with 500ms interval (recommended for UI updates)
    pub fn with_default_interval() -> Self {
        Self::new(Duration::from_millis(500))
    }
    
    /// Check if enough time has passed to send an update
    /// Returns true if the update should be sent
    pub async fn should_update(&self) -> bool {
        let mut last = self.last_update.lock().await;
        let now = Instant::now();
        
        if now.duration_since(*last) >= self.min_interval {
            *last = now;
            true
        } else {
            false
        }
    }
    
    /// Force an update regardless of throttle interval
    /// Useful for final progress updates (100%)
    pub async fn force_update(&self) {
        let mut last = self.last_update.lock().await;
        *last = Instant::now();
    }
    
    /// Call the progress callback only if throttle allows
    pub async fn throttled_call<F>(&self, progress: &DownloadProgress, callback: F)
    where
        F: FnOnce(&DownloadProgress),
    {
        if self.should_update().await || progress.percentage >= 100.0 {
            callback(progress);
        }
    }
}

impl Default for ProgressThrottler {
    fn default() -> Self {
        Self::with_default_interval()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_throttle_basic() {
        let throttler = ProgressThrottler::new(Duration::from_millis(100));
        
        // First update should always go through
        assert!(throttler.should_update().await);
        
        // Immediate second update should be throttled
        assert!(!throttler.should_update().await);
        
        // After waiting, update should go through
        sleep(Duration::from_millis(150)).await;
        assert!(throttler.should_update().await);
    }
    
    #[tokio::test]
    async fn test_force_update() {
        let throttler = ProgressThrottler::new(Duration::from_millis(100));
        
        throttler.should_update().await;
        
        // Force update should reset the timer
        throttler.force_update().await;
        
        // Next update should be throttled
        assert!(!throttler.should_update().await);
    }
}
