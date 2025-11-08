# Design Document - YouTube Downloader GUI

## Overview

本设计文档描述了一个基于 Tauri 的 macOS 视频下载器应用程序的架构和实现细节。该应用程序采用可扩展的插件架构，初期支持 YouTube，后续可轻松扩展支持其他视频平台（如 Bilibili、Vimeo 等）。应用使用 yt-dlp 作为下载引擎，React 作为 UI 框架，提供直观的图形界面来管理视频、播放列表和频道的下载。

### Technology Stack

- **Tauri**: 轻量级跨平台桌面应用框架（使用 Rust 后端）
- **React**: UI 组件库
- **TypeScript**: 前端类型安全的开发语言
- **Rust**: 后端核心逻辑语言
- **yt-dlp**: YouTube 下载核心引擎
- **Zustand**: 轻量级状态管理
- **Tailwind CSS**: UI 样式框架
- **tauri-plugin-store**: 持久化配置存储

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Frontend (WebView)                    │
│  ┌──────────────────────────────────────────────────┐  │
│  │              React UI Components                  │  │
│  │  - URL Input  - Queue Manager  - Settings        │  │
│  └──────────────────────────────────────────────────┘  │
│                         │                                │
│                         ↓                                │
│  ┌──────────────────────────────────────────────────┐  │
│  │            State Management (Zustand)             │  │
│  └──────────────────────────────────────────────────┘  │
│                         │                                │
│                         ↓ Tauri Commands                 │
└─────────────────────────┼───────────────────────────────┘
                          │
┌─────────────────────────┼───────────────────────────────┐
│                  Rust Backend (Tauri Core)               │
│  ┌──────────────────────────────────────────────────┐  │
│  │              Download Manager                     │  │
│  │  - Queue Management  - Concurrent Control        │  │
│  └──────────────────────────────────────────────────┘  │
│                         │                                │
│  ┌──────────────────────────────────────────────────┐  │
│  │           Platform Provider Registry              │  │
│  │  - Provider Discovery  - URL Routing             │  │
│  └──────────────────────────────────────────────────┘  │
│                         │                                │
│         ┌───────────────┼───────────────┐               │
│         ↓               ↓               ↓               │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐         │
│  │ YouTube  │    │ Bilibili │    │  Future  │         │
│  │ Provider │    │ Provider │    │ Providers│         │
│  └──────────┘    └──────────┘    └──────────┘         │
│         │               │               │               │
│         └───────────────┼───────────────┘               │
│                         ↓                                │
│  ┌──────────────────────────────────────────────────┐  │
│  │            Storage & Configuration                │  │
│  │  - Settings Store  - Queue Persistence           │  │
│  └──────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

### Process Communication

Tauri 的 Frontend 和 Backend 通过 Tauri Commands 进行通信：

- **Frontend → Backend**: 使用 `invoke()` 调用 Rust 命令
- **Backend → Frontend**: 使用 `emit()` 发送事件到前端

## Components and Interfaces

### 1. UI Components (Renderer Process)

#### URLInputPanel
负责接收和验证 YouTube URL 输入。

```typescript
interface URLInputPanelProps {
  onSubmit: (url: string, options: DownloadOptions) => void;
}

interface DownloadOptions {
  quality: VideoQuality;
  format: VideoFormat;
  audioOnly: boolean;
}

enum VideoQuality {
  BEST = 'best',
  HIGH_1080P = '1080p',
  MEDIUM_720P = '720p',
  LOW_480P = '480p'
}

enum VideoFormat {
  MP4 = 'mp4',
  WEBM = 'webm',
  MKV = 'mkv'
}
```

#### VideoPreviewPanel
显示视频、播放列表或频道的预览信息。

```typescript
interface VideoPreviewPanelProps {
  content: VideoContent | PlaylistContent | ChannelContent;
  onAddToQueue: (items: DownloadItem[]) => void;
}

interface VideoContent {
  type: 'video';
  id: string;
  title: string;
  thumbnail: string;
  duration: number;
  uploader: string;
}

interface PlaylistContent {
  type: 'playlist';
  id: string;
  title: string;
  videos: VideoContent[];
  totalCount: number;
}

interface ChannelContent {
  type: 'channel';
  id: string;
  name: string;
  playlists: PlaylistContent[];
  uploadsPlaylist: PlaylistContent;
}
```

