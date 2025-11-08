// UI State Store
import { create } from 'zustand';
import type { ContentData } from '../types';

type ViewMode = 'main' | 'settings';

interface UIStore {
  // Current view
  currentView: ViewMode;
  setCurrentView: (view: ViewMode) => void;
  
  // Content preview
  content: ContentData | null;
  setContent: (content: ContentData | null) => void;
  
  // Loading states
  isLoadingContent: boolean;
  setLoadingContent: (loading: boolean) => void;
  
  // Operation loading states
  loadingOperations: Map<string, boolean>;
  setOperationLoading: (operation: string, loading: boolean) => void;
  isOperationLoading: (operation: string) => boolean;
  
  // Settings panel
  showSettings: boolean;
  openSettings: () => void;
  closeSettings: () => void;
  
  // Selected items for batch operations
  selectedVideoIds: Set<string>;
  toggleVideoSelection: (id: string) => void;
  selectAllVideos: (ids: string[]) => void;
  clearSelection: () => void;
  
  // Error state
  error: string | null;
  setError: (error: string | null) => void;
  clearError: () => void;
}

export const useUIStore = create<UIStore>((set) => ({
  // Current view
  currentView: 'main',
  setCurrentView: (view) => set({ currentView: view }),
  
  // Content preview
  content: null,
  setContent: (content) => set({ content }),
  
  // Loading states
  isLoadingContent: false,
  setLoadingContent: (loading) => set({ isLoadingContent: loading }),
  
  // Operation loading states
  loadingOperations: new Map(),
  setOperationLoading: (operation, loading) =>
    set((state) => {
      const newMap = new Map(state.loadingOperations);
      if (loading) {
        newMap.set(operation, true);
      } else {
        newMap.delete(operation);
      }
      return { loadingOperations: newMap };
    }),
  isOperationLoading: (operation: string): boolean => {
    return useUIStore.getState().loadingOperations.has(operation);
  },
  
  // Settings panel
  showSettings: false,
  openSettings: () => set({ showSettings: true }),
  closeSettings: () => set({ showSettings: false }),
  
  // Selected items
  selectedVideoIds: new Set(),
  toggleVideoSelection: (id) =>
    set((state) => {
      const newSet = new Set(state.selectedVideoIds);
      if (newSet.has(id)) {
        newSet.delete(id);
      } else {
        newSet.add(id);
      }
      return { selectedVideoIds: newSet };
    }),
  selectAllVideos: (ids) =>
    set({ selectedVideoIds: new Set(ids) }),
  clearSelection: () =>
    set({ selectedVideoIds: new Set() }),
  
  // Error state
  error: null,
  setError: (error) => set({ error }),
  clearError: () => set({ error: null })
}));
