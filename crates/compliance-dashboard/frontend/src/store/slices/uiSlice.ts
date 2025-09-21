// Modified: 2025-09-21

import type { StateCreator } from 'zustand';
import type { Store, UIState } from '../types';
import type { FilterState, ErrorState } from '../../types';

// Initial UI state
const initialUIState: UIState = {
  selectedFramework: null,
  filters: {
    search: '',
    status: [],
    priority: [],
  },
  loading: false,
  errors: [],
  sidebarOpen: true,
  theme: 'system',
};

export const createUISlice: StateCreator<
  Store,
  [['zustand/immer', never]],
  [],
  { ui: UIState } & {
    setSelectedFramework: (frameworkId: string | null) => void;
    updateFilters: (filters: Partial<FilterState>) => void;
    clearFilters: () => void;
    setUILoading: (loading: boolean) => void;
    addError: (error: ErrorState) => void;
    removeError: (errorId: string) => void;
    clearErrors: () => void;
    toggleSidebar: () => void;
    setSidebarOpen: (open: boolean) => void;
    setTheme: (theme: 'light' | 'dark' | 'system') => void;
  }
> = (set, get) => ({
  // UI state
  ui: initialUIState,

  // UI actions
  setSelectedFramework: (frameworkId: string | null) => {
    set((state) => {
      state.ui.selectedFramework = frameworkId;
    });

    // Auto-load controls for the selected framework
    if (frameworkId) {
      const { loadControls } = get();
      loadControls(frameworkId).catch(console.error);
    }
  },

  updateFilters: (filters: Partial<FilterState>) => {
    set((state) => {
      state.ui.filters = {
        ...state.ui.filters,
        ...filters,
      };
    });
  },

  clearFilters: () => {
    set((state) => {
      state.ui.filters = {
        search: '',
        status: [],
        priority: [],
      };
    });
  },

  setUILoading: (loading: boolean) => {
    set((state) => {
      state.ui.loading = loading;
    });
  },

  addError: (error: ErrorState) => {
    set((state) => {
      // Prevent duplicate errors
      const exists = state.ui.errors.some(e => e.id === error.id);
      if (!exists) {
        state.ui.errors.push({
          ...error,
          timestamp: error.timestamp || new Date(),
        });
      }
    });

    // Auto-remove error after timeout if specified
    if (error.timeout) {
      setTimeout(() => {
        const { removeError } = get();
        removeError(error.id);
      }, error.timeout);
    }
  },

  removeError: (errorId: string) => {
    set((state) => {
      state.ui.errors = state.ui.errors.filter(e => e.id !== errorId);
    });
  },

  clearErrors: () => {
    set((state) => {
      state.ui.errors = [];
    });
  },

  toggleSidebar: () => {
    set((state) => {
      state.ui.sidebarOpen = !state.ui.sidebarOpen;
    });
  },

  setSidebarOpen: (open: boolean) => {
    set((state) => {
      state.ui.sidebarOpen = open;
    });
  },

  setTheme: (theme: 'light' | 'dark' | 'system') => {
    set((state) => {
      state.ui.theme = theme;
    });

    // Apply theme to document
    uiUtils.applyTheme(theme);
  },
});

// UI utility functions
export const uiUtils = {
  // Generate unique error ID
  generateErrorId: (): string => {
    return `error_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  },

  // Create error object with defaults
  createError: (
    message: string,
    type: 'error' | 'warning' | 'info' = 'error',
    timeout?: number
  ): ErrorState => ({
    id: uiUtils.generateErrorId(),
    message,
    type,
    timestamp: new Date(),
    timeout,
  }),

  // Apply theme to document
  applyTheme: (theme: 'light' | 'dark' | 'system') => {
    const root = document.documentElement;
    
    if (theme === 'system') {
      // Use system preference
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      root.setAttribute('data-theme', prefersDark ? 'dark' : 'light');
    } else {
      root.setAttribute('data-theme', theme);
    }
  },

  // Get current theme from system
  getSystemTheme: (): 'light' | 'dark' => {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  },

  // Format filter display text
  formatFilterText: (filters: FilterState): string => {
    const parts: string[] = [];

    if (filters.search) {
      parts.push(`Search: "${filters.search}"`);
    }

    if (filters.status && filters.status.length > 0) {
      parts.push(`Status: ${filters.status.join(', ')}`);
    }

    if (filters.priority && filters.priority.length > 0) {
      parts.push(`Priority: ${filters.priority.join(', ')}`);
    }

    return parts.join(' | ');
  },

  // Check if filters are active
  hasActiveFilters: (filters: FilterState): boolean => {
    return !!(
      filters.search ||
      (filters.status && filters.status.length > 0) ||
      (filters.priority && filters.priority.length > 0)
    );
  },

  // Debounce function for search input
  debounce: <T extends (...args: any[]) => any>(
    func: T,
    wait: number
  ): ((...args: Parameters<T>) => void) => {
    let timeout: NodeJS.Timeout;
    return (...args: Parameters<T>) => {
      clearTimeout(timeout);
      timeout = setTimeout(() => func(...args), wait);
    };
  },

  // Format error message for display
  formatErrorMessage: (error: ErrorState): string => {
    const timestamp = error.timestamp.toLocaleTimeString();
    return `[${timestamp}] ${error.message}`;
  },

  // Get error icon based on type
  getErrorIcon: (type: 'error' | 'warning' | 'info'): string => {
    switch (type) {
      case 'error':
        return '❌';
      case 'warning':
        return '⚠️';
      case 'info':
        return 'ℹ️';
      default:
        return '❌';
    }
  },

  // Handle responsive sidebar behavior
  handleResponsiveSidebar: (windowWidth: number): boolean => {
    // Auto-close sidebar on mobile
    return windowWidth >= 768;
  },

  // Local storage helpers for UI preferences
  saveUIPreferences: (preferences: Partial<UIState>) => {
    try {
      const existing = localStorage.getItem('ui-preferences');
      const current = existing ? JSON.parse(existing) : {};
      const updated = { ...current, ...preferences };
      localStorage.setItem('ui-preferences', JSON.stringify(updated));
    } catch (error) {
      console.warn('Failed to save UI preferences:', error);
    }
  },

  loadUIPreferences: (): Partial<UIState> => {
    try {
      const stored = localStorage.getItem('ui-preferences');
      return stored ? JSON.parse(stored) : {};
    } catch (error) {
      console.warn('Failed to load UI preferences:', error);
      return {};
    }
  },

  // Keyboard shortcut helpers
  handleKeyboardShortcuts: (event: KeyboardEvent, actions: any) => {
    // Ctrl/Cmd + K for search
    if ((event.ctrlKey || event.metaKey) && event.key === 'k') {
      event.preventDefault();
      // Focus search input
      const searchInput = document.querySelector('[data-search-input]') as HTMLInputElement;
      if (searchInput) {
        searchInput.focus();
      }
    }

    // Escape to clear filters
    if (event.key === 'Escape') {
      actions.clearFilters();
    }

    // Ctrl/Cmd + B to toggle sidebar
    if ((event.ctrlKey || event.metaKey) && event.key === 'b') {
      event.preventDefault();
      actions.toggleSidebar();
    }
  },
};

// Initialize theme on load
if (typeof window !== 'undefined') {
  // Listen for system theme changes
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
    const currentTheme = document.documentElement.getAttribute('data-theme');
    if (currentTheme === 'system' || !currentTheme) {
      uiUtils.applyTheme('system');
    }
  });
}
