// Modified: 2025-09-21

import React from 'react';
import './AlertBadge.css';

interface AlertBadgeProps {
  type: 'info' | 'success' | 'warning' | 'error';
  children: React.ReactNode;
  size?: 'sm' | 'md' | 'lg';
  variant?: 'filled' | 'outlined' | 'subtle';
  dismissible?: boolean;
  onDismiss?: () => void;
  className?: string;
  icon?: React.ReactNode;
}

export const AlertBadge: React.FC<AlertBadgeProps> = ({
  type,
  children,
  size = 'md',
  variant = 'filled',
  dismissible = false,
  onDismiss,
  className = '',
  icon,
}) => {
  const defaultIcons = {
    info: 'ℹ',
    success: '✓',
    warning: '⚠',
    error: '✗',
  };

  const displayIcon = icon || defaultIcons[type];

  return (
    <div
      className={`alert-badge alert-badge--${type} alert-badge--${variant} alert-badge--${size} ${className}`}
      role="alert"
      aria-live="polite"
    >
      {displayIcon && (
        <span className="alert-badge__icon" aria-hidden="true">
          {displayIcon}
        </span>
      )}
      <span className="alert-badge__content">{children}</span>
      {dismissible && (
        <button
          className="alert-badge__dismiss"
          onClick={onDismiss}
          aria-label="Dismiss alert"
          type="button"
        >
          ✕
        </button>
      )}
    </div>
  );
};

interface NotificationBadgeProps {
  count: number;
  max?: number;
  size?: 'sm' | 'md' | 'lg';
  variant?: 'dot' | 'count';
  className?: string;
  children?: React.ReactNode;
}

export const NotificationBadge: React.FC<NotificationBadgeProps> = ({
  count,
  max = 99,
  size = 'md',
  variant = 'count',
  className = '',
  children,
}) => {
  const displayCount = count > max ? `${max}+` : count.toString();
  const showBadge = count > 0;

  if (!showBadge && !children) {
    return null;
  }

  return (
    <div className={`notification-badge-container ${className}`}>
      {children}
      {showBadge && (
        <span
          className={`notification-badge notification-badge--${size} notification-badge--${variant}`}
          aria-label={`${count} notifications`}
        >
          {variant === 'count' && (
            <span className="notification-badge__count">{displayCount}</span>
          )}
        </span>
      )}
    </div>
  );
};

interface StatusBannerProps {
  type: 'info' | 'success' | 'warning' | 'error';
  title?: string;
  children: React.ReactNode;
  actions?: React.ReactNode;
  dismissible?: boolean;
  onDismiss?: () => void;
  className?: string;
}

export const StatusBanner: React.FC<StatusBannerProps> = ({
  type,
  title,
  children,
  actions,
  dismissible = false,
  onDismiss,
  className = '',
}) => {
  const icons = {
    info: 'ℹ',
    success: '✓',
    warning: '⚠',
    error: '✗',
  };

  return (
    <div
      className={`status-banner status-banner--${type} ${className}`}
      role="alert"
      aria-live="polite"
    >
      <div className="status-banner__icon" aria-hidden="true">
        {icons[type]}
      </div>
      <div className="status-banner__content">
        {title && <div className="status-banner__title">{title}</div>}
        <div className="status-banner__message">{children}</div>
      </div>
      {actions && <div className="status-banner__actions">{actions}</div>}
      {dismissible && (
        <button
          className="status-banner__dismiss"
          onClick={onDismiss}
          aria-label="Dismiss banner"
          type="button"
        >
          ✕
        </button>
      )}
    </div>
  );
};

interface TooltipProps {
  content: React.ReactNode;
  children: React.ReactNode;
  position?: 'top' | 'bottom' | 'left' | 'right';
  trigger?: 'hover' | 'click' | 'focus';
  className?: string;
}

export const Tooltip: React.FC<TooltipProps> = ({
  content,
  children,
  position = 'top',
  trigger = 'hover',
  className = '',
}) => {
  const [isVisible, setIsVisible] = React.useState(false);
  const [shouldShow, setShouldShow] = React.useState(false);
  const timeoutRef = React.useRef<NodeJS.Timeout | null>(null);

  const showTooltip = () => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    setShouldShow(true);
    timeoutRef.current = setTimeout(() => setIsVisible(true), 100);
  };

  const hideTooltip = () => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    setIsVisible(false);
    timeoutRef.current = setTimeout(() => setShouldShow(false), 150);
  };

  const handleClick = () => {
    if (trigger === 'click') {
      if (isVisible) {
        hideTooltip();
      } else {
        showTooltip();
      }
    }
  };

  React.useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  return (
    <div className={`tooltip-container ${className}`}>
      <div
        className="tooltip__trigger"
        onMouseEnter={trigger === 'hover' ? showTooltip : undefined}
        onMouseLeave={trigger === 'hover' ? hideTooltip : undefined}
        onFocus={trigger === 'focus' ? showTooltip : undefined}
        onBlur={trigger === 'focus' ? hideTooltip : undefined}
        onClick={handleClick}
      >
        {children}
      </div>
      {shouldShow && (
        <div
          className={`tooltip tooltip--${position} ${isVisible ? 'tooltip--visible' : ''}`}
          role="tooltip"
          aria-hidden={!isVisible}
        >
          <div className="tooltip__content">{content}</div>
          <div className="tooltip__arrow" />
        </div>
      )}
    </div>
  );
};

interface BadgeProps {
  children: React.ReactNode;
  variant?: 'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'info';
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

export const Badge: React.FC<BadgeProps> = ({
  children,
  variant = 'primary',
  size = 'md',
  className = '',
}) => {
  return (
    <span className={`badge badge--${variant} badge--${size} ${className}`}>
      {children}
    </span>
  );
};


