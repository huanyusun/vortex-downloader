use serde::{Deserialize, Serialize};
use tokio::sync::watch;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DownloadItem {
    pub id: String,
    pub video_id: String,
    pub title: String,
    pub thumbnail: String,
    pub status: DownloadStatus,
    pub progress: f64,
    pub speed: f64,
    pub eta: u64,
    pub save_path: String,
    pub error: Option<String>,
    pub url: String,
    pub platform: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

pub struct DownloadTask {
    pub item: DownloadItem,
    pub cancel_tx: watch::Sender<bool>,
    pub cancel_rx: watch::Receiver<bool>,
}

impl DownloadTask {
    pub fn new(item: DownloadItem) -> Self {
        let (cancel_tx, cancel_rx) = watch::channel(false);
        Self {
            item,
            cancel_tx,
            cancel_rx,
        }
    }
    
    pub fn is_cancelled(&self) -> bool {
        *self.cancel_rx.borrow()
    }
    
    pub fn cancel(&self) {
        let _ = self.cancel_tx.send(true);
    }
}
