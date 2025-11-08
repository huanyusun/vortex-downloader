# 下载进度显示问题修复

## 问题描述
下载队列中的项目显示状态为 "Downloading"，但进度始终为 0.0%，速度和 ETA 都显示为 0。

## 根本原因
**yt-dlp 的进度输出被发送到 stdout，而不是 stderr**，但代码只监听了 stderr 来获取进度信息。

从日志可以看到：
```
[process_queue_loop] Queue check: 0 queued, 1 active, 3 max, has_work=false
[process_queue_loop] No work: queue.len()=1, active.len()=1
```

这表明下载任务已经启动（1 个活动下载），但进度更新没有被捕获。

## 修复内容

### 文件：`src-tauri/src/platform/youtube.rs`

#### 1. 修改进度输出监听源
**之前：** 监听 stderr
```rust
// Get stderr for progress monitoring
let stderr = child.stderr.take().ok_or_else(|| {
    DownloadError::DownloadFailed("Failed to capture yt-dlp stderr".to_string())
})?;

let reader = BufReader::new(stderr);
let mut lines = reader.lines();
```

**之后：** 监听 stdout（yt-dlp 使用 `--newline` 参数时会将进度输出到 stdout）
```rust
// Get stdout for progress monitoring (yt-dlp outputs progress to stdout with --newline)
let stdout = child.stdout.take().ok_or_else(|| {
    DownloadError::DownloadFailed("Failed to capture yt-dlp stdout".to_string())
})?;

// Also capture stderr for error messages
let stderr = child.stderr.take();

let reader = BufReader::new(stdout);
let mut lines = reader.lines();
```

#### 2. 添加调试日志
添加了详细的日志输出来帮助诊断进度解析问题：
```rust
// Parse progress from stdout
while let Ok(Some(line)) = lines.next_line().await {
    println!("[yt-dlp] {}", line);
    if let Some(progress) = self.parse_progress_line(&line) {
        println!("[yt-dlp] Progress: {:.1}% at {:.2} MB/s, ETA: {}s", 
                 progress.percentage, progress.speed / (1024.0 * 1024.0), progress.eta);
        progress_callback(progress);
    }
}
```

#### 3. 改进错误处理
在下载失败时，现在会读取 stderr 来获取详细的错误信息：
```rust
if !status.success() {
    // Try to read stderr for error details
    let error_msg = if let Some(stderr_pipe) = stderr {
        let stderr_reader = BufReader::new(stderr_pipe);
        let mut stderr_lines = stderr_reader.lines();
        let mut error_output = String::new();
        while let Ok(Some(line)) = stderr_lines.next_line().await {
            error_output.push_str(&line);
            error_output.push('\n');
        }
        if error_output.is_empty() {
            format!("yt-dlp exited with status: {}", status)
        } else {
            format!("yt-dlp error: {}", error_output)
        }
    } else {
        format!("yt-dlp exited with status: {}", status)
    };
    
    return Err(DownloadError::DownloadFailed(error_msg));
}
```

## 测试步骤

1. 重新启动应用（如果正在运行）
2. 添加一个视频到下载队列
3. 观察控制台输出，应该能看到：
   - `[yt-dlp]` 开头的原始输出行
   - `[yt-dlp] Progress:` 开头的解析后的进度信息
4. 在 UI 中，下载进度应该正常更新，显示：
   - 百分比进度
   - 下载速度
   - 预计剩余时间

## 预期结果

- ✅ 下载进度百分比应该从 0% 逐渐增加到 100%
- ✅ 下载速度应该显示实际的 MB/s 或 KB/s
- ✅ ETA（预计剩余时间）应该显示合理的时间估计
- ✅ 进度条应该平滑地填充

## 技术说明

yt-dlp 使用 `--newline` 参数时的行为：
- 进度信息输出到 **stdout**
- 错误和警告信息输出到 **stderr**
- 每个进度更新都在新的一行上

进度行格式示例：
```
[download]  45.8% of 123.45MiB at 1.23MiB/s ETA 00:42
```

解析逻辑会提取：
- 百分比：`45.8%`
- 总大小：`123.45MiB`
- 速度：`1.23MiB/s`
- ETA：`00:42`（42 秒）
