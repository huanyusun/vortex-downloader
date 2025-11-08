// Hook for download queue operations with Tauri API integration
import { useEffect, useCallback } from 'react';
import { useDownloadStore } from '../stores';
import {
  addToDownloadQueue as apiAddToQueue,
  pauseDownload as apiPauseDownload,
  resumeDownload as apiResumeDownload,
  cancelDownload as apiCancelDownload,
  reorderQueue as apiReorderQueue,
  listenDownloadProgress,
  listenDownloadStatusChange,
  listenDownloadError,
  listenQueueUpdate
} from '../api/tauri';
import type { DownloadItem, VideoInfo } from '../types';

export function useDownloadQueue() {
  const {
    queue,
    setQueue,
    addToQueue: addToStoreQueue,
    updateDownloadProgress,
    updateDownloadStatus,
    updateDownloadError,
    removeFromQueue,
    clearCompleted,
    reorderQueue: reorderStoreQueue
  } = useDownloadStore();

  // Setup event listeners for real-time updates
  useEffect(() => {
    const unlistenProgress = listenDownloadProgress((data) => {
      updateDownloadProgress(data.id, data.progress);
    });

    const unlistenStatus = listenDownloadStatusChange((data) => {
      updateDownloadStatus(data.id, data.status as any);
    });

    const unlistenError = listenDownloadError((data) => {
      updateDownloadError(data.id, data.error);
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
  }, [updateDownloadProgress, updateDownloadStatus, updateDownloadError, setQueue]);

  // Add videos to download queue
  const addToQueue = useCallback(
    async (videos: VideoInfo[], savePath: string) => {
      console.log('[useDownloadQueue] Creating download items for', videos.length, 'videos');
      console.log('[useDownloadQueue] Save path:', savePath);
      
      const items: DownloadItem[] = videos.map((video) => ({
        id: `download-${Date.now()}-${video.id}`,
        videoId: video.id,
        title: video.title,
        thumbnail: video.thumbnail,
        status: 'queued' as const,
        progress: 0,
        speed: 0,
        eta: 0,
        savePath,
        url: video.url,
        platform: video.platform || 'youtube'
      }));

      console.log('[useDownloadQueue] Created items:', items);

      // Add to local store first for immediate UI update
      addToStoreQueue(items);

      // Then sync with backend
      try {
        console.log('[useDownloadQueue] Calling backend API with items:', items);
        await apiAddToQueue(items);
        console.log('[useDownloadQueue] Successfully added to backend queue');
      } catch (error) {
        console.error('[useDownloadQueue] Failed to add to backend queue:', error);
        // If backend fails, remove from local store
        items.forEach((item) => removeFromQueue(item.id));
        throw error;
      }
    },
    [addToStoreQueue, removeFromQueue]
  );

  // Pause download
  const pauseDownload = useCallback(async (id: string) => {
    await apiPauseDownload(id);
  }, []);

  // Resume download
  const resumeDownload = useCallback(async (id: string) => {
    await apiResumeDownload(id);
  }, []);

  // Cancel download
  const cancelDownload = useCallback(async (id: string) => {
    await apiCancelDownload(id);
  }, []);

  // Reorder queue
  const reorderQueue = useCallback(
    async (fromIndex: number, toIndex: number) => {
      // Optimistically update local state
      reorderStoreQueue(fromIndex, toIndex);

      try {
        await apiReorderQueue(fromIndex, toIndex);
      } catch (error) {
        // Revert on error (re-fetch from backend)
        throw error;
      }
    },
    [reorderStoreQueue]
  );

  return {
    queue,
    addToQueue,
    pauseDownload,
    resumeDownload,
    cancelDownload,
    reorderQueue,
    clearCompleted
  };
}