#### DownloadQueuePanel
管理和显示下载队列。

```typescript
interface DownloadQueuePanelProps {
  queue: DownloadItem[];
  onPause: (id: string) => void;
  onResume: (id: string) => void;
  onCancel: (id: string) => void;
  onReorder: (fromIndex: number, toIndex: number) => void;
}

interface DownloadItem {
  id: string;
  videoId: string;
  title: string;
  thumbnail: string;
  status: DownloadStatus;
  progress: number;
  speed: number;
  eta: number;
  savePath: string;
  error?: string;
}

enum DownloadStatus {
  QUEUED = 'queued',
  DOWNLOADING = 'downloading',
  PAUSED = 'paused',
  COMPLETED = 'completed',
  FAILED = 'failed',
  CANCELLED = 'cancelled'
}
```

#### SettingsPanel
配置应用程序设置。

```typescript
interface SettingsPanelProps {
  settings: AppSettings;
  onSave: (settings: AppSettings) => void;
}

interface AppSettings {
  defaultSavePath: string;
  defaultQuality: VideoQuality;
  defaultFormat: VideoFormat;
  maxConcurrentDownloads: number;
  autoRetryOnFailure: boolean;
  maxRetryAttempts: number;
  platformSettings: Record<string, Record<string, any>>;  // 平台特定设置
  enabledPlatforms: string[];  // 启用的平台列表
}
```

### 2. Backend Services (Rust)

#### Platform Provider Trait
定义所有平台提供者必须实现的接口，确保扩展性。

```rust
#[async_trait]
pub trait PlatformProvider: Send + Sync {
    /// 获取平台名称
    fn name(&self) -> &str;
    
    /// 检查 URL 是否属于该平台
    fn matches_url(&self, url: &str) -> bool;
    
    /// 获取支持的 URL 模式（用于 UI 提示）
    fn supported_patterns(&self) -> Vec<String>;
    
    /// 获取视频信息
    async fn get_video_info(&self, url: &str) -> Result<VideoInfo>;
    
    /// 获取播放列表信息
    async fn get_playlist_info(&self, url: &str) -> Result<PlaylistInfo>;
    
    /// 获取频道信息
    async fn get_channel_info(&self, url: &str) -> Result<ChannelInfo>;
    
    /// 下载视频
    async fn download_video(
        &self,
        url: &str,
        options: DownloadOptions,
        save_path: &Path,
        progress_callback: Box<dyn Fn(DownloadProgress) + Send>
    ) -> Result<()>;
    
    /// 验证内置依赖的完整性
    async fn verify_bundled_dependencies(&self) -> Result<Vec<Dependency>>;
    
    /// 获取平台特定的设置选项
    fn get_platform_settings(&self) -> Vec<PlatformSetting>;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Dependency {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub install_instructions: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlatformSetting {
    pub key: String,
    pub label: String,
    pub setting_type: SettingType,
    pub default_value: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SettingType {
    Boolean,
    String,
    Number,
    Select { options: Vec<String> },
}
```

#### PlatformRegistry
管理所有平台提供者的注册和路由。

```rust
pub struct PlatformRegistry {
    providers: HashMap<String, Arc<dyn PlatformProvider>>,
}

impl PlatformRegistry {
    /// 注册新的平台提供者
    pub fn register(&mut self, provider: Arc<dyn PlatformProvider>);
    
    /// 根据 URL 自动检测并返回对应的平台提供者
    pub fn detect_provider(&self, url: &str) -> Option<Arc<dyn PlatformProvider>>;
    
    /// 获取所有已注册的平台
    pub fn get_all_providers(&self) -> Vec<&dyn PlatformProvider>;
    
    /// 根据名称获取平台提供者
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn PlatformProvider>>;
}
```

#### YouTubeProvider
YouTube 平台的具体实现（使用 yt-dlp）。

