// Hook for fetching video/playlist/channel content with Tauri API integration
import { useCallback } from 'react';
import { useUIStore } from '../stores';
import {
  getVideoInfo,
  getPlaylistInfo,
  getChannelInfo,
  detectPlatform
} from '../api/tauri';
import type { DownloadOptions } from '../types';

export function useContentFetcher() {
  const { content, isLoadingContent, setContent, setLoadingContent, setError, clearError } =
    useUIStore();

  // Fetch content info based on URL
  const fetchContentInfo = useCallback(
    async (url: string, _options?: DownloadOptions) => {
      setLoadingContent(true);
      setContent(null);
      clearError();

      try {
        // Detect platform first
        const platform = await detectPlatform(url);
        console.log('Detected platform:', platform);

        // Determine content type and fetch appropriate info
        if (url.includes('/playlist')) {
          const playlistInfo = await getPlaylistInfo(url);
          setContent({ type: 'playlist', data: playlistInfo });
        } else if (
          url.includes('/@') ||
          url.includes('/channel/') ||
          url.includes('/c/')
        ) {
          const channelInfo = await getChannelInfo(url);
          setContent({ type: 'channel', data: channelInfo });
        } else {
          const videoInfo = await getVideoInfo(url);
          setContent({ type: 'video', data: videoInfo });
        }
      } catch (error: any) {
        const errorMessage = error.message || 'Failed to fetch content information';
        setError(errorMessage);
        throw error;
      } finally {
        setLoadingContent(false);
      }
    },
    [setContent, setLoadingContent, setError, clearError]
  );

  // Clear current content
  const clearContent = useCallback(() => {
    setContent(null);
    clearError();
  }, [setContent, clearError]);

  return {
    content,
    isLoading: isLoadingContent,
    fetchContentInfo,
    clearContent
  };
}
