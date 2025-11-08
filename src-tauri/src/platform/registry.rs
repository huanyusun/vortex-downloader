use std::collections::HashMap;
use std::sync::Arc;
use super::provider::PlatformProvider;

/// Registry for managing platform providers
pub struct PlatformRegistry {
    providers: HashMap<String, Arc<dyn PlatformProvider>>,
}

impl PlatformRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }
    
    /// Register a new platform provider
    pub fn register(&mut self, provider: Arc<dyn PlatformProvider>) {
        let name = provider.name().to_string();
        self.providers.insert(name, provider);
    }
    
    /// Detect provider based on URL
    pub fn detect_provider(&self, url: &str) -> Option<Arc<dyn PlatformProvider>> {
        for provider in self.providers.values() {
            if provider.matches_url(url) {
                return Some(Arc::clone(provider));
            }
        }
        None
    }
    
    /// Get all registered providers
    pub fn get_all_providers(&self) -> Vec<Arc<dyn PlatformProvider>> {
        self.providers.values().map(Arc::clone).collect()
    }
    
    /// Get provider by name
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn PlatformProvider>> {
        self.providers.get(name).map(Arc::clone)
    }
}

impl Default for PlatformRegistry {
    fn default() -> Self {
        Self::new()
    }
}