```rust
pub struct YouTubeProvider {
    ytdlp_path: PathBuf,
}

#[async_trait]
impl PlatformProvider for YouTubeProvider {
    fn name(&self) -> &str {
        "YouTube"
    }
    
    fn matches_url(&self, url: &str) -> bool {
        url.contains("youtube.com") || url.contains("youtu.be")
    }
    
    fn supported_patterns(&self) -> Vec<String> {
        vec![
            "https://www.youtube.com/watch?v=*".to_string(),
            "https://youtu.be/*".to_string(),
            "https://www.youtube.com/playlist?list=*".to_string(),
            "https://www.youtube.com/@*".to_string(),
            "https://www.youtube.com/channel/*".to_string(),
        ]
    }
    
    async fn get_video_info(&self, url: &str) -> Result<VideoInfo> {
        // 使用 yt-dlp 实现
    }
    
    async fn get_playlist_info(&self, url: &str) -> Result<PlaylistInfo> {
        // 使用 yt-dlp 实现
    }
    
    async fn get_channel_info(&self, url: &str) -> Result<ChannelInfo> {
        // 使用 yt-dlp 实现
    }
    
    async fn download_video(
        &self,
        url: &str,
        options: DownloadOptions,
        save_path: &Path,
        progress_callback: Box<dyn Fn(DownloadProgress) + Send>
    ) -> Result<()> {
        // 使用 yt-dlp 实现
    }
    
    async fn check_dependencies(&self) -> Result<Vec<Dependency>> {
        // 检查 yt-dlp 和 ffmpeg
    }
    
    fn get_platform_settings(&self) -> Vec<PlatformSetting> {
        vec![
            PlatformSetting {
                key: "youtube_prefer_av1".to_string(),
                label: "优先使用 AV1 编码".to_string(),
                setting_type: SettingType::Boolean,
                default_value: serde_json::json!(false),
            },
            PlatformSetting {
                key: "youtube_skip_ads".to_string(),
                label: "跳过赞助片段".to_string(),
                setting_type: SettingType::Boolean,
                default_value: serde_json::json!(true),
            },
        ]
    }
}

impl YouTubeProvider {
    /// 获取内置 yt-dlp 可执行文件路径
    pub fn get_bundled_ytdlp_path(&self) -> PathBuf;
    
    /// 获取内置 ffmpeg 可执行文件路径
    pub fn get_bundled_ffmpeg_path(&self) -> PathBuf;
    
    /// 验证可执行文件完整性
    pub async fn verify_executables(&self) -> Result<()>;
    
    /// 更新 yt-dlp 到最新版本
    pub async fn update_ytdlp(&self) -> Result<()>;
    
    /// 执行 yt-dlp 命令（使用内置可执行文件）
    async fn execute_ytdlp(&self, args: &[&str]) -> Result<String>;
}

#### BilibiliProvider (Future Extension Example)
Bilibili 平台的实现示例（未来扩展）。

```rust
pub struct BilibiliProvider {
    // 可能使用不同的下载工具或 API
}

#[async_trait]
impl PlatformProvider for BilibiliProvider {
    fn name(&self) -> &str {
        "Bilibili"
    }
    
    fn matches_url(&self, url: &str) -> bool {
        url.contains("bilibili.com")
    }
    
    fn supported_patterns(&self) -> Vec<String> {
        vec![
            "https://www.bilibili.com/video/*".to_string(),
            "https://space.bilibili.com/*".to_string(),
        ]
    }
    
    // 实现其他 trait 方法...
}
```

#### 通用数据模型

所有平台提供者共享的数据结构：

#[derive(Serialize, Deserialize, Clone)]
pub struct VideoInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub thumbnail: String,
    pub duration: u64,
    pub uploader: String,
    pub upload_date: String,
    pub view_count: u64,
    pub available_formats: Vec<FormatInfo>,
    pub platform: String,  // 标识来源平台
    pub url: String,       // 原始 URL
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlaylistInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub uploader: String,
    pub video_count: usize,
    pub videos: Vec<VideoInfo>,
    pub platform: String,  // 标识来源平台
    pub url: String,       // 原始 URL
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChannelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub playlists: Vec<PlaylistInfo>,
    pub all_videos: Vec<VideoInfo>,
    pub platform: String,  // 标识来源平台
    pub url: String,       // 原始 URL
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DownloadProgress {
    pub percentage: f64,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed: f64,
    pub eta: u64,
}
```

#### DownloadManager
管理下载队列和并发控制，支持多平台。

