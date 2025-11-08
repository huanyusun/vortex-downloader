# Dependency Checking

## Overview

The application now includes dependency checking functionality to verify that required external tools (yt-dlp and ffmpeg) are installed on the system.

## Backend Implementation

### Tauri Command

A new Tauri command `check_dependencies` has been added to check platform dependencies:

```rust
#[tauri::command]
pub async fn check_dependencies(
    platform_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<Dependency>, String>
```

### Usage

#### Check all platform dependencies:
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const dependencies = await invoke('check_dependencies', {
  platformName: null
});
```

#### Check specific platform dependencies:
```typescript
const youtubeDeps = await invoke('check_dependencies', {
  platformName: 'YouTube'
});
```

### Response Format

The command returns an array of `Dependency` objects:

```typescript
interface Dependency {
  name: string;              // e.g., "yt-dlp", "ffmpeg"
  installed: boolean;        // true if installed
  version: string | null;    // version string if available
  install_instructions: string; // installation guide
}
```

### Example Response

```json
[
  {
    "name": "yt-dlp",
    "installed": true,
    "version": "2024.10.07",
    "install_instructions": "Install via Homebrew: brew install yt-dlp\nOr download from: https://github.com/yt-dlp/yt-dlp/releases"
  },
  {
    "name": "ffmpeg",
    "installed": true,
    "version": "8.0",
    "install_instructions": "Install via Homebrew: brew install ffmpeg\nOr download from: https://ffmpeg.org/download.html"
  }
]
```

## Frontend Integration

### Recommended UI Flow

1. **On Application Startup**: Check dependencies and show a warning if any are missing
2. **Settings Panel**: Display dependency status with install buttons
3. **Before Download**: Verify dependencies are installed before allowing downloads

### Example React Component

```typescript
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface Dependency {
  name: string;
  installed: boolean;
  version: string | null;
  install_instructions: string;
}

function DependencyChecker() {
  const [dependencies, setDependencies] = useState<Dependency[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    checkDependencies();
  }, []);

  const checkDependencies = async () => {
    try {
      const deps = await invoke<Dependency[]>('check_dependencies', {
        platformName: null
      });
      setDependencies(deps);
    } catch (error) {
      console.error('Failed to check dependencies:', error);
    } finally {
      setLoading(false);
    }
  };

  if (loading) return <div>Checking dependencies...</div>;

  const missingDeps = dependencies.filter(d => !d.installed);

  if (missingDeps.length === 0) {
    return <div>✅ All dependencies installed</div>;
  }

  return (
    <div>
      <h3>⚠️ Missing Dependencies</h3>
      {missingDeps.map(dep => (
        <div key={dep.name}>
          <h4>{dep.name}</h4>
          <pre>{dep.install_instructions}</pre>
        </div>
      ))}
    </div>
  );
}
```

## Testing

Run the integration test to verify dependency checking:

```bash
cargo test --manifest-path src-tauri/Cargo.toml test_youtube_provider_check_dependencies -- --nocapture
```

This will output the current status of yt-dlp and ffmpeg on your system.
