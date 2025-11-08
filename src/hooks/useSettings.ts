// Hook for settings management with Tauri API integration
import { useEffect, useCallback } from 'react';
import { useSettingsStore } from '../stores';
import {
  getSettings as apiGetSettings,
  saveSettings as apiSaveSettings,
  selectDirectory as apiSelectDirectory
} from '../api/tauri';
import type { AppSettings } from '../types';

export function useSettings() {
  const { settings, isLoading, setSettings, setLoading, resetToDefaults } =
    useSettingsStore();

  // Load settings on mount
  useEffect(() => {
    const loadSettings = async () => {
      setLoading(true);
      try {
        const loadedSettings = await apiGetSettings();
        setSettings(loadedSettings);
      } catch (error) {
        console.error('Failed to load settings:', error);
      } finally {
        setLoading(false);
      }
    };

    loadSettings();
  }, [setSettings, setLoading]);

  // Save settings
  const saveSettings = useCallback(
    async (newSettings: AppSettings) => {
      setLoading(true);
      try {
        await apiSaveSettings(newSettings);
        setSettings(newSettings);
      } catch (error) {
        console.error('Failed to save settings:', error);
        throw error;
      } finally {
        setLoading(false);
      }
    },
    [setSettings, setLoading]
  );

  // Update partial settings
  const updatePartialSettings = useCallback(
    async (partial: Partial<AppSettings>) => {
      const newSettings = { ...settings, ...partial };
      await saveSettings(newSettings);
    },
    [settings, saveSettings]
  );

  // Select directory
  const selectDirectory = useCallback(async () => {
    try {
      const path = await apiSelectDirectory();
      if (path) {
        await updatePartialSettings({ defaultSavePath: path });
        return path;
      }
      return null;
    } catch (error) {
      console.error('Failed to select directory:', error);
      throw error;
    }
  }, [updatePartialSettings]);

  return {
    settings,
    isLoading,
    saveSettings,
    updateSettings: updatePartialSettings,
    selectDirectory,
    resetToDefaults
  };
}
