import { useState, useEffect } from 'react';
import { detectPlatform } from '../api/tauri';
import { VideoQuality, VideoFormat, DownloadOptions } from '../types';
import { validateURL, getSupportedURLExamples, detectContentType } from '../utils/urlValidation';

interface URLInputPanelProps {
  onFetchInfo: (url: string, options: DownloadOptions) => void;
  isLoading?: boolean;
}

export default function URLInputPanel({ onFetchInfo, isLoading = false }: URLInputPanelProps) {
  const [url, setUrl] = useState('');
  const [platform, setPlatform] = useState<string | null>(null);
  const [quality, setQuality] = useState<VideoQuality>(VideoQuality.BEST);
  const [format, setFormat] = useState<VideoFormat>(VideoFormat.MP4);
  const [audioOnly, setAudioOnly] = useState(false);
  const [isValidating, setIsValidating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [validationError, setValidationError] = useState<string | null>(null);
  const [showHelp, setShowHelp] = useState(false);
  const [contentType, setContentType] = useState<string | null>(null);

  // Auto-detect platform when URL changes with client-side validation
  useEffect(() => {
    const detectPlatformDebounced = async () => {
      if (!url.trim()) {
        setPlatform(null);
        setError(null);
        setValidationError(null);
        setContentType(null);
        return;
      }

      // First, perform client-side validation
      const validation = validateURL(url);
      
      if (!validation.isValid) {
        setPlatform(null);
        setValidationError(validation.errorMessage || 'Invalid URL');
        setError(null);
        setContentType(null);
        return;
      }

      // Clear validation error if URL is valid
      setValidationError(null);
      setIsValidating(true);
      setError(null);

      try {
        // Detect content type
        const type = detectContentType(url);
        setContentType(type !== 'unknown' ? type : null);

        // Verify with backend
        const detectedPlatform = await detectPlatform(url);
        setPlatform(detectedPlatform);
      } catch (err: any) {
        setPlatform(null);
        setContentType(null);
        const errorMsg = err.message || 'Unable to verify URL. Please check the URL and try again.';
        setError(errorMsg);
      } finally {
        setIsValidating(false);
      }
    };

    const timer = setTimeout(detectPlatformDebounced, 500);
    return () => clearTimeout(timer);
  }, [url]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    // Validate URL before submission
    const validation = validateURL(url);
    
    if (!validation.isValid) {
      setValidationError(validation.errorMessage || 'Invalid URL');
      return;
    }

    if (!platform) {
      setError('Unable to verify platform. Please wait for validation to complete.');
      return;
    }

    const options: DownloadOptions = {
      quality,
      format,
      audioOnly
    };

    onFetchInfo(url, options);
  };

  return (
    <div className="card">
      <h2 className="text-xl font-semibold mb-6 text-left">Download Video</h2>
      
      <form onSubmit={handleSubmit} className="space-y-6">
        {/* URL Input */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <label htmlFor="url" className="form-label">
              Video URL
            </label>
            <button
              type="button"
              onClick={() => setShowHelp(!showHelp)}
              className="text-xs text-primary hover:text-primary-light transition-colors flex items-center gap-1"
              aria-label="Show supported URL formats"
            >
              <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-8-3a1 1 0 00-.867.5 1 1 0 11-1.731-1A3 3 0 0113 8a3.001 3.001 0 01-2 2.83V11a1 1 0 11-2 0v-1a1 1 0 011-1 1 1 0 100-2zm0 8a1 1 0 100-2 1 1 0 000 2z" clipRule="evenodd" />
              </svg>
              <span>Supported formats</span>
            </button>
          </div>
          
          <div className="relative">
            <input
              id="url"
              type="text"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder="https://www.youtube.com/watch?v=..."
              className={`input-field pr-12 ${validationError ? 'border-error focus:ring-error' : ''}`}
              disabled={isLoading}
              aria-label="Video URL"
              aria-invalid={!!validationError}
              aria-describedby={validationError ? 'url-error' : undefined}
            />
            {isValidating && (
              <div className="absolute right-3 top-1/2 -translate-y-1/2">
                <div className="spinner-sm border-primary"></div>
              </div>
            )}
          </div>
          
          {/* Help Text - Supported Formats */}
          {showHelp && (
            <div className="p-3 bg-gray-700 rounded-lg text-sm space-y-2 animate-slide-in-right">
              <p className="text-text-secondary font-medium">Supported URL formats:</p>
              <ul className="space-y-1 text-text-muted">
                {getSupportedURLExamples().map((example, index) => (
                  <li key={index} className="flex items-start gap-2">
                    <svg className="w-4 h-4 mt-0.5 flex-shrink-0 text-primary" fill="currentColor" viewBox="0 0 20 20">
                      <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
                    </svg>
                    <code className="text-xs break-all">{example}</code>
                  </li>
                ))}
              </ul>
            </div>
          )}
          
          {/* Validation Error Display */}
          {validationError && (
            <div id="url-error" className="flex items-start gap-2 text-sm text-error animate-slide-in-right" role="alert">
              <svg className="w-4 h-4 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
              </svg>
              <span>{validationError}</span>
            </div>
          )}
          
          {/* Platform Detection Display */}
          {platform && !error && !validationError && (
            <div className="flex items-center justify-between gap-2 text-sm animate-slide-in-right">
              <div className="flex items-center gap-2 text-success">
                <svg className="w-4 h-4 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                </svg>
                <span>Platform: <strong>{platform}</strong></span>
              </div>
              {contentType && (
                <div className="flex items-center gap-1 text-text-muted">
                  <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                    <path d="M2 6a2 2 0 012-2h6a2 2 0 012 2v8a2 2 0 01-2 2H4a2 2 0 01-2-2V6zM14.553 7.106A1 1 0 0014 8v4a1 1 0 00.553.894l2 1A1 1 0 0018 13V7a1 1 0 00-1.447-.894l-2 1z" />
                  </svg>
                  <span className="capitalize">{contentType}</span>
                </div>
              )}
            </div>
          )}
          
          {/* Backend Error Display */}
          {error && !validationError && (
            <div className="flex items-start gap-2 text-sm text-error animate-slide-in-right" role="alert">
              <svg className="w-4 h-4 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
              </svg>
              <span>{error}</span>
            </div>
          )}
        </div>

        {/* Download Options */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {/* Quality Selection */}
          <div className="space-y-2">
            <label htmlFor="quality" className="form-label">
              Quality
            </label>
            <select
              id="quality"
              value={quality}
              onChange={(e) => setQuality(e.target.value as VideoQuality)}
              className="select-field"
              disabled={isLoading || audioOnly}
              aria-label="Video quality"
            >
              <option value={VideoQuality.BEST}>Best Quality</option>
              <option value={VideoQuality.HIGH_1080P}>1080p</option>
              <option value={VideoQuality.MEDIUM_720P}>720p</option>
              <option value={VideoQuality.LOW_480P}>480p</option>
            </select>
          </div>

          {/* Format Selection */}
          <div className="space-y-2">
            <label htmlFor="format" className="form-label">
              Format
            </label>
            <select
              id="format"
              value={format}
              onChange={(e) => setFormat(e.target.value as VideoFormat)}
              className="select-field"
              disabled={isLoading || audioOnly}
              aria-label="Video format"
            >
              <option value={VideoFormat.MP4}>MP4</option>
              <option value={VideoFormat.WEBM}>WebM</option>
              <option value={VideoFormat.MKV}>MKV</option>
            </select>
          </div>

          {/* Audio Only Checkbox */}
          <div className="flex items-end pb-2">
            <label className="flex items-center gap-2 cursor-pointer group">
              <input
                type="checkbox"
                checked={audioOnly}
                onChange={(e) => setAudioOnly(e.target.checked)}
                className="w-4 h-4 text-primary bg-gray-700 border-gray-600 rounded focus:ring-2 focus:ring-primary transition-colors"
                disabled={isLoading}
                aria-label="Audio only"
              />
              <span className="text-sm text-gray-300 group-hover:text-white transition-colors">
                Audio Only
              </span>
            </label>
          </div>
        </div>

        {/* Submit Button */}
        <button
          type="submit"
          disabled={isLoading || !platform || isValidating || !!validationError}
          className="btn-primary w-full disabled:bg-gray-600 disabled:cursor-not-allowed disabled:opacity-50"
          aria-label="Get video information"
          title={
            validationError 
              ? 'Please enter a valid URL' 
              : isValidating 
              ? 'Validating URL...' 
              : !platform 
              ? 'Enter a URL to continue' 
              : 'Fetch video information'
          }
        >
          {isLoading ? (
            <>
              <div className="spinner"></div>
              <span>Fetching Info...</span>
            </>
          ) : isValidating ? (
            <>
              <div className="spinner"></div>
              <span>Validating...</span>
            </>
          ) : (
            <>
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
              <span>Get Video Info</span>
            </>
          )}
        </button>
      </form>
    </div>
  );
}
