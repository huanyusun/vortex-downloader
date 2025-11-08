// Success feedback component with animations and statistics

import { useEffect, useState } from 'react';

interface SuccessFeedbackProps {
  count: number;
  message: string;
  onComplete?: () => void;
  duration?: number;
}

export default function SuccessFeedback({
  count,
  message,
  onComplete,
  duration = 2000,
}: SuccessFeedbackProps) {
  const [isVisible, setIsVisible] = useState(true);

  useEffect(() => {
    const timer = setTimeout(() => {
      setIsVisible(false);
      if (onComplete) {
        setTimeout(onComplete, 300); // Wait for fade out animation
      }
    }, duration);

    return () => clearTimeout(timer);
  }, [duration, onComplete]);

  if (!isVisible) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center pointer-events-none">
      <div className="animate-bounce-in pointer-events-auto">
        <div className="bg-success/90 backdrop-blur-sm text-white rounded-2xl shadow-2xl p-8 flex flex-col items-center gap-4 animate-pulse-success">
          {/* Success Icon */}
          <div className="w-16 h-16 bg-white rounded-full flex items-center justify-center">
            <svg className="w-10 h-10 text-success" fill="currentColor" viewBox="0 0 20 20">
              <path
                fillRule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                clipRule="evenodd"
              />
            </svg>
          </div>

          {/* Count Display */}
          <div className="text-center">
            <div className="text-5xl font-bold mb-2">{count}</div>
            <div className="text-lg font-medium">{message}</div>
          </div>
        </div>
      </div>
    </div>
  );
}
