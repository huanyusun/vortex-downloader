// Modal component to display detailed error information

import { ErrorCategory } from '../utils/batchOperations';

interface ErrorDetailsModalProps {
  isOpen: boolean;
  onClose: () => void;
  errorCategories: ErrorCategory[];
  onRetry?: () => void;
}

export default function ErrorDetailsModal({
  isOpen,
  onClose,
  errorCategories,
  onRetry,
}: ErrorDetailsModalProps) {
  if (!isOpen) return null;

  const totalErrors = errorCategories.reduce((sum, cat) => sum + cat.count, 0);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm animate-fade-in">
      <div className="bg-gray-800 rounded-xl shadow-2xl max-w-2xl w-full max-h-[80vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-700">
          <div>
            <h2 className="text-xl font-semibold text-white">Error Details</h2>
            <p className="text-sm text-text-muted mt-1">
              {totalErrors} item{totalErrors !== 1 ? 's' : ''} failed to add to queue
            </p>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-gray-700 rounded-lg transition-colors"
            aria-label="Close"
          >
            <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
              <path
                fillRule="evenodd"
                d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                clipRule="evenodd"
              />
            </svg>
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6 space-y-4 custom-scrollbar">
          {errorCategories.map((category, index) => (
            <div key={index} className="bg-gray-700 rounded-lg p-4">
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-2">
                  <svg className="w-5 h-5 text-error" fill="currentColor" viewBox="0 0 20 20">
                    <path
                      fillRule="evenodd"
                      d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                      clipRule="evenodd"
                    />
                  </svg>
                  <h3 className="font-semibold text-white">{category.type}</h3>
                </div>
                <span className="badge badge-error">{category.count}</span>
              </div>

              <p className="text-sm text-text-secondary mb-3">{category.message}</p>

              <div className="space-y-1">
                <p className="text-xs text-text-muted font-medium mb-2">Affected items:</p>
                <ul className="space-y-1">
                  {category.items.map((item, itemIndex) => (
                    <li
                      key={itemIndex}
                      className="text-sm text-text-muted flex items-start gap-2 pl-2"
                    >
                      <span className="text-error mt-1">•</span>
                      <span className="flex-1 break-words">{item}</span>
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          ))}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 p-6 border-t border-gray-700">
          <button onClick={onClose} className="btn-secondary">
            Close
          </button>
          {onRetry && (
            <button onClick={onRetry} className="btn-primary">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                />
              </svg>
              <span>Retry Failed Items</span>
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