```rust
pub struct DownloadManager {
    queue: Arc<Mutex<Vec<DownloadItem>>>,
    active_downloads: Arc<Mutex<HashMap<String, DownloadTask>>>,
    max_concurrent: usize,
    app_handle: AppHandle,
    platform_registry: Arc<PlatformRegistry>,
}

impl DownloadManager {
    /// 添加下载任务到队列
    pub async fn add_to_queue(&self, items: Vec<DownloadItem>) -> Result<()>;
    
    /// 开始处理队列
    pub async fn process_queue(&self);
    
    /// 暂停下载
    pub async fn pause_download(&self, id: &str) -> Result<()>;
    
    /// 恢复下载
    pub async fn resume_download(&self, id: &str) -> Result<()>;
    
    /// 取消下载
    pub async fn cancel_download(&self, id: &str) -> Result<()>;
    
    /// 重新排序队列
    pub async fn reorder_queue(&self, from_index: usize, to_index: usize) -> Result<()>;
    
    /// 获取队列状态
    pub async fn get_queue_status(&self) -> Vec<DownloadItem>;
    
    /// 保存队列状态到磁盘
    pub async fn save_queue_state(&self) -> Result<()>;
    
    /// 从磁盘恢复队列状态
    pub async fn restore_queue_state(&self) -> Result<()>;
}

pub struct DownloadTask {
    item: DownloadItem,
    child_process: Option<Child>,
    cancel_token: CancellationToken,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlatformInfo {
    pub name: String,
    pub supported_patterns: Vec<String>,
    pub dependencies: Vec<Dependency>,
    pub settings: Vec<PlatformSetting>,
}
```

#### StorageService
管理文件系统操作和配置持久化。

```rust
pub struct StorageService {
    store: Arc<Store>,
}

impl StorageService {
    /// 创建目录结构
    pub async fn create_directory_structure(
        &self,
        base_path: &Path,
        channel_name: Option<&str>,
        playlist_name: Option<&str>
    ) -> Result<PathBuf>;
    
    /// 检查磁盘空间
    pub async fn check_disk_space(&self, path: &Path, required_bytes: u64) -> Result<bool>;
    
    /// 保存应用设置
    pub fn save_settings(&self, settings: &AppSettings) -> Result<()>;
    
    /// 加载应用设置
    pub fn load_settings(&self) -> Result<AppSettings>;
    
    /// 获取默认保存路径
    pub fn get_default_save_path(&self) -> PathBuf;
}
```

### 3. Tauri Commands

定义 Frontend 和 Backend 之间的通信接口。

```rust
// Tauri Commands (Frontend → Backend)

#[tauri::command]
async fn detect_platform(url: String, state: State<'_, AppState>) -> Result<String, String>;

#[tauri::command]
async fn get_supported_platforms(state: State<'_, AppState>) -> Result<Vec<PlatformInfo>, String>;

#[tauri::command]
async fn get_video_info(url: String, state: State<'_, AppState>) -> Result<VideoInfo, String>;

#[tauri::command]
async fn get_playlist_info(url: String, state: State<'_, AppState>) -> Result<PlaylistInfo, String>;

#[tauri::command]
async fn get_channel_info(url: String, state: State<'_, AppState>) -> Result<ChannelInfo, String>;

#[tauri::command]
async fn add_to_download_queue(
    items: Vec<DownloadItem>,
    state: State<'_, AppState>
) -> Result<(), String>;

#[tauri::command]
async fn pause_download(id: String, state: State<'_, AppState>) -> Result<(), String>;

#[tauri::command]
async fn resume_download(id: String, state: State<'_, AppState>) -> Result<(), String>;

#[tauri::command]
async fn cancel_download(id: String, state: State<'_, AppState>) -> Result<(), String>;

#[tauri::command]
async fn reorder_queue(
    from_index: usize,
    to_index: usize,
    state: State<'_, AppState>
) -> Result<(), String>;

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String>;

#[tauri::command]
async fn save_settings(
    settings: AppSettings,
    state: State<'_, AppState>
) -> Result<(), String>;

#[tauri::command]
async fn select_directory() -> Result<Option<String>, String>;

// Events (Backend → Frontend)
// 使用 app.emit() 发送事件
// - "download:progress": { id: String, progress: DownloadProgress }
// - "download:status_change": { id: String, status: DownloadStatus }
// - "download:error": { id: String, error: String }
// - "queue:update": Vec<DownloadItem>
```

Frontend TypeScript 类型定义：

