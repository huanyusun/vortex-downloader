# Implementation Plan

- [x] 1. 初始化项目结构和核心接口
  - 创建 Tauri 项目基础结构
  - 设置 React + TypeScript 前端
  - 配置 Tailwind CSS
  - 定义 Rust 后端模块结构
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 2. 实现平台提供者抽象层
- [x] 2.1 定义 PlatformProvider trait 和相关数据结构
  - 创建 `src-tauri/src/platform/mod.rs` 定义 trait
  - 实现 VideoInfo, PlaylistInfo, ChannelInfo 数据模型
  - 定义 DownloadOptions 和 DownloadProgress 结构
  - 实现 Dependency 和 PlatformSetting 结构
  - _Requirements: 2.1, 3.1, 4.1_

- [x] 2.2 实现 PlatformRegistry
  - 创建提供者注册和管理逻辑
  - 实现 URL 自动检测功能
  - 实现提供者查询接口
  - _Requirements: 2.1, 3.1, 4.1_

- [ ] 3. 实现 YouTube 平台提供者
- [x] 3.1 创建 YouTubeProvider 基础结构
  - 实现 PlatformProvider trait
  - 实现 URL 匹配逻辑
  - 定义 YouTube 特定的设置选项
  - _Requirements: 2.1, 2.2, 3.1, 4.1_

- [x] 3.2 实现 yt-dlp 集成
  - 实现 yt-dlp 命令执行封装
  - 实现视频信息提取（JSON 解析）
  - 实现播放列表信息提取
  - 实现频道信息提取（包括播放列表分组）
  - _Requirements: 2.1, 2.2, 3.1, 3.2, 4.1, 4.2, 4.3_

- [x] 3.3 实现下载功能
  - 实现视频下载逻辑
  - 实现进度解析和回调
  - 实现下载暂停/恢复机制
  - 实现下载取消功能
  - _Requirements: 2.3, 2.4, 5.2, 5.3, 5.4_

- [x] 3.4 实现依赖检查
  - 检查 yt-dlp 安装状态
  - 检查 ffmpeg 安装状态
  - 提供安装指引信息
  - _Requirements: 7.1, 7.2_

- [x] 4. 实现下载管理器
- [x] 4.1 创建 DownloadManager 核心逻辑
  - 实现下载队列数据结构
  - 实现并发控制逻辑（最大并发数）
  - 实现队列处理循环
  - _Requirements: 2.3, 5.1, 5.5_

- [x] 4.2 实现队列操作
  - 实现添加任务到队列
  - 实现暂停/恢复/取消操作
  - 实现队列重新排序
  - 实现队列状态查询
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 4.3 实现队列持久化
  - 实现队列状态保存到磁盘
  - 实现应用启动时恢复队列
  - 处理异常退出的队列恢复
  - _Requirements: 5.6, 5.7_

- [x] 5. 实现存储服务
- [x] 5.1 创建 StorageService
  - 实现目录结构创建（频道/播放列表）
  - 实现磁盘空间检查
  - 实现文件路径安全验证
  - _Requirements: 3.5, 4.5, 6.6, 7.3_

- [x] 5.2 实现配置持久化
  - 集成 tauri-plugin-store
  - 实现应用设置保存/加载
  - 实现平台特定设置管理
  - 实现下载历史记录
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [x] 6. 实现 Tauri Commands
- [x] 6.1 实现平台相关命令
  - `detect_platform`: 检测 URL 所属平台
  - `get_supported_platforms`: 获取所有支持的平台
  - `get_video_info`: 获取视频信息
  - `get_playlist_info`: 获取播放列表信息
  - `get_channel_info`: 获取频道信息
  - _Requirements: 2.1, 2.2, 3.1, 3.2, 4.1, 4.2_

- [x] 6.2 实现下载管理命令
  - `add_to_download_queue`: 添加下载任务
  - `pause_download`: 暂停下载
  - `resume_download`: 恢复下载
  - `cancel_download`: 取消下载
  - `reorder_queue`: 重新排序队列
  - _Requirements: 2.3, 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 6.3 实现设置和存储命令
  - `get_settings`: 获取应用设置
  - `save_settings`: 保存应用设置
  - `select_directory`: 选择保存目录
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [x] 6.4 实现事件发送
  - 实现下载进度事件发送
  - 实现状态变化事件发送
  - 实现错误事件发送
  - 实现队列更新事件发送
  - _Requirements: 2.4, 5.1, 7.1, 7.2_

