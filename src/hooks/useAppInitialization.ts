// Hook for initializing the application and syncing state
import { useEffect } from 'react';
import { useSettings } from './useSettings';
import { useDownloadQueue } from './useDownloadQueue';

/**
 * Initialize the application by loading settings and setting up event listeners.
 * This hook should be called once at the app root level.
 */
export function useAppInitialization() {
  const { settings, isLoading: isLoadingSettings } = useSettings();
  const { queue } = useDownloadQueue();

  // Log initialization status
  useEffect(() => {
    if (!isLoadingSettings) {
      console.log('App initialized with settings:', settings);
      console.log('Download queue items:', queue.length);
    }
  }, [isLoadingSettings, settings, queue.length]);

  return {
    isInitialized: !isLoadingSettings,
    settings,
    queueCount: queue.length
  };
}
