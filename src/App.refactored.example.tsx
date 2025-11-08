// Example of App.tsx refactored to use the new state management
// This file demonstrates how to use the new hooks and stores
// To use this, rename it to App.tsx and replace the existing file

import URLInputPanel from './components/URLInputPanel';
import VideoPreviewPanel from './components/VideoPreviewPanel';
import DownloadQueuePanel from './components/DownloadQueuePanel';
import SettingsPanel from './components/SettingsPanel';
import ToastContainer from './components/ToastContainer';
import { useToast } from './hooks/useToast';
import { useDownloadQueue } from './hooks/useDownloadQueue';
import { useSettings } from './hooks/useSettings';
import { useContentFetcher } from './hooks/useContentFetcher';
import { useAppInitialization } from './hooks/useAppInitialization';
import { useUIStore } from './stores';
import type { DownloadOptions, VideoInfo } from './types';

function App() {
  // Initialize app (loads settings, sets up event listeners)
  const { isInitialized } = useAppInitialization();

  // Use custom hooks for state management
  const { queue, addToQueue, pauseDownload, resumeDownload, cancelDownload, reorderQueue } =
    useDownloadQueue();
  const { settings, saveSettings } = useSettings();
  const { content, isLoading, fetchContentInfo, clearContent } = useContentFetcher();
  const { showSettings, openSettings, closeSettings } = useUIStore();
  const { toasts, closeToast, showSuccess, showError, showInfo } = useToast();

  // Show loading screen while initializing
  if (!isInitialized) {
    return (
      <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-white mx-auto mb-4"></div>
          <p>Initializing application...</p>
        </div>
      </div>
    );
  }

  const handleFetchInfo = async (url: string, options: DownloadOptions) => {
    try {
      await fetchContentInfo(url, options);
      if (content?.type === 'video') {
        showSuccess('Video Loaded', content.data.title);
      } else if (content?.type === 'playlist') {
        showInfo('Playlist Loaded', `Found ${content.data.video_count} videos`);
      } else if (content?.type === 'channel') {
        showInfo('Channel Loaded', `Found ${content.data.playlists.length} playlists`);
      }
    } catch (error: any) {
      showError('Failed to Load', error.message || 'Could not fetch video information');
    }
  };

  const handleAddToQueue = async (videos: VideoInfo[]) => {
    try {
      await addToQueue(videos, settings.defaultSavePath);
      showSuccess('Added to Queue', `${videos.length} video(s) added to download queue`);
      clearContent();
    } catch (error: any) {
      showError('Queue Error', error.message || 'Failed to add videos to queue');
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

  const handleSaveSettings = async (newSettings: typeof settings) => {
    try {
      await saveSettings(newSettings);
      showSuccess('Settings Saved', 'Your settings have been saved successfully');
      closeSettings();
    } catch (error: any) {
      showError('Save Failed', error.message || 'Failed to save settings');
      throw error;
    }
  };

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      {/* Header */}
      <header className="bg-gray-800 border-b border-gray-700 px-6 py-4">
        <div className="max-w-7xl mx-auto flex items-center justify-between">
          <h1 className="text-2xl font-bold">YouTube Downloader</h1>
          <button
            onClick={openSettings}
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
            {content && (
              <VideoPreviewPanel content={content} onAddToQueue={handleAddToQueue} />
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

      {/* Settings Modal */}
      {showSettings && (
        <SettingsPanel
          settings={settings}
          onSave={handleSaveSettings}
          onClose={closeSettings}
        />
      )}

      {/* Toast Notifications */}
      <ToastContainer toasts={toasts} onClose={closeToast} />
    </div>
  );
}

export default App;
