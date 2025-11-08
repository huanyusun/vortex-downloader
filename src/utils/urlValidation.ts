// URL validation utilities for supported platforms

export interface URLValidationResult {
  isValid: boolean;
  platform?: string;
  errorMessage?: string;
}

export interface PlatformPattern {
  name: string;
  patterns: RegExp[];
  examples: string[];
}

// Supported platform patterns
export const PLATFORM_PATTERNS: PlatformPattern[] = [
  {
    name: 'YouTube',
    patterns: [
      /^https?:\/\/(www\.)?youtube\.com\/watch\?v=[\w-]+/,
      /^https?:\/\/(www\.)?youtube\.com\/playlist\?list=[\w-]+/,
      /^https?:\/\/(www\.)?youtube\.com\/@[\w-]+/,
      /^https?:\/\/(www\.)?youtube\.com\/channel\/[\w-]+/,
      /^https?:\/\/(www\.)?youtube\.com\/c\/[\w-]+/,
      /^https?:\/\/youtu\.be\/[\w-]+/,
    ],
    examples: [
      'https://www.youtube.com/watch?v=dQw4w9WgXcQ',
      'https://youtu.be/dQw4w9WgXcQ',
      'https://www.youtube.com/playlist?list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf',
      'https://www.youtube.com/@channelname',
      'https://www.youtube.com/channel/UCxxxxxx',
    ],
  },
];

/**
 * Validate URL format on the client side
 */
export function validateURL(url: string): URLValidationResult {
  // Check if URL is empty
  if (!url || url.trim().length === 0) {
    return {
      isValid: false,
      errorMessage: 'Please enter a URL',
    };
  }

  const trimmedUrl = url.trim();

  // Check if it's a valid URL format
  try {
    new URL(trimmedUrl);
  } catch {
    return {
      isValid: false,
      errorMessage: 'Invalid URL format. Please enter a valid URL starting with http:// or https://',
    };
  }

  // Check against supported platform patterns
  for (const platform of PLATFORM_PATTERNS) {
    for (const pattern of platform.patterns) {
      if (pattern.test(trimmedUrl)) {
        return {
          isValid: true,
          platform: platform.name,
        };
      }
    }
  }

  // URL is valid but not from a supported platform
  return {
    isValid: false,
    errorMessage: 'Unsupported platform. Currently only YouTube URLs are supported.',
  };
}

/**
 * Get supported URL format examples for display
 */
export function getSupportedURLExamples(): string[] {
  return PLATFORM_PATTERNS.flatMap((platform) => platform.examples);
}

/**
 * Get formatted help text for supported URLs
 */
export function getSupportedURLHelpText(): string {
  const platforms = PLATFORM_PATTERNS.map((p) => p.name).join(', ');
  return `Supported platforms: ${platforms}`;
}

/**
 * Check if URL is likely a video, playlist, or channel
 */
export function detectContentType(url: string): 'video' | 'playlist' | 'channel' | 'unknown' {
  if (url.includes('/playlist')) {
    return 'playlist';
  }
  if (url.includes('/@') || url.includes('/channel/') || url.includes('/c/')) {
    return 'channel';
  }
  if (url.includes('/watch') || url.includes('youtu.be/')) {
    return 'video';
  }
  return 'unknown';
}
