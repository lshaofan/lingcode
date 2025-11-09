import React from 'react';

interface ProgressBarProps {
  progress: number; // 0-100
  label?: string;
  showPercentage?: boolean;
  className?: string;
  size?: 'sm' | 'md' | 'lg';
}

export const ProgressBar: React.FC<ProgressBarProps> = ({
  progress,
  label,
  showPercentage = true,
  className = '',
  size = 'md',
}) => {
  const clampedProgress = Math.max(0, Math.min(100, progress));

  const heightClasses = {
    sm: 'h-1',
    md: 'h-2',
    lg: 'h-3',
  };

  return (
    <div className={`w-full ${className}`}>
      {(label || showPercentage) && (
        <div className="flex justify-between items-center mb-1">
          {label && <span className="text-sm text-gray-700">{label}</span>}
          {showPercentage && (
            <span className="text-sm font-medium text-gray-900">{Math.round(clampedProgress)}%</span>
          )}
        </div>
      )}
      <div className={`w-full bg-gray-200 rounded-full overflow-hidden ${heightClasses[size]}`}>
        <div
          className="h-full bg-green-500 transition-all duration-300 ease-in-out"
          style={{ width: `${clampedProgress}%` }}
        />
      </div>
    </div>
  );
};
