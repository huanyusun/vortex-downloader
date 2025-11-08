// Type definitions for the application

export enum VideoQuality {
  BEST = 'best',
  HIGH_1080P = '1080p',
  MEDIUM_720P = '720p',
  LOW_480P = '480p'
}

export enum VideoFormat {
  MP4 = 'mp4',
  WEBM = 'webm',
  MKV = 'mkv'
}

export interface DownloadOptions {
  quality: VideoQuality;
  format: VideoFormat;
  audioOnly: boolean;
}

export type DownloadStatus = 'queued' | 'downloading' | 'paused' | 'completed' | 'failed' | 'cancelled';

export interface VideoInfo {
  id: string;
  title: string;
  description: string;
  thumbnail: string;
  duration: number;
  uploader: string;
  upload_date: string;
  view_count: number;
  platform: string;
  url: string;
}

export interface PlaylistInfo {
  id: string;
  title: string;
  description: string;
  uploader: string;
  video_count: number;
  videos: VideoInfo[];
  platform: string;
  url: string;
}

export interface ChannelInfo {
  id: string;
  name: string;
  description: string;
  playlists: PlaylistInfo[];
  all_videos: VideoInfo[];
  platform: string;
  url: string;
}

export interface DownloadProgress {
  percentage: number;
  downloaded_bytes: number;
  total_bytes: number;
  speed: number;
  eta: number;
}

export interface DownloadItem {
  id: string;
  videoId: string;
  title: string;
  thumbnail: string;
  status: DownloadStatus;
  progress: number;
  speed: number;
  eta: number;
  savePath: string;
  url: string;
  platform: string;
  error?: string;
}

export interface PlatformInfo {
  name: string;
  supported_patterns: string[];
}

export interface AppSettings {
  defaultSavePath: string;
  defaultQuality: VideoQuality;
  defaultFormat: VideoFormat;
  maxConcurrentDownloads: number;
  autoRetryOnFailure: boolean;
  maxRetryAttempts: number;
  platformSettings: Record<string, Record<string, any>>;
  enabledPlatforms: string[];
  firstLaunchCompleted: boolean;
}

export interface Dependency {
  name: string;
  installed: boolean;
  version: string | null;
  install_instructions: string;
}

export type ContentType = 'video' | 'playlist' | 'channel';

export interface VideoContent {
  type: 'video';
  data: VideoInfo;
}

export interface PlaylistContent {
  type: 'playlist';
  data: PlaylistInfo;
}

export interface ChannelContent {
  type: 'channel';
  data: ChannelInfo;
}

export type ContentData = VideoContent | PlaylistContent | ChannelContent;
