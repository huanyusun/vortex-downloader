# State Management with Zustand

This directory contains all Zustand stores for the application.

## Stores

### 1. Download Store (`downloadStore.ts`)
Manages the download queue and download operations.

**State:**
- `queue`: Array of download items
- Operations: add, update progress, update status, reorder, clear

**Usage:**
```typescript
import { useDownloadStore } from './stores';

function MyComponent() {
  const { queue, addToQueue, updateDownloadProgress } = useDownloadStore();
  // Use the store...
}
```

### 2. Settings Store (`settingsStore.ts`)
Manages application settings and preferences.

**State:**
- `settings`: Application settings object
- `isLoading`: Loading state for settings operations

**Usage:**
```typescript
import { useSettingsStore } from './stores';

function MyComponent() {
  const { settings, setSettings, updateSettings } = useSettingsStore();
  // Use the store...
}
```

### 3. UI Store (`uiStore.ts`)
Manages UI state including current view, content preview, and loading states.

**State:**
- `currentView`: Current view mode
- `content`: Content preview data (video/playlist/channel)
- `isLoadingContent`: Loading state for content fetching
- `showSettings`: Settings panel visibility
- `selectedVideoIds`: Set of selected video IDs
- `error`: Current error message

**Usage:**
```typescript
import { useUIStore } from './stores';

function MyComponent() {
  const { content, setContent, isLoadingContent } = useUIStore();
  // Use the store...
}
```

## Hooks

Instead of using stores directly, it's recommended to use the provided hooks that integrate with Tauri API:

### 1. `useDownloadQueue()`
Integrates download store with Tauri backend API and event listeners.

```typescript
import { useDownloadQueue } from './hooks';

function MyComponent() {
  const { queue, addToQueue, pauseDownload, resumeDownload } = useDownloadQueue();
  
  const handleAdd = async (videos) => {
    await addToQueue(videos, '/path/to/save');
  };
}
```

### 2. `useSettings()`
Integrates settings store with Tauri backend API.

```typescript
import { useSettings } from './hooks';

function MyComponent() {
  const { settings, saveSettings, selectDirectory } = useSettings();
  
  const handleSave = async (newSettings) => {
    await saveSettings(newSettings);
  };
}
```

### 3. `useContentFetcher()`
Integrates UI store with content fetching API.

```typescript
import { useContentFetcher } from './hooks';

function MyComponent() {
  const { content, isLoading, fetchContentInfo } = useContentFetcher();
  
  const handleFetch = async (url) => {
    await fetchContentInfo(url);
  };
}
```

### 4. `useAppInitialization()`
Initializes the application by loading settings and setting up event listeners.

```typescript
import { useAppInitialization } from './hooks';

function App() {
  const { isInitialized } = useAppInitialization();
  
  if (!isInitialized) {
    return <LoadingScreen />;
  }
  
  return <MainApp />;
}
```

## Architecture

```
┌─────────────────────────────────────────┐
│           React Components              │
│  (Use hooks, not stores directly)       │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│              Hooks Layer                │
│  - useDownloadQueue()                   │
│  - useSettings()                        │
│  - useContentFetcher()                  │
│  - useAppInitialization()               │
└─────────────────┬───────────────────────┘
                  │
        ┌─────────┴─────────┐
        │                   │
┌───────▼────────┐  ┌───────▼────────┐
│  Zustand Store │  │   Tauri API    │
│  - Download    │  │  - Commands    │
│  - Settings    │  │  - Events      │
│  - UI          │  │                │
└────────────────┘  └────────────────┘
```

## Best Practices

1. **Use hooks instead of stores directly**: Hooks provide automatic synchronization with the backend.

2. **Event listeners are automatic**: The `useDownloadQueue()` hook automatically sets up event listeners for download progress, status changes, and errors.

3. **Optimistic updates**: Some operations (like reordering) update the local state immediately for better UX, then sync with the backend.

4. **Error handling**: All hooks throw errors that should be caught by the calling component.

5. **Initialization**: Call `useAppInitialization()` once at the app root level to ensure proper setup.
