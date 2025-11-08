import { useState } from 'react';
import type { DownloadItem, DownloadStatus } from '../types';

interface DownloadQueuePanelProps {
  queue: DownloadItem[];
  onPause: (id: string) => void;
  onResume: (id: string) => void;
  onCancel: (id: string) => void;
  onReorder: (fromIndex: number, toIndex: number) => void;
}

export default function DownloadQueuePanel({
  queue,
  onPause,
  onResume,
  onCancel,
  onReorder
}: DownloadQueuePanelProps) {
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null);
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null);
  const [loadingOperations, setLoadingOperations] = useState<Set<string>>(new Set());

  const formatSpeed = (bytesPerSecond: number): string => {
    if (bytesPerSecond >= 1024 * 1024) {
      return `${(bytesPerSecond / (1024 * 1024)).toFixed(1)} MB/s`;
    }
    if (bytesPerSecond >= 1024) {
      return `${(bytesPerSecond / 1024).toFixed(1)} KB/s`;
    }
    return `${bytesPerSecond} B/s`;
  };

  const formatETA = (seconds: number): string => {
    if (seconds < 60) {
      return `${seconds}s`;
    }
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) {
      return `${minutes}m ${seconds % 60}s`;
    }
    const hours = Math.floor(minutes / 60);
    return `${hours}h ${minutes % 60}m`;
  };

  const getStatusBadgeClass = (status: DownloadStatus): string => {
    switch (status) {
      case 'downloading':
        return 'badge-info';
      case 'completed':
        return 'badge-success';
      case 'failed':
        return 'badge-error';
      case 'paused':
        return 'badge-warning';
      case 'cancelled':
        return 'badge bg-gray-600 text-gray-300';
      default:
        return 'badge bg-gray-600 text-gray-300';
    }
  };

  const getStatusIcon = (status: DownloadStatus) => {
    switch (status) {
      case 'downloading':
        return (
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
          </svg>
        );
      case 'completed':
        return (
          <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
          </svg>
        );
      case 'failed':
        return (
          <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
          </svg>
        );
      case 'paused':
        return (
          <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zM7 8a1 1 0 012 0v4a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v4a1 1 0 102 0V8a1 1 0 00-1-1z" clipRule="evenodd" />
          </svg>
        );
      case 'queued':
        return (
          <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-12a1 1 0 10-2 0v4a1 1 0 00.293.707l2.828 2.829a1 1 0 101.415-1.415L11 9.586V6z" clipRule="evenodd" />
          </svg>
        );
      default:
        return null;
    }
  };

  const getStatusText = (status: DownloadStatus): string => {
    switch (status) {
      case 'queued':
        return 'Queued';
      case 'downloading':
        return 'Downloading';
      case 'paused':
        return 'Paused';
      case 'completed':
        return 'Completed';
      case 'failed':
        return 'Failed';
      case 'cancelled':
        return 'Cancelled';
      default:
        return status;
    }
  };

  const getProgressBarClass = (status: DownloadStatus): string => {
    switch (status) {
      case 'downloading':
        return 'bg-primary';
      case 'paused':
        return 'bg-warning';
      case 'completed':
        return 'bg-success';
      case 'failed':
        return 'bg-error';
      default:
        return 'bg-gray-500';
    }
  };

  const handleDragStart = (index: number) => {
    setDraggedIndex(index);
  };

  const handleDragOver = (e: React.DragEvent, index: number) => {
    e.preventDefault();
    setDragOverIndex(index);
  };

  const handleDrop = (e: React.DragEvent, toIndex: number) => {
    e.preventDefault();
    
    if (draggedIndex !== null && draggedIndex !== toIndex) {
      onReorder(draggedIndex, toIndex);
    }
    
    setDraggedIndex(null);
    setDragOverIndex(null);
  };

  const handleDragEnd = () => {
    setDraggedIndex(null);
    setDragOverIndex(null);
  };

  if (queue.length === 0) {
    return (
      <div className="card">
        <h2 className="text-xl font-semibold mb-6 text-left">Download Queue</h2>
        <div className="text-center py-16 text-text-muted">
          <svg className="w-20 h-20 mx-auto mb-4 opacity-40" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
          </svg>
          <p className="text-lg">No downloads in queue</p>
          <p className="text-sm mt-2 text-text-muted">Add videos to start downloading</p>
        </div>
      </div>
    );
  }

  return (
    <div className="card">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-semibold text-left">Download Queue</h2>
        <div className="badge badge-info">
          {queue.length} item{queue.length !== 1 ? 's' : ''}
        </div>
      </div>

      <div className="space-y-3 max-h-[600px] overflow-y-auto pr-2 custom-scrollbar">
        {queue.map((item, index) => (
          <div
            key={item.id}
            draggable={item.status === 'queued' || item.status === 'paused'}
            onDragStart={() => handleDragStart(index)}
            onDragOver={(e) => handleDragOver(e, index)}
            onDrop={(e) => handleDrop(e, index)}
            onDragEnd={handleDragEnd}
            className={`card-compact transition-all duration-200 ${
              draggedIndex === index ? 'opacity-50 scale-95' : ''
            } ${
              dragOverIndex === index && draggedIndex !== index ? 'border-2 border-primary shadow-lg' : 'border-2 border-transparent'
            } ${
              item.status === 'queued' || item.status === 'paused' ? 'cursor-move hover:shadow-md' : ''
            }`}
          >
            <div className="flex items-start gap-4">
              {/* Thumbnail */}
              <div className="relative w-32 h-20 flex-shrink-0 rounded-lg overflow-hidden bg-gray-900">
                <img 
                  src={item.thumbnail} 
                  alt={item.title}
                  className="w-full h-full object-cover"
                  loading="lazy"
                />
              </div>

              {/* Content */}
              <div className="flex-1 min-w-0">
                <div className="flex items-start justify-between gap-2 mb-2">
                  <h3 className="text-sm font-medium text-white line-clamp-2 leading-tight flex-1">
                    {item.title}
                  </h3>
                  
                  {/* Status Badge */}
                  <div className={`${getStatusBadgeClass(item.status)} flex items-center gap-1 flex-shrink-0`}>
                    {getStatusIcon(item.status)}
                    <span>{getStatusText(item.status)}</span>
                  </div>
                </div>

                {/* Error Message */}
                {item.error && (
                  <div className="flex items-center gap-2 text-xs text-error mb-2">
                    <svg className="w-3 h-3 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                      <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                    </svg>
                    <span className="truncate">{item.error}</span>
                  </div>
                )}

                {/* Progress Bar */}
                {(item.status === 'downloading' || item.status === 'paused') && (
                  <div className="mt-2">
                    <div className="flex items-center justify-between text-xs text-text-muted mb-1.5">
                      <span className="font-medium">{item.progress.toFixed(1)}%</span>
                      {item.status === 'downloading' && (
                        <div className="flex items-center gap-3">
                          <div className="flex items-center gap-1">
                            <svg className="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                              <path fillRule="evenodd" d="M12 7a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0V8.414l-4.293 4.293a1 1 0 01-1.414 0L8 10.414l-4.293 4.293a1 1 0 01-1.414-1.414l5-5a1 1 0 011.414 0L11 10.586 14.586 7H12z" clipRule="evenodd" />
                            </svg>
                            <span>{formatSpeed(item.speed)}</span>
                          </div>
                          <span>•</span>
                          <div className="flex items-center gap-1">
                            <svg className="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                              <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-12a1 1 0 10-2 0v4a1 1 0 00.293.707l2.828 2.829a1 1 0 101.415-1.415L11 9.586V6z" clipRule="evenodd" />
                            </svg>
                            <span>{formatETA(item.eta)}</span>
                          </div>
                        </div>
                      )}
                    </div>
                    <div className="progress-container">
                      <div 
                        className={`progress-fill ${getProgressBarClass(item.status)}`}
                        style={{ width: `${item.progress}%` }}
                      />
                    </div>
                  </div>
                )}

                {/* Completed Progress */}
                {item.status === 'completed' && (
                  <div className="mt-2">
                    <div className="progress-container">
                      <div className="progress-fill bg-success" style={{ width: '100%' }} />
                    </div>
                  </div>
                )}

                {/* Save Path */}
                <div className="flex items-center gap-1 text-xs text-text-muted mt-2">
                  <svg className="w-3 h-3 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                    <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
                  </svg>
                  <span className="truncate">{item.savePath}</span>
                </div>
              </div>

              {/* Action Buttons */}
              <div className="flex items-center gap-2 flex-shrink-0">
                {item.status === 'downloading' && (
                  <button
                    onClick={async () => {
                      const opKey = `pause-${item.id}`;
                      setLoadingOperations(prev => new Set(prev).add(opKey));
                      try {
                        await onPause(item.id);
                      } finally {
                        setLoadingOperations(prev => {
                          const next = new Set(prev);
                          next.delete(opKey);
                          return next;
                        });
                      }
                    }}
                    disabled={loadingOperations.has(`pause-${item.id}`)}
                    className="p-2 bg-warning hover:bg-warning/80 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                    title="Pause download"
                    aria-label="Pause download"
                  >
                    {loadingOperations.has(`pause-${item.id}`) ? (
                      <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                    ) : (
                      <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                        <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zM7 8a1 1 0 012 0v4a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v4a1 1 0 102 0V8a1 1 0 00-1-1z" clipRule="evenodd" />
                      </svg>
                    )}
                  </button>
                )}

                {item.status === 'paused' && (
                  <button
                    onClick={async () => {
                      const opKey = `resume-${item.id}`;
                      setLoadingOperations(prev => new Set(prev).add(opKey));
                      try {
                        await onResume(item.id);
                      } finally {
                        setLoadingOperations(prev => {
                          const next = new Set(prev);
                          next.delete(opKey);
                          return next;
                        });
                      }
                    }}
                    disabled={loadingOperations.has(`resume-${item.id}`)}
                    className="p-2 bg-success hover:bg-success-hover text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                    title="Resume download"
                    aria-label="Resume download"
                  >
                    {loadingOperations.has(`resume-${item.id}`) ? (
                      <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                    ) : (
                      <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                        <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z" clipRule="evenodd" />
                      </svg>
                    )}
                  </button>
                )}

                {(item.status === 'queued' || item.status === 'downloading' || item.status === 'paused' || item.status === 'failed') && (
                  <button
                    onClick={async () => {
                      const opKey = `cancel-${item.id}`;
                      setLoadingOperations(prev => new Set(prev).add(opKey));
                      try {
                        await onCancel(item.id);
                      } finally {
                        setLoadingOperations(prev => {
                          const next = new Set(prev);
                          next.delete(opKey);
                          return next;
                        });
                      }
                    }}
                    disabled={loadingOperations.has(`cancel-${item.id}`)}
                    className="p-2 bg-error hover:bg-error-hover text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                    title="Cancel download"
                    aria-label="Cancel download"
                  >
                    {loadingOperations.has(`cancel-${item.id}`) ? (
                      <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                    ) : (
                      <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                        <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
                      </svg>
                    )}
                  </button>
                )}

                {/* Drag Handle */}
                {(item.status === 'queued' || item.status === 'paused') && (
                  <div 
                    className="p-2 text-text-muted hover:text-white cursor-move transition-colors" 
                    title="Drag to reorder"
                    aria-label="Drag to reorder"
                  >
                    <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                      <path d="M10 6a2 2 0 110-4 2 2 0 010 4zM10 12a2 2 0 110-4 2 2 0 010 4zM10 18a2 2 0 110-4 2 2 0 010 4z" />
                    </svg>
                  </div>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
