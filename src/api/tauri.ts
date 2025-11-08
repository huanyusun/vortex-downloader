// Tauri API wrapper functions
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import type {
  VideoInfo,
  PlaylistInfo,
  ChannelInfo,
  DownloadItem,
  DownloadProgress,
  AppSettings,
  PlatformInfo,
  Dependency
} from '../types';

// Platform detection
export async function detectPlatform(url: string): Promise<string> {
  return invoke('detect_platform', { url });
}

export async function getSupportedPlatforms(): Promise<PlatformInfo[]> {
  return invoke('get_supported_platforms');
}

// Content info retrieval
export async function getVideoInfo(url: string): Promise<VideoInfo> {
  return invoke('get_video_info', { url });
}

export async function getPlaylistInfo(url: string): Promise<PlaylistInfo> {
  return invoke('get_playlist_info', { url });
}

export async function getChannelInfo(url: string): Promise<ChannelInfo> {
  return invoke('get_channel_info', { url });
}

// Download management
export async function addToDownloadQueue(items: DownloadItem[]): Promise<void> {
  return invoke('add_to_download_queue', { items });
}

export async function pauseDownload(id: string): Promise<void> {
  return invoke('pause_download', { id });
}

export async function resumeDownload(id: string): Promise<void> {
  return invoke('resume_download', { id });
}

export async function cancelDownload(id: string): Promise<void> {
  return invoke('cancel_download', { id });
}

export async function reorderQueue(fromIndex: number, toIndex: number): Promise<void> {
  return invoke('reorder_queue', { fromIndex, toIndex });
}

// Settings
export async function getSettings(): Promise<AppSettings> {
  return invoke('get_settings');
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke('save_settings', { settings });
}

export async function selectDirectory(): Promise<string | null> {
  return invoke('select_directory');
}

// Dependency checking
export async function checkDependencies(platformName?: string): Promise<Dependency[]> {
  return invoke('check_dependencies', { platformName });
}

export async function verifyBundledExecutables(): Promise<boolean> {
  return invoke('verify_bundled_executables');
}

export async function checkHomebrewInstalled(): Promise<boolean> {
  return invoke('check_homebrew_installed');
}

export async function installYtdlpViaHomebrew(): Promise<void> {
  return invoke('install_ytdlp_via_homebrew');
}

// Event listeners
export function listenInstallProgress(
  callback: (message: string) => void
) {
  return listen('install:progress', (event) => callback(event.payload as string));
}
export function listenDownloadProgress(
  callback: (data: { id: string; progress: DownloadProgress }) => void
) {
  return listen('download:progress', (event) => callback(event.payload as any));
}

export function listenDownloadStatusChange(
  callback: (data: { id: string; status: string }) => void
) {
  return listen('download:status_change', (event) => callback(event.payload as any));
}

export function listenDownloadError(
  callback: (data: { id: string; error: string }) => void
) {
  return listen('download:error', (event) => callback(event.payload as any));
}

export function listenQueueUpdate(
  callback: (items: DownloadItem[]) => void
) {
  return listen('queue:update', (event) => callback(event.payload as any));
}
