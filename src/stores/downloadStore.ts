// Download Queue Store
import { create } from 'zustand';
import type { DownloadItem, DownloadProgress } from '../types';

interface DownloadStore {
  queue: DownloadItem[];
  setQueue: (queue: DownloadItem[]) => void;
  addToQueue: (items: DownloadItem[]) => void;
  updateDownloadProgress: (id: string, progress: DownloadProgress) => void;
  updateDownloadStatus: (id: string, status: DownloadItem['status']) => void;
  updateDownloadError: (id: string, error: string) => void;
  removeFromQueue: (id: string) => void;
  clearCompleted: () => void;
  reorderQueue: (fromIndex: number, toIndex: number) => void;
}

export const useDownloadStore = create<DownloadStore>((set) => ({
  queue: [],
  
  setQueue: (queue) => set({ queue }),
  
  addToQueue: (items) =>
    set((state) => ({
      queue: [...state.queue, ...items]
    })),
  
  updateDownloadProgress: (id, progress) =>
    set((state) => ({
      queue: state.queue.map((item) =>
        item.id === id
          ? {
              ...item,
              progress: progress.percentage,
              speed: progress.speed,
              eta: progress.eta
            }
          : item
      )
    })),
  
  updateDownloadStatus: (id, status) =>
    set((state) => ({
      queue: state.queue.map((item) =>
        item.id === id ? { ...item, status } : item
      )
    })),
  
  updateDownloadError: (id, error) =>
    set((state) => ({
      queue: state.queue.map((item) =>
        item.id === id
          ? { ...item, status: 'failed' as const, error }
          : item
      )
    })),
  
  removeFromQueue: (id) =>
    set((state) => ({
      queue: state.queue.filter((item) => item.id !== id)
    })),
  
  clearCompleted: () =>
    set((state) => ({
      queue: state.queue.filter((item) => item.status !== 'completed')
    })),
  
  reorderQueue: (fromIndex, toIndex) =>
    set((state) => {
      const newQueue = [...state.queue];
      const [removed] = newQueue.splice(fromIndex, 1);
      newQueue.splice(toIndex, 0, removed);
      return { queue: newQueue };
    })
}));