```typescript
// 对应 Rust 的 Tauri Commands
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

export async function getVideoInfo(url: string): Promise<VideoInfo> {
  return invoke('get_video_info', { url });
}

export async function getPlaylistInfo(url: string): Promise<PlaylistInfo> {
  return invoke('get_playlist_info', { url });
}

// ... 其他命令

// 监听事件
export function listenDownloadProgress(
  callback: (data: { id: string; progress: DownloadProgress }) => void
) {
  return listen('download:progress', (event) => callback(event.payload));
}
```

## Data Models

### Database Schema

使用 tauri-plugin-store 存储配置和队列状态（JSON 格式）。

```rust
#[derive(Serialize, Deserialize)]
pub struct StoreSchema {
    pub settings: AppSettings,
    pub queue: QueueState,
    pub history: DownloadHistory,
}

#[derive(Serialize, Deserialize)]
pub struct QueueState {
    pub items: Vec<DownloadItem>,
    pub last_updated: String,
}

#[derive(Serialize, Deserialize)]
pub struct DownloadHistory {
    pub downloads: Vec<CompletedDownload>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CompletedDownload {
    pub id: String,
    pub video_id: String,
    pub title: String,
    pub completed_at: String,
    pub save_path: String,
    pub file_size: u64,
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ErrorType {
    NetworkError,
    VideoUnavailable,
    InsufficientSpace,
    InvalidUrl,
    YtdlpNotFound,
    DownloadFailed,
    PermissionDenied,
}

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Video unavailable: {0}")]
    VideoUnavailable(String),
    
    #[error("Insufficient disk space")]
    InsufficientSpace,
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    #[error("yt-dlp not found")]
    YtdlpNotFound,
    
    #[error("Download failed: {0}")]
    DownloadFailed(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}
```

### Error Handling Strategy

1. **网络错误**: 自动重试（可配置次数），显示网络状态提示
2. **视频不可用**: 标记为失败，显示具体原因（私有、删除、地区限制等）
3. **磁盘空间不足**: 下载前检查，不足时阻止下载并提示用户
4. **无效 URL**: 输入时验证，显示友好的错误提示
5. **可执行文件错误**: 启动时验证内置可执行文件，如损坏则提示重新安装应用
6. **权限错误**: 提示用户选择有写入权限的目录
7. **添加队列失败**: 
   - 显示具体失败原因（URL 无效、网络问题、视频不可用等）
   - 提供重试选项
   - 记录失败的 URL 供用户查看

### Enhanced User Feedback

1. **URL Processing**:
   ```typescript
   // 状态流程
   Idle → Validating → Fetching → Success/Error
   
   // UI 反馈
   - Validating: 显示 "验证 URL..." + spinner
   - Fetching: 显示 "获取视频信息..." + spinner
   - Success: 显示视频预览 + 成功提示
   - Error: 显示错误消息 + 重试按钮
   ```

2. **Queue Operations**:
   ```typescript
   // 添加到队列
   - 显示 "正在添加 X 个视频到队列..."
   - 成功: Toast "已添加 X 个视频到下载队列"
   - 失败: Toast "添加失败: [具体原因]" + 详情按钮
   
   // 批量操作反馈
   - 显示进度: "已添加 3/10 个视频..."
   - 部分失败: "成功添加 8 个，2 个失败" + 查看失败列表
   ```

3. **Download Progress**:
   - 实时更新进度条
   - 显示下载速度（MB/s）
   - 显示预计剩余时间
   - 显示当前状态（排队中、下载中、暂停、完成、失败）

### User Notifications

```typescript
interface Notification {
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message: string;
  duration?: number;
  action?: {
    label: string;
    callback: () => void;
  };
}
```

## Testing Strategy

### Unit Tests

- **YtDlpService**: 模拟 yt-dlp 命令输出，测试解析逻辑
- **DownloadManager**: 测试队列管理、并发控制、状态转换
- **StorageService**: 测试文件系统操作、配置持久化

### Integration Tests

- **IPC Communication**: 测试 Renderer 和 Main Process 之间的通信
- **Download Flow**: 测试完整的下载流程（从 URL 输入到文件保存）
- **Queue Persistence**: 测试队列状态的保存和恢复

### E2E Tests

使用 Tauri 的 WebDriver 测试框架进行端到端测试：
- 启动应用程序
- 输入 YouTube URL
- 验证视频信息显示
- 添加到下载队列
- 验证下载进度更新
- 验证文件保存成功

