# Frontend UI Components

This directory contains all the React components for the YouTube Downloader GUI application.

## Components Overview

### Core Components

#### URLInputPanel (`components/URLInputPanel.tsx`)
- URL input with real-time validation
- Automatic platform detection (debounced)
- Download options selection (quality, format, audio-only)
- Loading states and error handling
- "Get Video Info" button

#### VideoPreviewPanel (`components/VideoPreviewPanel.tsx`)
- Single video preview display
- Playlist video list with selection
- Channel structure with expandable playlists
- Checkbox selection for multiple videos
- "Add to Queue" functionality
- Formatted duration and view counts

#### DownloadQueuePanel (`components/DownloadQueuePanel.tsx`)
- Queue list display with thumbnails
- Real-time progress bars
- Download speed and ETA display
- Pause/Resume/Cancel controls
- Drag-and-drop reordering
- Status indicators with color coding

#### SettingsPanel (`components/SettingsPanel.tsx`)
- Modal overlay design
- General settings (save path, quality, format)
- Concurrent downloads configuration
- Auto-retry settings
- Platform-specific settings (YouTube)
- Save/Reset functionality

#### Toast Notification System
- `Toast.tsx`: Individual toast component with auto-dismiss
- `ToastContainer.tsx`: Container for managing multiple toasts
- `useToast.ts`: Hook for toast management
- Support for success, error, warning, and info types
- macOS system notification integration
- Action buttons support

### Type Definitions (`types/index.ts`)
- Complete TypeScript interfaces for all data structures
- Enums for video quality and format
- Download status types
- Content type unions

### API Layer (`api/tauri.ts`)
- Wrapper functions for all Tauri commands
- Event listener setup functions
- Type-safe API calls
- Error handling

## State Management

The application uses React hooks for state management:
- `useState` for local component state
- `useEffect` for side effects and event listeners
- Custom `useToast` hook for notifications

## Styling

- Tailwind CSS for utility-first styling
- Dark theme optimized for macOS
- Responsive design with grid layouts
- Custom animations for toasts
- Hover states and transitions

## Integration

All components are integrated in `App.tsx`:
- Event listeners for download progress and status
- Settings persistence
- Queue management
- Toast notifications for user feedback

## Build

The application builds successfully with TypeScript and Vite:
```bash
npm run build
```

All TypeScript diagnostics have been resolved.
