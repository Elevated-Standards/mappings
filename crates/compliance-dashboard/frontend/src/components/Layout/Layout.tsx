// Modified: 2025-09-21

import React from 'react';
import './Layout.css';

interface LayoutProps {
  children: React.ReactNode;
  className?: string;
}

export const Layout: React.FC<LayoutProps> = ({ children, className = '' }) => {
  return (
    <div className={`layout ${className}`}>
      <div className="layout__container">
        {children}
      </div>
    </div>
  );
};

interface ContainerProps {
  children: React.ReactNode;
  className?: string;
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
}

export const Container: React.FC<ContainerProps> = ({ 
  children, 
  className = '', 
  size = 'lg' 
}) => {
  return (
    <div className={`container container--${size} ${className}`}>
      {children}
    </div>
  );
};

interface GridProps {
  children: React.ReactNode;
  className?: string;
  cols?: {
    mobile?: number;
    tablet?: number;
    desktop?: number;
  };
  gap?: 'sm' | 'md' | 'lg';
}

export const Grid: React.FC<GridProps> = ({ 
  children, 
  className = '', 
  cols = { mobile: 1, tablet: 2, desktop: 3 },
  gap = 'md'
}) => {
  const gridClasses = [
    'grid',
    `grid-cols-${cols.mobile}`,
    cols.tablet && `md:grid-cols-${cols.tablet}`,
    cols.desktop && `lg:grid-cols-${cols.desktop}`,
    `gap-${gap}`,
    className
  ].filter(Boolean).join(' ');

  return (
    <div className={gridClasses}>
      {children}
    </div>
  );
};

interface GridItemProps {
  children: React.ReactNode;
  className?: string;
  span?: {
    mobile?: number;
    tablet?: number;
    desktop?: number;
  };
}

export const GridItem: React.FC<GridItemProps> = ({ 
  children, 
  className = '', 
  span = { mobile: 1 }
}) => {
  const itemClasses = [
    'grid-item',
    span.mobile && `col-span-${span.mobile}`,
    span.tablet && `md:col-span-${span.tablet}`,
    span.desktop && `lg:col-span-${span.desktop}`,
    className
  ].filter(Boolean).join(' ');

  return (
    <div className={itemClasses}>
      {children}
    </div>
  );
};

interface FlexProps {
  children: React.ReactNode;
  className?: string;
  direction?: 'row' | 'col';
  align?: 'start' | 'center' | 'end' | 'stretch';
  justify?: 'start' | 'center' | 'end' | 'between' | 'around' | 'evenly';
  wrap?: boolean;
  gap?: 'sm' | 'md' | 'lg';
}

export const Flex: React.FC<FlexProps> = ({ 
  children, 
  className = '', 
  direction = 'row',
  align = 'start',
  justify = 'start',
  wrap = false,
  gap = 'md'
}) => {
  const flexClasses = [
    'flex',
    `flex-${direction}`,
    `items-${align}`,
    `justify-${justify}`,
    wrap && 'flex-wrap',
    `gap-${gap}`,
    className
  ].filter(Boolean).join(' ');

  return (
    <div className={flexClasses}>
      {children}
    </div>
  );
};

interface StackProps {
  children: React.ReactNode;
  className?: string;
  spacing?: 'sm' | 'md' | 'lg';
}

export const Stack: React.FC<StackProps> = ({ 
  children, 
  className = '', 
  spacing = 'md' 
}) => {
  return (
    <div className={`stack stack--${spacing} ${className}`}>
      {children}
    </div>
  );
};

interface ResponsiveProps {
  children: React.ReactNode;
  show?: {
    mobile?: boolean;
    tablet?: boolean;
    desktop?: boolean;
  };
}

export const Responsive: React.FC<ResponsiveProps> = ({ 
  children, 
  show = { mobile: true, tablet: true, desktop: true }
}) => {
  const responsiveClasses = [
    show.mobile === false && 'hidden',
    show.tablet === false && 'md:hidden',
    show.tablet === true && show.mobile === false && 'hidden md:block',
    show.desktop === false && 'lg:hidden',
    show.desktop === true && (show.mobile === false || show.tablet === false) && 'hidden lg:block'
  ].filter(Boolean).join(' ');

  return (
    <div className={responsiveClasses}>
      {children}
    </div>
  );
};
