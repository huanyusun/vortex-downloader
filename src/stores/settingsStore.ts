// App Settings Store
import { create } from 'zustand';
import type { AppSettings, VideoQuality, VideoFormat } from '../types';

interface SettingsStore {
  settings: AppSettings;
  isLoading: boolean;
  setSettings: (settings: AppSettings) => void;
  updateSettings: (partial: Partial<AppSettings>) => void;
  setLoading: (loading: boolean) => void;
  resetToDefaults: () => void;
}

const defaultSettings: AppSettings = {
  defaultSavePath: '',
  defaultQuality: 'best' as VideoQuality,
  defaultFormat: 'mp4' as VideoFormat,
  maxConcurrentDownloads: 3,
  autoRetryOnFailure: true,
  maxRetryAttempts: 3,
  platformSettings: {},
  enabledPlatforms: ['YouTube'],
  firstLaunchCompleted: false
};

export const useSettingsStore = create<SettingsStore>((set) => ({
  settings: defaultSettings,
  isLoading: false,
  
  setSettings: (settings) => set({ settings }),
  
  updateSettings: (partial) =>
    set((state) => ({
      settings: { ...state.settings, ...partial }
    })),
  
  setLoading: (loading) => set({ isLoading: loading }),
  
  resetToDefaults: () => set({ settings: defaultSettings })
}));