- [x] 7. 实现错误处理
- [x] 7.1 定义错误类型
  - 创建 DownloadError 枚举
  - 实现错误类型转换
  - 实现错误序列化（用于前端）
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [x] 7.2 实现错误处理策略
  - 实现网络错误自动重试
  - 实现磁盘空间预检查
  - 实现 URL 验证
  - 实现友好的错误消息生成
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [x] 8. 实现前端 UI 组件
- [x] 8.1 创建 URL 输入面板
  - 实现 URL 输入框和验证
  - 实现平台自动检测显示
  - 实现下载选项选择（质量、格式）
  - 实现"获取信息"按钮
  - _Requirements: 2.1, 2.2, 6.2_

- [x] 8.2 创建视频预览面板
  - 实现单个视频信息展示
  - 实现播放列表视频列表展示
  - 实现频道结构树形展示
  - 实现视频选择功能（复选框）
  - 实现"添加到队列"按钮
  - _Requirements: 2.1, 3.1, 3.2, 4.1, 4.2, 4.3, 4.4_

- [x] 8.3 创建下载队列面板
  - 实现队列列表展示
  - 实现下载进度条和状态显示
  - 实现暂停/恢复/取消按钮
  - 实现拖拽排序功能
  - 实现下载速度和 ETA 显示
  - _Requirements: 2.4, 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 8.4 创建设置面板
  - 实现通用设置表单
  - 实现平台特定设置动态渲染
  - 实现目录选择器
  - 实现设置保存和重置
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6_

- [x] 8.5 创建通知系统
  - 实现 Toast 通知组件
  - 实现下载完成通知
  - 实现错误通知
  - 实现系统通知集成（macOS）
  - _Requirements: 2.5, 7.1, 7.2, 7.5_

- [x] 9. 实现状态管理
- [x] 9.1 设置 Zustand store
  - 创建下载队列 store
  - 创建应用设置 store
  - 创建 UI 状态 store（当前视图、加载状态等）
  - _Requirements: 所有 UI 相关需求_

- [x] 9.2 实现 Tauri API 集成
  - 创建 API 调用封装函数
  - 实现事件监听器设置
  - 实现自动状态同步
  - _Requirements: 所有 UI 相关需求_

- [x] 10. 实现首次启动流程
- [x] 10.1 创建欢迎向导
  - 实现依赖检查界面
  - 实现 yt-dlp 安装指引
  - 实现默认保存路径设置
  - 实现快速入门教程
  - _Requirements: 1.1, 1.2, 1.3, 6.2_

- [x] 10.2 实现依赖自动安装（可选）
  - 检测 Homebrew 是否安装
  - 提供一键安装 yt-dlp 选项
  - 显示安装进度
  - _Requirements: 1.1, 1.2_

- [x] 11. 集成和优化
- [x] 11.1 实现应用初始化流程
  - 注册所有平台提供者
  - 初始化 DownloadManager
  - 恢复上次的队列状态
  - 加载用户设置
  - _Requirements: 5.6, 5.7, 6.5_

- [x] 11.2 性能优化
  - 实现大型播放列表分页加载
  - 实现视频元数据缓存
  - 实现进度更新节流（500ms）
  - 优化内存使用
  - _Requirements: 3.2, 4.2_

- [x] 11.3 实现应用打包配置
  - 配置 tauri.conf.json
  - 设置应用图标和元数据
  - 配置 macOS 代码签名
  - 配置 DMG 打包选项
  - _Requirements: 1.1, 1.2_

- [x] 12. 测试
- [x] 12.1 编写单元测试
  - 测试 PlatformRegistry URL 检测
  - 测试 DownloadManager 队列逻辑
  - 测试 StorageService 路径处理
  - 测试错误处理逻辑
  - _Requirements: 所有功能需求_

- [x] 12.2 编写集成测试
  - 测试 YouTube 信息提取
  - 测试完整下载流程
  - 测试队列持久化和恢复
  - 测试设置保存和加载
  - _Requirements: 所有功能需求_

- [x] 12.3 手动测试
  - 测试不同 macOS 版本兼容性
  - 测试大型播放列表性能
  - 测试网络异常场景
  - 测试应用崩溃恢复
  - _Requirements: 1.1, 3.2, 4.2, 5.6, 5.7, 7.1_

- [ ] 13. 内置依赖打包
- [x] 13.1 下载和准备可执行文件
  - 下载 yt-dlp 的 macOS x86_64 和 arm64 版本
  - 下载 ffmpeg 的 macOS x86_64 和 arm64 版本
  - 验证可执行文件的 SHA256 校验和
  - 组织文件到 resources/bin/{arch}/ 目录结构
  - _Requirements: 8.1, 8.2_

- [x] 13.2 配置 Tauri 资源打包
  - 更新 tauri.conf.json 的 bundle.resources 配置
  - 配置可执行文件的目标路径
  - 设置文件权限（executable）
  - _Requirements: 8.1, 8.2_

