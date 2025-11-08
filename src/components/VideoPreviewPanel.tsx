import { useState } from 'react';
import type { ContentData, VideoInfo, PlaylistInfo } from '../types';
import { SkeletonVideoList } from './SkeletonLoader';

interface VideoPreviewPanelProps {
  content: ContentData | null;
  onAddToQueue: (selectedVideos: VideoInfo[]) => void;
  isLoading?: boolean;
}

export default function VideoPreviewPanel({ content, onAddToQueue, isLoading = false }: VideoPreviewPanelProps) {
  const [selectedVideos, setSelectedVideos] = useState<Set<string>>(new Set());
  const [expandedPlaylists, setExpandedPlaylists] = useState<Set<string>>(new Set());
  const [isAddingToQueue, setIsAddingToQueue] = useState(false);

  if (isLoading) {
    return (
      <div className="card">
        <h2 className="text-xl font-semibold mb-6 text-left">Preview</h2>
        <SkeletonVideoList count={3} />
      </div>
    );
  }

  if (!content) {
    return null;
  }

  const formatDuration = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    
    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }
    return `${minutes}:${secs.toString().padStart(2, '0')}`;
  };

  const formatNumber = (num: number): string => {
    if (num >= 1000000) {
      return `${(num / 1000000).toFixed(1)}M`;
    }
    if (num >= 1000) {
      return `${(num / 1000).toFixed(1)}K`;
    }
    return num.toString();
  };

  const toggleVideo = (videoId: string) => {
    const newSelected = new Set(selectedVideos);
    if (newSelected.has(videoId)) {
      newSelected.delete(videoId);
    } else {
      newSelected.add(videoId);
    }
    setSelectedVideos(newSelected);
  };

  const togglePlaylist = (playlistId: string) => {
    const newExpanded = new Set(expandedPlaylists);
    if (newExpanded.has(playlistId)) {
      newExpanded.delete(playlistId);
    } else {
      newExpanded.add(playlistId);
    }
    setExpandedPlaylists(newExpanded);
  };

  const selectAllInPlaylist = (videos: VideoInfo[]) => {
    const newSelected = new Set(selectedVideos);
    videos.forEach(video => newSelected.add(video.id));
    setSelectedVideos(newSelected);
  };

  const deselectAllInPlaylist = (videos: VideoInfo[]) => {
    const newSelected = new Set(selectedVideos);
    videos.forEach(video => newSelected.delete(video.id));
    setSelectedVideos(newSelected);
  };

  const handleAddToQueue = async () => {
    const videos: VideoInfo[] = [];
    
    if (content.type === 'video') {
      videos.push(content.data);
    } else if (content.type === 'playlist') {
      content.data.videos.forEach(video => {
        if (selectedVideos.has(video.id)) {
          videos.push(video);
        }
      });
    } else if (content.type === 'channel') {
      content.data.playlists.forEach(playlist => {
        playlist.videos.forEach(video => {
          if (selectedVideos.has(video.id)) {
            videos.push(video);
          }
        });
      });
    }
    
    if (videos.length > 0) {
      setIsAddingToQueue(true);
      try {
        await onAddToQueue(videos);
        setSelectedVideos(new Set());
      } finally {
        setIsAddingToQueue(false);
      }
    }
  };

  const renderVideoItem = (video: VideoInfo, showCheckbox: boolean = true) => (
    <div 
      key={video.id} 
      className={`group flex items-start gap-4 p-4 rounded-lg transition-all duration-200 ${
        showCheckbox 
          ? selectedVideos.has(video.id)
            ? 'bg-primary/10 border-2 border-primary'
            : 'bg-gray-700 border-2 border-transparent hover:border-gray-600 hover:shadow-md cursor-pointer'
          : 'bg-gray-700'
      }`}
      onClick={showCheckbox ? () => toggleVideo(video.id) : undefined}
    >
      {showCheckbox && (
        <input
          type="checkbox"
          checked={selectedVideos.has(video.id)}
          onChange={() => toggleVideo(video.id)}
          className="mt-1 w-4 h-4 text-primary bg-gray-600 border-gray-500 rounded focus:ring-2 focus:ring-primary transition-colors flex-shrink-0"
          onClick={(e) => e.stopPropagation()}
          aria-label={`Select ${video.title}`}
        />
      )}
      
      <div className="relative w-40 h-24 flex-shrink-0 rounded-lg overflow-hidden bg-gray-900">
        <img 
          src={video.thumbnail} 
          alt={video.title}
          className="w-full h-full object-cover transition-transform duration-200 group-hover:scale-105"
          loading="lazy"
        />
        <div className="absolute bottom-1 right-1 px-1.5 py-0.5 bg-black/80 text-white text-xs rounded">
          {formatDuration(video.duration)}
        </div>
      </div>
      
      <div className="flex-1 min-w-0 flex flex-col justify-between">
        <div>
          <h4 className="text-sm font-medium text-white line-clamp-2 leading-tight group-hover:text-primary-light transition-colors">
            {video.title}
          </h4>
          <p className="text-xs text-text-muted mt-1 truncate">{video.uploader}</p>
        </div>
        <div className="flex items-center gap-2 mt-1 text-xs text-text-muted">
          <svg className="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
            <path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
            <path fillRule="evenodd" d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z" clipRule="evenodd" />
          </svg>
          <span>{formatNumber(video.view_count)} views</span>
        </div>
      </div>
    </div>
  );

  const renderPlaylistSection = (playlist: PlaylistInfo) => {
    const isExpanded = expandedPlaylists.has(playlist.id);
    const allSelected = playlist.videos.every(v => selectedVideos.has(v.id));
    const someSelected = playlist.videos.some(v => selectedVideos.has(v.id));

    return (
      <div key={playlist.id} className="card-compact border border-gray-600 overflow-hidden">
        <div className="bg-bg-secondary p-4">
          <div className="flex items-center justify-between gap-4">
            <div className="flex items-center gap-3 flex-1 min-w-0">
              <button
                onClick={() => togglePlaylist(playlist.id)}
                className="text-text-muted hover:text-white transition-colors flex-shrink-0"
                aria-label={isExpanded ? 'Collapse playlist' : 'Expand playlist'}
              >
                <svg 
                  className={`w-5 h-5 transform transition-transform duration-200 ${isExpanded ? 'rotate-90' : ''}`}
                  fill="currentColor" 
                  viewBox="0 0 20 20"
                >
                  <path fillRule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clipRule="evenodd" />
                </svg>
              </button>
              
              <div className="flex-1 min-w-0">
                <h3 className="text-base font-semibold text-white truncate">{playlist.title}</h3>
                <p className="text-sm text-text-muted">{playlist.video_count} videos</p>
              </div>
            </div>
            
            <button
              onClick={() => allSelected ? deselectAllInPlaylist(playlist.videos) : selectAllInPlaylist(playlist.videos)}
              className="btn-secondary text-sm px-3 py-1.5 flex-shrink-0"
              aria-label={allSelected ? 'Deselect all videos in playlist' : 'Select all videos in playlist'}
            >
              {allSelected ? 'Deselect All' : someSelected ? 'Select All' : 'Select All'}
            </button>
          </div>
        </div>
        
        {isExpanded && (
          <div className="p-4 space-y-3 max-h-96 overflow-y-auto custom-scrollbar">
            {playlist.videos.map(video => renderVideoItem(video))}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="card">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-semibold text-left">Preview</h2>
        
        {content.type !== 'video' && (
          <div className="badge badge-info">
            {selectedVideos.size} selected
          </div>
        )}
      </div>

      <div className="space-y-4">
        {content.type === 'video' && (
          <div>
            {renderVideoItem(content.data, false)}
          </div>
        )}

        {content.type === 'playlist' && (
          <div>
            <div className="mb-4 p-4 bg-bg-secondary rounded-lg">
              <h3 className="text-lg font-semibold text-white mb-1">{content.data.title}</h3>
              <p className="text-sm text-text-secondary">{content.data.uploader}</p>
              <div className="flex items-center gap-2 mt-2">
                <svg className="w-4 h-4 text-text-muted" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M2 6a2 2 0 012-2h6a2 2 0 012 2v8a2 2 0 01-2 2H4a2 2 0 01-2-2V6zM14.553 7.106A1 1 0 0014 8v4a1 1 0 00.553.894l2 1A1 1 0 0018 13V7a1 1 0 00-1.447-.894l-2 1z" />
                </svg>
                <span className="text-sm text-text-muted">{content.data.video_count} videos</span>
              </div>
            </div>
            
            <div className="flex justify-end mb-3">
              <button
                onClick={() => {
                  const allSelected = content.data.videos.every(v => selectedVideos.has(v.id));
                  if (allSelected) {
                    setSelectedVideos(new Set());
                  } else {
                    setSelectedVideos(new Set(content.data.videos.map(v => v.id)));
                  }
                }}
                className="btn-secondary text-sm px-3 py-1.5"
                aria-label={content.data.videos.every(v => selectedVideos.has(v.id)) ? 'Deselect all videos' : 'Select all videos'}
              >
                {content.data.videos.every(v => selectedVideos.has(v.id)) ? 'Deselect All' : 'Select All'}
              </button>
            </div>
            
            <div className="space-y-3 max-h-[32rem] overflow-y-auto pr-2 custom-scrollbar">
              {content.data.videos.map(video => renderVideoItem(video))}
            </div>
          </div>
        )}

        {content.type === 'channel' && (
          <div>
            <div className="mb-4 p-4 bg-bg-secondary rounded-lg">
              <h3 className="text-lg font-semibold text-white mb-1">{content.data.name}</h3>
              <div className="flex items-center gap-2 mt-2">
                <svg className="w-4 h-4 text-text-muted" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M2 6a2 2 0 012-2h6a2 2 0 012 2v8a2 2 0 01-2 2H4a2 2 0 01-2-2V6zM14.553 7.106A1 1 0 0014 8v4a1 1 0 00.553.894l2 1A1 1 0 0018 13V7a1 1 0 00-1.447-.894l-2 1z" />
                </svg>
                <span className="text-sm text-text-muted">
                  {content.data.playlists.length} playlists
                </span>
              </div>
            </div>
            
            <div className="space-y-4 max-h-[32rem] overflow-y-auto pr-2 custom-scrollbar">
              {content.data.playlists.map(playlist => renderPlaylistSection(playlist))}
            </div>
          </div>
        )}
      </div>

      {/* Add to Queue Button */}
      <button
        onClick={handleAddToQueue}
        disabled={isAddingToQueue || (content.type !== 'video' && selectedVideos.size === 0)}
        className="btn-success w-full mt-6 disabled:bg-gray-600 disabled:cursor-not-allowed disabled:opacity-50"
        aria-label="Add selected videos to download queue"
      >
        {isAddingToQueue ? (
          <>
            <div className="spinner"></div>
            <span>Adding to Queue...</span>
          </>
        ) : (
          <>
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
            </svg>
            <span>
              Add to Queue {content.type !== 'video' && selectedVideos.size > 0 && `(${selectedVideos.size})`}
            </span>
          </>
        )}
      </button>
    </div>
  );
}
