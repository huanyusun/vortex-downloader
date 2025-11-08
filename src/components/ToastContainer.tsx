import Toast, { ToastProps } from './Toast';

interface ToastContainerProps {
  toasts: Omit<ToastProps, 'onClose'>[];
  onClose: (id: string) => void;
}

export default function ToastContainer({ toasts, onClose }: ToastContainerProps) {
  // Limit to 3 visible toasts at a time
  const visibleToasts = toasts.slice(0, 3);

  return (
    <div className="fixed top-4 right-4 z-50 space-y-3 max-w-sm w-full pointer-events-none">
      <div className="space-y-3 pointer-events-auto">
        {visibleToasts.map((toast) => (
          <Toast key={toast.id} {...toast} onClose={onClose} />
        ))}
      </div>
      
      {/* Show count of hidden toasts */}
      {toasts.length > 3 && (
        <div className="text-center">
          <div className="inline-block px-3 py-1 bg-gray-800 border border-gray-700 rounded-full text-xs text-text-muted shadow-lg">
            +{toasts.length - 3} more notification{toasts.length - 3 !== 1 ? 's' : ''}
          </div>
        </div>
      )}
    </div>
  );
}
