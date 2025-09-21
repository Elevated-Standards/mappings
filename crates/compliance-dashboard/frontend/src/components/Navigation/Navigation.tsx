// Modified: 2025-09-21

import React, { useState } from 'react';
import './Navigation.css';

interface NavigationItem {
  id: string;
  label: string;
  href: string;
  icon?: React.ReactNode;
  active?: boolean;
  children?: NavigationItem[];
}

interface NavigationProps {
  items: NavigationItem[];
  className?: string;
  variant?: 'horizontal' | 'vertical';
  onItemClick?: (item: NavigationItem) => void;
}

export const Navigation: React.FC<NavigationProps> = ({
  items,
  className = '',
  variant = 'horizontal',
  onItemClick,
}) => {
  return (
    <nav className={`navigation navigation--${variant} ${className}`}>
      <ul className="navigation__list">
        {items.map((item) => (
          <NavigationItem
            key={item.id}
            item={item}
            onItemClick={onItemClick}
          />
        ))}
      </ul>
    </nav>
  );
};

interface NavigationItemProps {
  item: NavigationItem;
  onItemClick?: (item: NavigationItem) => void;
}

const NavigationItem: React.FC<NavigationItemProps> = ({
  item,
  onItemClick,
}) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const hasChildren = item.children && item.children.length > 0;

  const handleClick = (e: React.MouseEvent) => {
    if (hasChildren) {
      e.preventDefault();
      setIsExpanded(!isExpanded);
    }
    onItemClick?.(item);
  };

  return (
    <li className="navigation__item">
      <a
        href={item.href}
        className={`navigation__link ${item.active ? 'navigation__link--active' : ''}`}
        onClick={handleClick}
        aria-expanded={hasChildren ? isExpanded : undefined}
      >
        {item.icon && (
          <span className="navigation__icon">{item.icon}</span>
        )}
        <span className="navigation__label">{item.label}</span>
        {hasChildren && (
          <span className={`navigation__chevron ${isExpanded ? 'navigation__chevron--expanded' : ''}`}>
            ▼
          </span>
        )}
      </a>
      {hasChildren && (
        <ul className={`navigation__submenu ${isExpanded ? 'navigation__submenu--expanded' : ''}`}>
          {item.children!.map((child) => (
            <NavigationItem
              key={child.id}
              item={child}
              onItemClick={onItemClick}
            />
          ))}
        </ul>
      )}
    </li>
  );
};

interface MobileMenuProps {
  isOpen: boolean;
  onClose: () => void;
  children: React.ReactNode;
}

export const MobileMenu: React.FC<MobileMenuProps> = ({
  isOpen,
  onClose,
  children,
}) => {
  return (
    <>
      {isOpen && (
        <div className="mobile-menu__backdrop" onClick={onClose} />
      )}
      <div className={`mobile-menu ${isOpen ? 'mobile-menu--open' : ''}`}>
        <div className="mobile-menu__header">
          <button
            className="mobile-menu__close"
            onClick={onClose}
            aria-label="Close menu"
          >
            ✕
          </button>
        </div>
        <div className="mobile-menu__content">
          {children}
        </div>
      </div>
    </>
  );
};

interface HeaderProps {
  children: React.ReactNode;
  className?: string;
  sticky?: boolean;
}

export const Header: React.FC<HeaderProps> = ({
  children,
  className = '',
  sticky = true,
}) => {
  return (
    <header className={`header ${sticky ? 'header--sticky' : ''} ${className}`}>
      <div className="header__container">
        {children}
      </div>
    </header>
  );
};

interface SidebarProps {
  children: React.ReactNode;
  className?: string;
  isOpen?: boolean;
  onClose?: () => void;
  width?: 'sm' | 'md' | 'lg';
}

export const Sidebar: React.FC<SidebarProps> = ({
  children,
  className = '',
  isOpen = true,
  onClose,
  width = 'md',
}) => {
  return (
    <>
      {isOpen && (
        <div className="sidebar__backdrop lg:hidden" onClick={onClose} />
      )}
      <aside className={`sidebar sidebar--${width} ${isOpen ? 'sidebar--open' : ''} ${className}`}>
        <div className="sidebar__content">
          {children}
        </div>
      </aside>
    </>
  );
};

interface BreadcrumbProps {
  items: Array<{
    label: string;
    href?: string;
  }>;
  className?: string;
}

export const Breadcrumb: React.FC<BreadcrumbProps> = ({
  items,
  className = '',
}) => {
  return (
    <nav className={`breadcrumb ${className}`} aria-label="Breadcrumb">
      <ol className="breadcrumb__list">
        {items.map((item, index) => (
          <li key={index} className="breadcrumb__item">
            {item.href ? (
              <a href={item.href} className="breadcrumb__link">
                {item.label}
              </a>
            ) : (
              <span className="breadcrumb__current" aria-current="page">
                {item.label}
              </span>
            )}
            {index < items.length - 1 && (
              <span className="breadcrumb__separator" aria-hidden="true">
                /
              </span>
            )}
          </li>
        ))}
      </ol>
    </nav>
  );
};

interface MenuToggleProps {
  isOpen: boolean;
  onToggle: () => void;
  className?: string;
}

export const MenuToggle: React.FC<MenuToggleProps> = ({
  isOpen,
  onToggle,
  className = '',
}) => {
  return (
    <button
      className={`menu-toggle ${isOpen ? 'menu-toggle--open' : ''} ${className}`}
      onClick={onToggle}
      aria-label={isOpen ? 'Close menu' : 'Open menu'}
      aria-expanded={isOpen}
    >
      <span className="menu-toggle__line"></span>
      <span className="menu-toggle__line"></span>
      <span className="menu-toggle__line"></span>
    </button>
  );
};
