// Modified: 2025-09-21

import React from 'react';
import type { Control, Priority } from '../../types';
import { StatusIndicator } from './StatusIndicator';
import './ControlCard.css';

interface ControlCardProps {
  control: Control;
  onClick?: (control: Control) => void;
  className?: string;
  compact?: boolean;
}

export const ControlCard: React.FC<ControlCardProps> = ({
  control,
  onClick,
  className = '',
  compact = false,
}) => {
  const handleClick = () => {
    onClick?.(control);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      onClick?.(control);
    }
  };

  return (
    <div
      className={`control-card ${compact ? 'control-card--compact' : ''} ${onClick ? 'control-card--clickable' : ''} ${className}`}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      tabIndex={onClick ? 0 : undefined}
      role={onClick ? 'button' : undefined}
      aria-label={onClick ? `View details for ${control.identifier}` : undefined}
    >
      <div className="control-card__header">
        <div className="control-card__identifier">
          <span className="control-card__id">{control.identifier}</span>
          <PriorityBadge priority={control.priority} />
        </div>
        <StatusIndicator 
          status={control.implementationStatus} 
          size="sm" 
          showLabel={!compact}
        />
      </div>
      
      <div className="control-card__content">
        <h3 className="control-card__title">{control.title}</h3>
        {!compact && (
          <p className="control-card__description">
            {control.description.length > 150 
              ? `${control.description.substring(0, 150)}...`
              : control.description
            }
          </p>
        )}
      </div>
      
      <div className="control-card__footer">
        <span className="control-card__category">{control.category}</span>
        <span className="control-card__updated">
          Updated {formatDate(control.lastUpdated)}
        </span>
      </div>
    </div>
  );
};

interface PriorityBadgeProps {
  priority: Priority;
  className?: string;
}

const PriorityBadge: React.FC<PriorityBadgeProps> = ({
  priority,
  className = '',
}) => {
  const priorityConfig = {
    high: { label: 'High', color: 'error' },
    medium: { label: 'Medium', color: 'warning' },
    low: { label: 'Low', color: 'success' },
  };

  const config = priorityConfig[priority as keyof typeof priorityConfig];

  return (
    <span 
      className={`priority-badge priority-badge--${config.color} ${className}`}
      aria-label={`Priority: ${config.label}`}
    >
      {config.label}
    </span>
  );
};

interface ControlGridProps {
  controls: Control[];
  onControlClick?: (control: Control) => void;
  className?: string;
  compact?: boolean;
  loading?: boolean;
}

export const ControlGrid: React.FC<ControlGridProps> = ({
  controls,
  onControlClick,
  className = '',
  compact = false,
  loading = false,
}) => {
  if (loading) {
    return (
      <div className={`control-grid ${className}`}>
        {Array.from({ length: 6 }).map((_, index) => (
          <ControlCardSkeleton key={index} compact={compact} />
        ))}
      </div>
    );
  }

  if (controls.length === 0) {
    return (
      <div className={`control-grid control-grid--empty ${className}`}>
        <div className="control-grid__empty-state">
          <span className="control-grid__empty-icon" aria-hidden="true">📋</span>
          <h3 className="control-grid__empty-title">No controls found</h3>
          <p className="control-grid__empty-description">
            Try adjusting your filters or search criteria.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className={`control-grid ${compact ? 'control-grid--compact' : ''} ${className}`}>
      {controls.map((control) => (
        <ControlCard
          key={control.id}
          control={control}
          onClick={onControlClick}
          compact={compact}
        />
      ))}
    </div>
  );
};

interface ControlCardSkeletonProps {
  compact?: boolean;
}

const ControlCardSkeleton: React.FC<ControlCardSkeletonProps> = ({
  compact = false,
}) => {
  return (
    <div className={`control-card control-card--skeleton ${compact ? 'control-card--compact' : ''}`}>
      <div className="control-card__header">
        <div className="control-card__identifier">
          <div className="skeleton skeleton--text skeleton--sm"></div>
          <div className="skeleton skeleton--badge"></div>
        </div>
        <div className="skeleton skeleton--status"></div>
      </div>
      
      <div className="control-card__content">
        <div className="skeleton skeleton--title"></div>
        {!compact && (
          <>
            <div className="skeleton skeleton--text"></div>
            <div className="skeleton skeleton--text skeleton--short"></div>
          </>
        )}
      </div>
      
      <div className="control-card__footer">
        <div className="skeleton skeleton--text skeleton--xs"></div>
        <div className="skeleton skeleton--text skeleton--xs"></div>
      </div>
    </div>
  );
};

// Utility function to format dates
function formatDate(date: Date): string {
  const now = new Date();
  const diffInMs = now.getTime() - date.getTime();
  const diffInDays = Math.floor(diffInMs / (1000 * 60 * 60 * 24));

  if (diffInDays === 0) {
    return 'today';
  } else if (diffInDays === 1) {
    return 'yesterday';
  } else if (diffInDays < 7) {
    return `${diffInDays} days ago`;
  } else if (diffInDays < 30) {
    const weeks = Math.floor(diffInDays / 7);
    return `${weeks} week${weeks > 1 ? 's' : ''} ago`;
  } else {
    return date.toLocaleDateString();
  }
}

export { PriorityBadge };
