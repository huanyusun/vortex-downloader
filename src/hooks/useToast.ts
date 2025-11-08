import { useState, useCallback } from 'react';
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/api/notification';
import type { ToastProps, ToastType } from '../components/Toast';

export interface ToastOptions {
  title: string;
  message: string;
  type?: ToastType;
  duration?: number;
  action?: {
    label: string;
    callback: () => void;
  };
  systemNotification?: boolean;
}

export function useToast() {
  const [toasts, setToasts] = useState<Omit<ToastProps, 'onClose'>[]>([]);

  const showToast = useCallback(async (options: ToastOptions) => {
    const id = `toast-${Date.now()}-${Math.random()}`;
    const toast: Omit<ToastProps, 'onClose'> = {
      id,
      type: options.type || 'info',
      title: options.title,
      message: options.message,
      duration: options.duration,
      action: options.action
    };

    setToasts((prev) => [...prev, toast]);

    // Send system notification if requested (macOS)
    if (options.systemNotification) {
      try {
        let permissionGranted = await isPermissionGranted();
        
        if (!permissionGranted) {
          const permission = await requestPermission();
          permissionGranted = permission === 'granted';
        }
        
        if (permissionGranted) {
          await sendNotification({
            title: options.title,
            body: options.message
          });
        }
      } catch (error) {
        console.error('Failed to send system notification:', error);
      }
    }

    return id;
  }, []);

  const closeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((toast) => toast.id !== id));
  }, []);

  const showSuccess = useCallback((title: string, message: string, systemNotification = false) => {
    return showToast({ title, message, type: 'success', systemNotification });
  }, [showToast]);

  const showError = useCallback((title: string, message: string, systemNotification = false) => {
    return showToast({ title, message, type: 'error', systemNotification });
  }, [showToast]);

  const showWarning = useCallback((title: string, message: string, systemNotification = false) => {
    return showToast({ title, message, type: 'warning', systemNotification });
  }, [showToast]);

  const showInfo = useCallback((title: string, message: string, systemNotification = false) => {
    return showToast({ title, message, type: 'info', systemNotification });
  }, [showToast]);

  return {
    toasts,
    showToast,
    closeToast,
    showSuccess,
    showError,
    showWarning,
    showInfo
  };
}
