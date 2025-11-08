import { useState, useEffect } from 'react';
import { selectDirectory } from '../api/tauri';
import type { AppSettings, VideoQuality, VideoFormat } from '../types';

interface SettingsPanelProps {
  settings: AppSettings;
  onSave: (settings: AppSettings) => void;
  onClose: () => void;
}

export default function SettingsPanel({ settings, onSave, onClose }: SettingsPanelProps) {
  const [formData, setFormData] = useState<AppSettings>(settings);
  const [hasChanges, setHasChanges] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    const changed = JSON.stringify(formData) !== JSON.stringify(settings);
    setHasChanges(changed);
  }, [formData, settings]);

  const handleSelectDirectory = async () => {
    try {
      const path = await selectDirectory();
      if (path) {
        setFormData({ ...formData, defaultSavePath: path });
      }
    } catch (error) {
      console.error('Failed to select directory:', error);
    }
  };

  const handleSave = async () => {
    setIsSaving(true);
    try {
      await onSave(formData);
      setHasChanges(false);
    } catch (error) {
      console.error('Failed to save settings:', error);
    } finally {
      setIsSaving(false);
    }
  };

  const handleReset = () => {
    setFormData(settings);
    setHasChanges(false);
  };

  const handlePlatformSettingChange = (platform: string, key: string, value: any) => {
    setFormData({
      ...formData,
      platformSettings: {
        ...formData.platformSettings,
        [platform]: {
          ...(formData.platformSettings[platform] || {}),
          [key]: value
        }
      }
    });
  };

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-gray-800 rounded-xl shadow-2xl w-full max-w-2xl max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-700">
          <div className="flex items-center gap-3">
            <svg className="w-6 h-6 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            <h2 className="text-2xl font-semibold">Settings</h2>
          </div>
          <button
            onClick={onClose}
            className="text-text-muted hover:text-white transition-colors p-1 rounded-lg hover:bg-gray-700"
            aria-label="Close settings"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6 space-y-8 custom-scrollbar">
          {/* General Settings */}
          <section>
            <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <svg className="w-5 h-5 text-primary" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M11.49 3.17c-.38-1.56-2.6-1.56-2.98 0a1.532 1.532 0 01-2.286.948c-1.372-.836-2.942.734-2.106 2.106.54.886.061 2.042-.947 2.287-1.561.379-1.561 2.6 0 2.978a1.532 1.532 0 01.947 2.287c-.836 1.372.734 2.942 2.106 2.106a1.532 1.532 0 012.287.947c.379 1.561 2.6 1.561 2.978 0a1.533 1.533 0 012.287-.947c1.372.836 2.942-.734 2.106-2.106a1.533 1.533 0 01.947-2.287c1.561-.379 1.561-2.6 0-2.978a1.532 1.532 0 01-.947-2.287c.836-1.372-.734-2.942-2.106-2.106a1.532 1.532 0 01-2.287-.947zM10 13a3 3 0 100-6 3 3 0 000 6z" clipRule="evenodd" />
              </svg>
              General Settings
            </h3>
            
            <div className="space-y-5">
              {/* Default Save Path */}
              <div className="space-y-2">
                <label className="form-label">
                  Default Save Location
                </label>
                <p className="text-xs text-text-muted">
                  Choose where downloaded videos will be saved
                </p>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={formData.defaultSavePath}
                    readOnly
                    className="input-field flex-1 cursor-default"
                    aria-label="Default save location"
                  />
                  <button
                    onClick={handleSelectDirectory}
                    className="btn-primary px-4 flex-shrink-0"
                    aria-label="Browse for directory"
                  >
                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                    </svg>
                    <span>Browse</span>
                  </button>
                </div>
              </div>

              {/* Default Quality */}
              <div className="space-y-2">
                <label htmlFor="defaultQuality" className="form-label">
                  Default Video Quality
                </label>
                <p className="text-xs text-text-muted">
                  Preferred video quality for downloads
                </p>
                <select
                  id="defaultQuality"
                  value={formData.defaultQuality}
                  onChange={(e) => setFormData({ ...formData, defaultQuality: e.target.value as VideoQuality })}
                  className="select-field"
                  aria-label="Default video quality"
                >
                  <option value="best">Best Quality</option>
                  <option value="1080p">1080p</option>
                  <option value="720p">720p</option>
                  <option value="480p">480p</option>
                </select>
              </div>

              {/* Default Format */}
              <div className="space-y-2">
                <label htmlFor="defaultFormat" className="form-label">
                  Default Video Format
                </label>
                <p className="text-xs text-text-muted">
                  Preferred container format for video files
                </p>
                <select
                  id="defaultFormat"
                  value={formData.defaultFormat}
                  onChange={(e) => setFormData({ ...formData, defaultFormat: e.target.value as VideoFormat })}
                  className="select-field"
                  aria-label="Default video format"
                >
                  <option value="mp4">MP4</option>
                  <option value="webm">WebM</option>
                  <option value="mkv">MKV</option>
                </select>
              </div>

              {/* Max Concurrent Downloads */}
              <div className="space-y-2">
                <label htmlFor="maxConcurrent" className="form-label">
                  Maximum Concurrent Downloads
                </label>
                <p className="text-xs text-text-muted">
                  Number of simultaneous downloads (1-5)
                </p>
                <input
                  id="maxConcurrent"
                  type="number"
                  min="1"
                  max="5"
                  value={formData.maxConcurrentDownloads}
                  onChange={(e) => setFormData({ ...formData, maxConcurrentDownloads: parseInt(e.target.value) })}
                  className="input-field"
                  aria-label="Maximum concurrent downloads"
                />
              </div>

              {/* Auto Retry */}
              <div className="flex items-start justify-between gap-4 p-4 bg-bg-secondary rounded-lg">
                <div className="flex-1">
                  <label className="text-sm font-medium text-gray-300 cursor-pointer">
                    Auto Retry on Failure
                  </label>
                  <p className="text-xs text-text-muted mt-1">
                    Automatically retry failed downloads
                  </p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer flex-shrink-0">
                  <input
                    type="checkbox"
                    checked={formData.autoRetryOnFailure}
                    onChange={(e) => setFormData({ ...formData, autoRetryOnFailure: e.target.checked })}
                    className="sr-only peer"
                    aria-label="Auto retry on failure"
                  />
                  <div className="w-11 h-6 bg-gray-600 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary/30 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary"></div>
                </label>
              </div>

              {/* Max Retry Attempts */}
              {formData.autoRetryOnFailure && (
                <div className="space-y-2 ml-4 pl-4 border-l-2 border-primary">
                  <label htmlFor="maxRetry" className="form-label">
                    Maximum Retry Attempts
                  </label>
                  <p className="text-xs text-text-muted">
                    Number of times to retry a failed download
                  </p>
                  <input
                    id="maxRetry"
                    type="number"
                    min="1"
                    max="10"
                    value={formData.maxRetryAttempts}
                    onChange={(e) => setFormData({ ...formData, maxRetryAttempts: parseInt(e.target.value) })}
                    className="input-field"
                    aria-label="Maximum retry attempts"
                  />
                </div>
              )}
            </div>
          </section>

          {/* Divider */}
          <div className="border-t border-gray-700"></div>

          {/* Platform-Specific Settings */}
          <section>
            <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <svg className="w-5 h-5 text-primary" fill="currentColor" viewBox="0 0 20 20">
                <path d="M2 6a2 2 0 012-2h6a2 2 0 012 2v8a2 2 0 01-2 2H4a2 2 0 01-2-2V6zM14.553 7.106A1 1 0 0014 8v4a1 1 0 00.553.894l2 1A1 1 0 0018 13V7a1 1 0 00-1.447-.894l-2 1z" />
              </svg>
              Platform Settings
            </h3>
            
            {/* YouTube Settings */}
            <div className="card-compact bg-bg-secondary space-y-4">
              <div className="flex items-center gap-2">
                <svg className="w-5 h-5 text-error" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M2 6a2 2 0 012-2h6a2 2 0 012 2v8a2 2 0 01-2 2H4a2 2 0 01-2-2V6zM14.553 7.106A1 1 0 0014 8v4a1 1 0 00.553.894l2 1A1 1 0 0018 13V7a1 1 0 00-1.447-.894l-2 1z" />
                </svg>
                <h4 className="font-medium text-white">YouTube</h4>
              </div>
              
              <div className="flex items-start justify-between gap-4 p-3 bg-gray-800 rounded-lg">
                <div className="flex-1">
                  <label className="text-sm text-gray-300 cursor-pointer">
                    Prefer AV1 Codec
                  </label>
                  <p className="text-xs text-text-muted mt-1">
                    Use AV1 encoding when available (better compression)
                  </p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer flex-shrink-0">
                  <input
                    type="checkbox"
                    checked={formData.platformSettings?.youtube?.prefer_av1 || false}
                    onChange={(e) => handlePlatformSettingChange('youtube', 'prefer_av1', e.target.checked)}
                    className="sr-only peer"
                    aria-label="Prefer AV1 codec"
                  />
                  <div className="w-11 h-6 bg-gray-600 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary/30 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary"></div>
                </label>
              </div>

              <div className="flex items-start justify-between gap-4 p-3 bg-gray-800 rounded-lg">
                <div className="flex-1">
                  <label className="text-sm text-gray-300 cursor-pointer">
                    Skip Sponsored Segments
                  </label>
                  <p className="text-xs text-text-muted mt-1">
                    Automatically skip sponsor segments using SponsorBlock
                  </p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer flex-shrink-0">
                  <input
                    type="checkbox"
                    checked={formData.platformSettings?.youtube?.skip_ads || true}
                    onChange={(e) => handlePlatformSettingChange('youtube', 'skip_ads', e.target.checked)}
                    className="sr-only peer"
                    aria-label="Skip sponsored segments"
                  />
                  <div className="w-11 h-6 bg-gray-600 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary/30 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary"></div>
                </label>
              </div>
            </div>
          </section>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 p-6 border-t border-gray-700 bg-bg-secondary">
          <button
            onClick={handleReset}
            disabled={!hasChanges || isSaving}
            className="btn-secondary disabled:opacity-50"
            aria-label="Reset changes"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            <span>Reset</span>
          </button>
          <button
            onClick={handleSave}
            disabled={!hasChanges || isSaving}
            className="btn-primary disabled:opacity-50"
            aria-label="Save changes"
          >
            {isSaving ? (
              <>
                <div className="spinner-sm"></div>
                <span>Saving...</span>
              </>
            ) : (
              <>
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
                <span>Save Changes</span>
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
