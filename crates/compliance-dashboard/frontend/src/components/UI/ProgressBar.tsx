// Modified: 2025-09-21

import React from 'react';
import type { ImplementationStatus } from '../../types';
import './ProgressBar.css';

interface ProgressBarProps {
  percentage: number;
  status?: ImplementationStatus;
  size?: 'sm' | 'md' | 'lg';
  showPercentage?: boolean;
  label?: string;
  className?: string;
  animated?: boolean;
}

export const ProgressBar: React.FC<ProgressBarProps> = ({
  percentage,
  status = 'implemented',
  size = 'md',
  showPercentage = true,
  label,
  className = '',
  animated = true,
}) => {
  const clampedPercentage = Math.max(0, Math.min(100, percentage));
  
  const statusColorMap = {
    implemented: 'success',
    partial: 'warning',
    not_implemented: 'error',
    not_applicable: 'neutral',
  };

  return (
    <div className={`progress-bar-container ${className}`}>
      {(label || showPercentage) && (
        <div className="progress-bar__header">
          {label && (
            <span className="progress-bar__label">{label}</span>
          )}
          {showPercentage && (
            <span className="progress-bar__percentage">
              {clampedPercentage}%
            </span>
          )}
        </div>
      )}
      <div 
        className={`progress-bar progress-bar--${size} progress-bar--${statusColorMap[status as keyof typeof statusColorMap]}`}
        role="progressbar"
        aria-valuenow={clampedPercentage}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-label={label || `Progress: ${clampedPercentage}%`}
      >
        <div 
          className={`progress-bar__fill ${animated ? 'progress-bar__fill--animated' : ''}`}
          style={{ width: `${clampedPercentage}%` }}
        />
      </div>
    </div>
  );
};

interface MultiProgressBarProps {
  segments: Array<{
    percentage: number;
    status: ImplementationStatus;
    label: string;
    count?: number;
  }>;
  size?: 'sm' | 'md' | 'lg';
  showLegend?: boolean;
  className?: string;
}

export const MultiProgressBar: React.FC<MultiProgressBarProps> = ({
  segments,
  size = 'md',
  showLegend = true,
  className = '',
}) => {
  const total = segments.reduce((sum, segment) => sum + segment.percentage, 0);
  const normalizedSegments = segments.map(segment => ({
    ...segment,
    normalizedPercentage: total > 0 ? (segment.percentage / total) * 100 : 0,
  }));

  const statusColorMap = {
    implemented: 'success',
    partial: 'warning',
    not_implemented: 'error',
    not_applicable: 'neutral',
  };

  return (
    <div className={`multi-progress-bar-container ${className}`}>
      <div 
        className={`multi-progress-bar multi-progress-bar--${size}`}
        role="progressbar"
        aria-label="Compliance progress breakdown"
      >
        {normalizedSegments.map((segment, index) => (
          <div
            key={index}
            className={`multi-progress-bar__segment multi-progress-bar__segment--${statusColorMap[segment.status as keyof typeof statusColorMap]}`}
            style={{ width: `${segment.normalizedPercentage}%` }}
            title={`${segment.label}: ${segment.percentage}%${segment.count ? ` (${segment.count} items)` : ''}`}
          />
        ))}
      </div>
      
      {showLegend && (
        <div className="multi-progress-bar__legend">
          {segments.map((segment, index) => (
            <div key={index} className="multi-progress-bar__legend-item">
              <div 
                className={`multi-progress-bar__legend-color multi-progress-bar__legend-color--${statusColorMap[segment.status as keyof typeof statusColorMap]}`}
              />
              <span className="multi-progress-bar__legend-label">
                {segment.label}
              </span>
              <span className="multi-progress-bar__legend-value">
                {segment.percentage}%
                {segment.count && ` (${segment.count})`}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

interface CircularProgressProps {
  percentage: number;
  size?: number;
  strokeWidth?: number;
  status?: ImplementationStatus;
  showPercentage?: boolean;
  label?: string;
  className?: string;
}

export const CircularProgress: React.FC<CircularProgressProps> = ({
  percentage,
  size = 120,
  strokeWidth = 8,
  status = 'implemented',
  showPercentage = true,
  label,
  className = '',
}) => {
  const clampedPercentage = Math.max(0, Math.min(100, percentage));
  const radius = (size - strokeWidth) / 2;
  const circumference = 2 * Math.PI * radius;
  const strokeDashoffset = circumference - (clampedPercentage / 100) * circumference;

  const statusColorMap = {
    implemented: 'success',
    partial: 'warning',
    not_implemented: 'error',
    not_applicable: 'neutral',
  };

  return (
    <div className={`circular-progress ${className}`}>
      <svg
        width={size}
        height={size}
        className="circular-progress__svg"
        role="img"
        aria-label={label || `Progress: ${clampedPercentage}%`}
      >
        {/* Background circle */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          className="circular-progress__background"
          strokeWidth={strokeWidth}
          fill="none"
        />
        {/* Progress circle */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          className={`circular-progress__progress circular-progress__progress--${statusColorMap[status as keyof typeof statusColorMap]}`}
          strokeWidth={strokeWidth}
          fill="none"
          strokeDasharray={circumference}
          strokeDashoffset={strokeDashoffset}
          transform={`rotate(-90 ${size / 2} ${size / 2})`}
        />
      </svg>
      
      <div className="circular-progress__content">
        {showPercentage && (
          <span className="circular-progress__percentage">
            {clampedPercentage}%
          </span>
        )}
        {label && (
          <span className="circular-progress__label">
            {label}
          </span>
        )}
      </div>
    </div>
  );
};

interface ProgressStepsProps {
  steps: Array<{
    label: string;
    status: 'completed' | 'current' | 'pending';
    description?: string;
  }>;
  orientation?: 'horizontal' | 'vertical';
  className?: string;
}

export const ProgressSteps: React.FC<ProgressStepsProps> = ({
  steps,
  orientation = 'horizontal',
  className = '',
}) => {
  return (
    <div className={`progress-steps progress-steps--${orientation} ${className}`}>
      {steps.map((step, index) => (
        <div key={index} className={`progress-step progress-step--${step.status}`}>
          <div className="progress-step__indicator">
            <div className="progress-step__circle">
              {step.status === 'completed' && (
                <span className="progress-step__check" aria-hidden="true">✓</span>
              )}
              {step.status === 'current' && (
                <span className="progress-step__current" aria-hidden="true"></span>
              )}
              {step.status === 'pending' && (
                <span className="progress-step__number">{index + 1}</span>
              )}
            </div>
            {index < steps.length - 1 && (
              <div className="progress-step__connector" />
            )}
          </div>
          <div className="progress-step__content">
            <div className="progress-step__label">{step.label}</div>
            {step.description && (
              <div className="progress-step__description">{step.description}</div>
            )}
          </div>
        </div>
      ))}
    </div>
  );
};
