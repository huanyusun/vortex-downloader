// Skeleton loader components for loading states

interface SkeletonProps {
  className?: string;
}

export function SkeletonBox({ className = '' }: SkeletonProps) {
  return (
    <div
      className={`bg-gray-700 animate-pulse rounded ${className}`}
      role="status"
      aria-label="Loading"
    />
  );
}

export function SkeletonText({ className = '' }: SkeletonProps) {
  return <SkeletonBox className={`h-4 ${className}`} />;
}

export function SkeletonVideoCard() {
  return (
    <div className="flex items-start gap-4 p-4 bg-gray-700 rounded-lg">
      {/* Thumbnail skeleton */}
      <SkeletonBox className="w-40 h-24 flex-shrink-0" />
      
      {/* Content skeleton */}
      <div className="flex-1 space-y-2">
        <SkeletonText className="w-3/4" />
        <SkeletonText className="w-1/2" />
        <SkeletonText className="w-1/4" />
      </div>
    </div>
  );
}

export function SkeletonVideoList({ count = 3 }: { count?: number }) {
  return (
    <div className="space-y-3">
      {Array.from({ length: count }).map((_, index) => (
        <SkeletonVideoCard key={index} />
      ))}
    </div>
  );
}

export function SkeletonPlaylistCard() {
  return (
    <div className="card-compact border border-gray-600">
      <div className="bg-bg-secondary p-4 space-y-3">
        <SkeletonText className="w-2/3" />
        <SkeletonText className="w-1/3" />
      </div>
    </div>
  );
}
