import { useState, useEffect } from 'react';
import URLInputPanel from './components/URLInputPanel';
import VideoPreviewPanel from './components/VideoPreviewPanel';
import DownloadQueuePanel from './components/DownloadQueuePanel';
import SettingsPanel from './components/SettingsPanel';
import ToastContainer from './components/ToastContainer';
import WelcomeWizard from './components/WelcomeWizard';
import ErrorDetailsModal from './components/ErrorDetailsModal';
import SuccessFeedback from './components/SuccessFeedback';
import { useToast } from './hooks/useToast';
import { executeBatchOperation, formatBatchResultMessage, categorizeErrors, type ErrorCategory } from './utils/batchOperations';
import {
  getVideoInfo,
  getPlaylistInfo,
  getChannelInfo,
  addToDownloadQueue,
  pauseDownload,
  resumeDownload,
  cancelDownload,
  reorderQueue,
  getSettings,
  saveSettings,
  listenDownloadProgress,
  listenDownloadStatusChange,
  listenDownloadError,
  listenQueueUpdate
} from './api/tauri';
import type {
  ContentData,
  DownloadOptions,
  VideoInfo,
  DownloadItem,
  AppSettings,
  VideoQuality,
  VideoFormat
} from './types';

function App() {
  const [content, setContent] = useState<ContentData | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [queue, setQueue] = useState<DownloadItem[]>([]);
  const [settings, setSettings] = useState<AppSettings>({
    defaultSavePath: '',
    defaultQuality: 'best' as VideoQuality,
    defaultFormat: 'mp4' as VideoFormat,
    maxConcurrentDownloads: 3,
    autoRetryOnFailure: true,
    maxRetryAttempts: 3,
    platformSettings: {},
    enabledPlatforms: ['YouTube'],
    firstLaunchCompleted: false
  });
  const [showSettings, setShowSettings] = useState(false);
  const [showWelcome, setShowWelcome] = useState(false);
  const [showErrorDetails, setShowErrorDetails] = useState(false);
  const [errorCategories, setErrorCategories] = useState<ErrorCategory[]>([]);
  const [failedVideos, setFailedVideos] = useState<VideoInfo[]>([]);
  const [showSuccessFeedback, setShowSuccessFeedback] = useState(false);
  const [successCount, setSuccessCount] = useState(0);
  const [successMessage, setSuccessMessage] = useState('');
  const { toasts, closeToast, showSuccess, showError, showInfo, showWarning } = useToast();

  // Load settings on mount
  useEffect(() => {
    const loadSettings = async () => {
      try {
        console.log('[App] Loading settings...');
        const loadedSettings = await getSettings();
        console.log('[App] Loaded settings:', loadedSettings);
        console.log('[App] firstLaunchCompleted:', loadedSettings.firstLaunchCompleted);
        setSettings(loadedSettings);
        
        // Show welcome wizard if this is the first launch
        if (!loadedSettings.firstLaunchCompleted) {
          console.log('[App] First launch detected, showing welcome wizard');
          setShowWelcome(true);
        } else {
          console.log('[App] Not first launch, skipping welcome wizard');
        }
      } catch (error) {
        console.error('Failed to load settings:', error);
        showError('Settings Error', 'Failed to load application settings');
      }
    };
    
    loadSettings();
  }, []);

  // Setup event listeners
  useEffect(() => {
    const unlistenProgress = listenDownloadProgress((data) => {
      setQueue((prev) =>
        prev.map((item) =>
          item.id === data.id
            ? {
                ...item,
                progress: data.progress.percentage,
                speed: data.progress.speed,
                eta: data.progress.eta
              }
            : item
        )
      );
    });

    const unlistenStatus = listenDownloadStatusChange((data) => {
      setQueue((prev) =>
        prev.map((item) =>
          item.id === data.id ? { ...item, status: data.status as any } : item
        )
      );
    });

    const unlistenError = listenDownloadError((data) => {
      setQueue((prev) =>
        prev.map((item) =>
          item.id === data.id
            ? { ...item, status: 'failed' as const, error: data.error }
            : item
        )
      );
      showError('Download Failed', data.error);
    });

    const unlistenQueue = listenQueueUpdate((items) => {
      setQueue(items);
    });

    return () => {
      unlistenProgress.then((fn) => fn());
      unlistenStatus.then((fn) => fn());
      unlistenError.then((fn) => fn());
      unlistenQueue.then((fn) => fn());
    };
  }, []);

  const handleFetchInfo = async (url: string, _options: DownloadOptions) => {
    setIsLoading(true);
    setContent(null);

    try {
      // Try to detect content type and fetch appropriate info
      if (url.includes('/playlist')) {
        const playlistInfo = await getPlaylistInfo(url);
        setContent({ type: 'playlist', data: playlistInfo });
        showInfo('Playlist Loaded', `Found ${playlistInfo.video_count} videos`);
      } else if (url.includes('/@') || url.includes('/channel/') || url.includes('/c/')) {
        const channelInfo = await getChannelInfo(url);
        setContent({ type: 'channel', data: channelInfo });
        showInfo('Channel Loaded', `Found ${channelInfo.playlists.length} playlists`);
      } else {
        const videoInfo = await getVideoInfo(url);
        setContent({ type: 'video', data: videoInfo });
        showSuccess('Video Loaded', videoInfo.title);
      }
    } catch (error: any) {
      showError('Failed to Load', error.message || 'Could not fetch video information');
      console.error('Failed to fetch info:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleAddToQueue = async (videos: VideoInfo[]) => {
    if (videos.length === 0) {
      showWarning('No Videos Selected', 'Please select at least one video to add to the queue');
      return;
    }

    // Show loading state
    showInfo('Adding to Queue', `Processing ${videos.length} video${videos.length !== 1 ? 's' : ''}...`);

    try {
      // Create download items
      const items: DownloadItem[] = videos.map((video) => ({
        id: `download-${Date.now()}-${Math.random()}-${video.id}`,
        videoId: video.id,
        title: video.title,
        thumbnail: video.thumbnail,
        status: 'queued' as const,
        progress: 0,
        speed: 0,
        eta: 0,
        savePath: settings.defaultSavePath,
        url: video.url,
        platform: video.platform
      }));

      // Optimistically update local state first
      setQueue((prev) => [...prev, ...items]);

      // Execute batch operation with error handling
      const result = await executeBatchOperation(
        items,
        async (item) => {
          // Add to backend queue (this might fail for individual items)
          await addToDownloadQueue([item]);
          return item;
        },
        { continueOnError: true, maxConcurrent: 3 }
      );

      // Remove failed items from local queue
      if (result.failureCount > 0) {
        const failedIds = new Set(result.failed.map(f => f.item.id));
        setQueue((prev) => prev.filter(item => !failedIds.has(item.id)));
      }

      // Format and display result
      const resultMessage = formatBatchResultMessage(result);
      
      if (resultMessage.type === 'success') {
        // Show animated success feedback
        setSuccessCount(result.successCount);
        setSuccessMessage(result.successCount === 1 ? 'Video Added' : 'Videos Added');
        setShowSuccessFeedback(true);
        
        // Also show toast notification
        showSuccess(resultMessage.title, resultMessage.message, true); // Enable system notification
        
        setContent(null);
        setFailedVideos([]);
        setErrorCategories([]);
      } else if (resultMessage.type === 'warning') {
        // Partial success - show warning with option to view details
        const failedVideosList = result.failed.map(f => {
          // Map back to VideoInfo for retry
          const item = f.item;
          return videos.find(v => v.id === item.videoId)!;
        }).filter(Boolean);
        const categories = categorizeErrors(result.failed, (item) => item.title);
        setErrorCategories(categories);
        setFailedVideos(failedVideosList);
        
        showWarning(
          resultMessage.title,
          resultMessage.message + ' Click to view details.',
        );
        
        // Don't clear content if there were failures (user might want to retry)
      } else {
        // Complete failure
        const failedVideosList = result.failed.map(f => {
          // Map back to VideoInfo for retry
          const item = f.item;
          return videos.find(v => v.id === item.videoId)!;
        }).filter(Boolean);
        const categories = categorizeErrors(result.failed, (item) => item.title);
        setErrorCategories(categories);
        setFailedVideos(failedVideosList);
        
        showError(
          resultMessage.title,
          resultMessage.message + ' Click to view details.',
        );
      }

      // Show error details modal if there were any failures
      if (result.failureCount > 0) {
        setShowErrorDetails(true);
      }

    } catch (error: any) {
      // Catastrophic failure (shouldn't happen with batch operation)
      showError('Queue Error', error.message || 'Failed to add videos to queue');
      console.error('Failed to add to queue:', error);
    }
  };

  const handleRetryFailedItems = async () => {
    setShowErrorDetails(false);
    if (failedVideos.length > 0) {
      await handleAddToQueue(failedVideos);
    }
  };

  const handlePause = async (id: string) => {
    try {
      await pauseDownload(id);
    } catch (error: any) {
      showError('Pause Failed', error.message || 'Failed to pause download');
    }
  };

  const handleResume = async (id: string) => {
    try {
      await resumeDownload(id);
    } catch (error: any) {
      showError('Resume Failed', error.message || 'Failed to resume download');
    }
  };

  const handleCancel = async (id: string) => {
    try {
      await cancelDownload(id);
      showInfo('Download Cancelled', 'Download has been cancelled');
    } catch (error: any) {
      showError('Cancel Failed', error.message || 'Failed to cancel download');
    }
  };

  const handleReorder = async (fromIndex: number, toIndex: number) => {
    try {
      await reorderQueue(fromIndex, toIndex);
    } catch (error: any) {
      showError('Reorder Failed', error.message || 'Failed to reorder queue');
    }
  };

  const handleSaveSettings = async (newSettings: AppSettings) => {
    try {
      await saveSettings(newSettings);
      setSettings(newSettings);
      showSuccess('Settings Saved', 'Your settings have been saved successfully');
      setShowSettings(false);
    } catch (error: any) {
      showError('Save Failed', error.message || 'Failed to save settings');
      throw error;
    }
  };

  const handleWelcomeComplete = (newSettings: AppSettings) => {
    console.log('[App] Welcome wizard completed with settings:', newSettings);
    console.log('[App] firstLaunchCompleted:', newSettings.firstLaunchCompleted);
    setSettings(newSettings);
    setShowWelcome(false);
    showSuccess('Setup Complete', 'Welcome to YouTube Downloader!');
  };

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      {/* Header */}
      <header className="bg-gray-800 border-b border-gray-700 px-6 py-4">
        <div className="max-w-7xl mx-auto flex items-center justify-between">
          <h1 className="text-2xl font-bold">YouTube Downloader</h1>
          <button
            onClick={() => setShowSettings(true)}
            className="p-2 hover:bg-gray-700 rounded-lg transition-colors"
            title="Settings"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
              />
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
              />
            </svg>
          </button>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-6 py-8">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Left Column */}
          <div className="space-y-6">
            <URLInputPanel onFetchInfo={handleFetchInfo} isLoading={isLoading} />
            {(content || isLoading) && (
              <VideoPreviewPanel 
                content={content} 
                onAddToQueue={handleAddToQueue}
                isLoading={isLoading}
              />
            )}
          </div>

          {/* Right Column */}
          <div>
            <DownloadQueuePanel
              queue={queue}
              onPause={handlePause}
              onResume={handleResume}
              onCancel={handleCancel}
              onReorder={handleReorder}
            />
          </div>
        </div>
      </main>

      {/* Welcome Wizard */}
      {showWelcome && (
        <WelcomeWizard
          settings={settings}
          onComplete={handleWelcomeComplete}
        />
      )}

      {/* Settings Modal */}
      {showSettings && (
        <SettingsPanel
          settings={settings}
          onSave={handleSaveSettings}
          onClose={() => setShowSettings(false)}
        />
      )}

      {/* Success Feedback */}
      {showSuccessFeedback && (
        <SuccessFeedback
          count={successCount}
          message={successMessage}
          onComplete={() => setShowSuccessFeedback(false)}
        />
      )}

      {/* Error Details Modal */}
      <ErrorDetailsModal
        isOpen={showErrorDetails}
        onClose={() => setShowErrorDetails(false)}
        errorCategories={errorCategories}
        onRetry={failedVideos.length > 0 ? handleRetryFailedItems : undefined}
      />

      {/* Toast Notifications */}
      <ToastContainer toasts={toasts} onClose={closeToast} />
    </div>
  );
}

export default App;