- [x] 13.3 实现可执行文件管理器
  - 创建 ExecutableManager 结构体
  - 实现系统架构检测（x86_64 vs arm64）
  - 实现从资源目录获取可执行文件路径
  - 实现可执行文件完整性验证（SHA256）
  - 实现设置执行权限的逻辑
  - _Requirements: 8.3, 8.5_

- [x] 13.4 更新 YouTubeProvider 使用内置可执行文件
  - 修改 YouTubeProvider 初始化逻辑
  - 使用 ExecutableManager 获取 yt-dlp 路径
  - 使用 ExecutableManager 获取 ffmpeg 路径
  - 更新所有 yt-dlp 命令调用使用内置路径
  - 移除系统 PATH 依赖检查
  - _Requirements: 8.1, 8.2, 8.3_

- [x] 13.5 实现 yt-dlp 自动更新
  - 创建更新检查服务
  - 实现版本比较逻辑
  - 实现后台下载新版本
  - 实现原子替换旧版本
  - 添加更新失败回滚机制
  - _Requirements: 8.4_

- [x] 14. UI 对齐和样式改进
- [x] 14.1 创建统一的设计系统
  - 创建 Tailwind 配置文件扩展主题
  - 定义颜色变量（primary, success, error, warning, backgrounds）
  - 定义字体和字号变量
  - 定义间距和圆角变量
  - 创建可复用的 CSS 类
  - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 14.2 重构 URLInputPanel 组件
  - 使用 flexbox/grid 确保标签和输入框对齐
  - 统一按钮样式和间距
  - 添加加载状态指示器
  - 改进平台检测显示样式
  - 确保响应式布局
  - _Requirements: 9.1, 9.2, 9.3, 10.3_

- [x] 14.3 重构 VideoPreviewPanel 组件
  - 改进视频卡片布局和对齐
  - 统一缩略图尺寸和比例
  - 改进文字截断和溢出处理
  - 添加悬停效果和过渡动画
  - 优化播放列表和频道的树形展示
  - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 14.4 重构 DownloadQueuePanel 组件
  - 改进队列项的布局和对齐
  - 统一进度条样式
  - 改进状态图标和颜色
  - 优化按钮组布局
  - 添加空状态提示
  - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 14.5 重构 SettingsPanel 组件
  - 使用表单布局确保标签和输入对齐
  - 统一表单元素样式
  - 改进分组和分隔线
  - 添加设置说明文字
  - 优化保存和重置按钮布局
  - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 14.6 改进 Toast 通知组件
  - 实现滑入/滑出动画
  - 添加图标（成功、错误、警告、信息）
  - 改进颜色和对比度
  - 实现多个通知的堆叠管理
  - 添加关闭按钮和自动消失
  - _Requirements: 9.4, 10.1_

- [x] 15. 增强错误处理和用户反馈
- [x] 15.1 改进 URL 验证和反馈
  - 实现客户端 URL 格式验证
  - 添加实时验证反馈
  - 显示支持的 URL 格式提示
  - 改进错误消息的清晰度
  - _Requirements: 10.1, 10.2, 10.5_

- [x] 15.2 改进添加队列的错误处理
  - 捕获所有可能的错误类型
  - 为每种错误类型提供具体的错误消息
  - 实现批量添加时的部分成功处理
  - 显示失败项的详细列表
  - 添加重试失败项的功能
  - _Requirements: 10.1, 10.2, 10.4_

- [x] 15.3 实现加载状态管理
  - 为所有异步操作添加加载状态
  - 实现 spinner 和 skeleton screens
  - 在加载时禁用相关按钮
  - 显示操作进度（如适用）
  - _Requirements: 10.3_

- [x] 15.4 改进成功反馈
  - 添加成功 Toast 通知
  - 显示操作结果统计（如"已添加 5 个视频"）
  - 添加视觉反馈动画
  - 更新 UI 状态反映操作结果
  - _Requirements: 10.1, 10.4_

- [x] 16. 更新欢迎向导
- [x] 16.1 简化依赖检查流程
  - 移除 yt-dlp 和 ffmpeg 的安装检查
  - 仅验证内置可执行文件完整性
  - 如验证失败，提示重新安装应用
  - 简化向导步骤
  - _Requirements: 8.1, 8.2, 8.3, 8.5_

- [x] 16.2 改进向导 UI
  - 使用新的设计系统样式
  - 改进步骤指示器
  - 优化文字对齐和间距
  - 添加过渡动画
  - _Requirements: 9.1, 9.2, 9.3, 9.4_