### Manual Testing Checklist

- [ ] macOS 版本兼容性测试（10.15+）
- [ ] 不同网络条件下的下载测试
- [ ] 大型播放列表（100+ 视频）性能测试
- [ ] 频道下载完整性测试
- [ ] 应用崩溃恢复测试
- [ ] 磁盘空间不足场景测试

## UI/UX Design Guidelines

### Layout and Alignment

1. **Grid System**: 使用 Tailwind 的 grid 和 flexbox 实现一致的布局
2. **Spacing**: 
   - 组件间距: 使用 `gap-4` (16px) 作为标准间距
   - 内边距: 使用 `p-4` 或 `p-6` 保持一致
   - 外边距: 避免使用 margin，优先使用 gap
3. **Text Alignment**:
   - 标签和输入框使用 `items-center` 垂直对齐
   - 多行文本使用 `items-start` 顶部对齐
   - 按钮文字使用 `flex items-center justify-center` 居中
4. **Responsive Design**: 
   - 最小窗口宽度: 800px
   - 最小窗口高度: 600px
   - 使用相对单位而非固定像素

### Color Scheme

```css
/* 主色调 - 蓝色系 */
--primary: #3B82F6;        /* Blue-500 */
--primary-hover: #2563EB;  /* Blue-600 */
--primary-light: #60A5FA;  /* Blue-400 */

/* 成功 - 绿色 */
--success: #10B981;        /* Green-500 */
--success-hover: #059669;  /* Green-600 */

/* 错误 - 红色 */
--error: #EF4444;          /* Red-500 */
--error-hover: #DC2626;    /* Red-600 */

/* 警告 - 黄色 */
--warning: #F59E0B;        /* Amber-500 */

/* 背景色 - 深色主题 */
--bg-primary: #1F2937;     /* Gray-800 */
--bg-secondary: #374151;   /* Gray-700 */
--bg-tertiary: #4B5563;    /* Gray-600 */

/* 文字颜色 */
--text-primary: #F9FAFB;   /* Gray-50 */
--text-secondary: #D1D5DB; /* Gray-300 */
--text-muted: #9CA3AF;     /* Gray-400 */
```

### Typography

```css
/* 标题 */
--font-heading: 'SF Pro Display', system-ui, sans-serif;
--text-xl: 1.25rem;   /* 20px */
--text-lg: 1.125rem;  /* 18px */

/* 正文 */
--font-body: 'SF Pro Text', system-ui, sans-serif;
--text-base: 1rem;    /* 16px */
--text-sm: 0.875rem;  /* 14px */
--text-xs: 0.75rem;   /* 12px */

/* 行高 */
--leading-tight: 1.25;
--leading-normal: 1.5;
--leading-relaxed: 1.75;
```

### Component Styling

1. **Buttons**:
   ```tsx
   // Primary Button
   className="px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg 
              font-medium transition-colors duration-200 flex items-center 
              justify-center gap-2"
   
   // Secondary Button
   className="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-lg 
              font-medium transition-colors duration-200"
   ```

2. **Input Fields**:
   ```tsx
   className="w-full px-4 py-2 bg-gray-700 border border-gray-600 
              rounded-lg text-white placeholder-gray-400 
              focus:outline-none focus:ring-2 focus:ring-blue-500"
   ```

3. **Cards/Panels**:
   ```tsx
   className="bg-gray-800 rounded-xl p-6 shadow-lg"
   ```

4. **Progress Bars**:
   ```tsx
   // Container
   className="w-full h-2 bg-gray-700 rounded-full overflow-hidden"
   // Fill
   className="h-full bg-blue-500 transition-all duration-300"
   ```

### Error Handling UI

1. **Toast Notifications**:
   - 位置: 右上角
   - 持续时间: 成功 3s, 错误 5s, 警告 4s
   - 动画: 滑入/滑出效果
   - 最多同时显示 3 个

2. **Inline Errors**:
   ```tsx
   <div className="flex items-center gap-2 text-red-400 text-sm mt-2">
     <AlertCircle className="w-4 h-4" />
     <span>{errorMessage}</span>
   </div>
   ```

3. **Loading States**:
   - 使用 spinner 或 skeleton screens
   - 禁用相关按钮并显示加载文字
   - 显示进度百分比（如适用）

### Accessibility

