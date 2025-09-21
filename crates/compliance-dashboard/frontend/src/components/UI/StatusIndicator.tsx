// Modified: 2025-09-21

import React from 'react';
import type { ImplementationStatus } from '../../types';
import './StatusIndicator.css';

interface StatusIndicatorProps {
  status: ImplementationStatus;
  size?: 'sm' | 'md' | 'lg';
  showLabel?: boolean;
  className?: string;
}

const statusConfig = {
  implemented: {
    label: 'Implemented',
    icon: '✓',
    color: 'success',
  },
  not_implemented: {
    label: 'Not Implemented',
    icon: '✗',
    color: 'error',
  },
  partial: {
    label: 'Partial',
    icon: '◐',
    color: 'warning',
  },
  not_applicable: {
    label: 'Not Applicable',
    icon: '—',
    color: 'neutral',
  },
};

export const StatusIndicator: React.FC<StatusIndicatorProps> = ({
  status,
  size = 'md',
  showLabel = true,
  className = '',
}) => {
  const config = statusConfig[status as keyof typeof statusConfig];
  
  return (
    <div 
      className={`status-indicator status-indicator--${size} status-indicator--${config.color} ${className}`}
      role="status"
      aria-label={`Status: ${config.label}`}
    >
      <span className="status-indicator__icon" aria-hidden="true">
        {config.icon}
      </span>
      {showLabel && (
        <span className="status-indicator__label">
          {config.label}
        </span>
      )}
    </div>
  );
};

interface StatusBadgeProps {
  status: ImplementationStatus;
  count?: number;
  className?: string;
}

export const StatusBadge: React.FC<StatusBadgeProps> = ({
  status,
  count,
  className = '',
}) => {
  const config = statusConfig[status as keyof typeof statusConfig];
  
  return (
    <span 
      className={`status-badge status-badge--${config.color} ${className}`}
      role="status"
      aria-label={count ? `${config.label}: ${count} items` : config.label}
    >
      <span className="status-badge__icon" aria-hidden="true">
        {config.icon}
      </span>
      <span className="status-badge__label">
        {config.label}
      </span>
      {count !== undefined && (
        <span className="status-badge__count">
          {count}
        </span>
      )}
    </span>
  );
};

interface StatusListProps {
  statuses: Array<{
    status: ImplementationStatus;
    count: number;
  }>;
  className?: string;
}

export const StatusList: React.FC<StatusListProps> = ({
  statuses,
  className = '',
}) => {
  const total = statuses.reduce((sum, item) => sum + item.count, 0);
  
  return (
    <div className={`status-list ${className}`} role="list">
      {statuses.map(({ status, count }) => {
        const percentage = total > 0 ? Math.round((count / total) * 100) : 0;
        
        return (
          <div key={status} className="status-list__item" role="listitem">
            <StatusBadge status={status} count={count} />
            <span className="status-list__percentage" aria-label={`${percentage} percent`}>
              {percentage}%
            </span>
          </div>
        );
      })}
    </div>
  );
};

interface ProgressRingProps {
  percentage: number;
  size?: 'sm' | 'md' | 'lg';
  status?: ImplementationStatus;
  showPercentage?: boolean;
  className?: string;
}

export const ProgressRing: React.FC<ProgressRingProps> = ({
  percentage,
  size = 'md',
  status = 'implemented',
  showPercentage = true,
  className = '',
}) => {
  const config = statusConfig[status as keyof typeof statusConfig];
  const radius = size === 'sm' ? 16 : size === 'md' ? 20 : 24;
  const circumference = 2 * Math.PI * radius;
  const strokeDashoffset = circumference - (percentage / 100) * circumference;
  
  return (
    <div className={`progress-ring progress-ring--${size} ${className}`}>
      <svg 
        className="progress-ring__svg" 
        width={radius * 2 + 8} 
        height={radius * 2 + 8}
        role="img"
        aria-label={`Progress: ${percentage}%`}
      >
        <circle
          className="progress-ring__background"
          cx={radius + 4}
          cy={radius + 4}
          r={radius}
          fill="none"
          strokeWidth="2"
        />
        <circle
          className={`progress-ring__progress progress-ring__progress--${config.color}`}
          cx={radius + 4}
          cy={radius + 4}
          r={radius}
          fill="none"
          strokeWidth="2"
          strokeDasharray={circumference}
          strokeDashoffset={strokeDashoffset}
          transform={`rotate(-90 ${radius + 4} ${radius + 4})`}
        />
      </svg>
      {showPercentage && (
        <span className="progress-ring__text">
          {percentage}%
        </span>
      )}
    </div>
  );
};