1. **Keyboard Navigation**: 所有交互元素支持 Tab 键导航
2. **Focus Indicators**: 清晰的焦点指示器（ring-2 ring-blue-500）
3. **ARIA Labels**: 为图标按钮添加 aria-label
4. **Color Contrast**: 确保文字和背景对比度 ≥ 4.5:1

## Performance Considerations

1. **并发下载**: 默认 3 个并发，可配置 1-5 个
2. **内存管理**: 大型播放列表分页加载，避免一次性加载所有视频信息
3. **缓存策略**: 缓存视频元数据，减少重复请求
4. **进度更新**: 节流更新频率（每 500ms），避免 UI 卡顿
5. **文件写入**: 使用流式写入，支持大文件下载

## Security Considerations

1. **URL 验证**: 严格验证 YouTube URL 格式，防止命令注入
2. **路径安全**: 验证保存路径，防止路径遍历攻击
3. **进程隔离**: 利用 Electron 的进程隔离机制
4. **自动更新**: 实现安全的应用自动更新机制
5. **用户数据**: 不收集或上传用户下载历史

## Extensibility

### 添加新平台支持

应用采用插件化架构，添加新平台支持只需以下步骤：

1. **实现 PlatformProvider Trait**
   ```rust
   pub struct NewPlatformProvider {
       // 平台特定的配置
   }
   
   #[async_trait]
   impl PlatformProvider for NewPlatformProvider {
       // 实现所有必需的方法
   }
   ```

2. **注册到 PlatformRegistry**
   ```rust
   // 在 main.rs 或初始化代码中
   let mut registry = PlatformRegistry::new();
   registry.register(Arc::new(YouTubeProvider::new()));
   registry.register(Arc::new(NewPlatformProvider::new()));
   ```

3. **无需修改 UI 代码**
   - UI 自动检测支持的平台
   - 平台特定设置自动显示在设置面板
   - 下载流程自动路由到正确的提供者

### 平台提供者最佳实践

- **URL 匹配**: 使用精确的 URL 模式匹配，避免误判
- **错误处理**: 提供清晰的错误信息，包含平台特定的解决方案
- **进度报告**: 统一使用 DownloadProgress 结构报告进度
- **依赖检查**: 在首次使用时检查必需的依赖项
- **设置验证**: 验证平台特定设置的有效性

### 未来扩展示例

可能支持的平台：
- **Bilibili**: 中国视频平台
- **Vimeo**: 专业视频平台
- **Twitch**: 直播和 VOD
- **Twitter/X**: 社交媒体视频
- **Instagram**: 短视频和 Reels
- **TikTok**: 短视频平台

## Deployment

### Build Process

```bash
# 安装前端依赖
npm install

# 开发模式
npm run tauri dev

# 构建 macOS 应用
npm run tauri build -- --target universal-apple-darwin

# 生成 DMG 安装包（自动）
# Tauri 会自动生成 .app 和 .dmg 文件
```

### Distribution

- Tauri 自动打包为 .app 和 .dmg 文件
- 代码签名（需要 Apple Developer 账号）
- 公证（Notarization）以通过 macOS Gatekeeper
- 配置在 `tauri.conf.json` 中的 `bundle` 部分

### Dependencies

应用将内置所有必需的依赖：
- **yt-dlp**: 打包在应用程序内的二进制文件
- **ffmpeg**: 打包在应用程序内的二进制文件

#### Bundling Strategy

1. **macOS Universal Binary**: 
   - 为 x86_64 和 arm64 架构分别打包对应的 yt-dlp 和 ffmpeg
   - 使用 Tauri 的资源目录存储二进制文件
   - 路径: `resources/bin/{arch}/yt-dlp` 和 `resources/bin/{arch}/ffmpeg`

2. **Executable Management**:
   - 应用启动时检测系统架构
   - 从资源目录复制可执行文件到临时目录（如需要）
   - 设置正确的执行权限（chmod +x）
   - 验证文件完整性（SHA256 校验）

3. **Auto-Update for yt-dlp**:
   - 定期检查 yt-dlp 新版本（每周一次）
   - 后台下载更新并替换旧版本
   - 保持 ffmpeg 版本稳定（仅在必要时更新）

### First Launch Setup

1. 验证内置可执行文件完整性
2. 设置可执行权限
3. 引导用户设置默认保存路径
4. 显示快速入门教程
