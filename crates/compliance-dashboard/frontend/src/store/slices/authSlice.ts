// Modified: 2025-09-21

import type { StateCreator } from 'zustand';
import type { Store, AuthState } from '../types';
import type { User, Permission } from '../../types';

// Initial auth state
const initialAuthState: AuthState = {
  user: null,
  permissions: [],
  isAuthenticated: false,
  isLoading: false,
  error: null,
};

export const createAuthSlice: StateCreator<
  Store,
  [['zustand/immer', never]],
  [],
  { auth: AuthState } & {
    login: (email: string, password: string) => Promise<void>;
    logout: () => void;
    setUser: (user: User | null) => void;
    setPermissions: (permissions: Permission[]) => void;
    setAuthLoading: (loading: boolean) => void;
    setAuthError: (error: string | null) => void;
  }
> = (set, get) => ({
  // Auth state
  auth: initialAuthState,

  // Auth actions
  login: async (email: string, password: string) => {
    set((state) => {
      state.auth.isLoading = true;
      state.auth.error = null;
    });

    try {
      // Simulate API call - replace with actual authentication logic
      const response = await fetch('/api/auth/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ email, password }),
      });

      if (!response.ok) {
        throw new Error('Authentication failed');
      }

      const data = await response.json();
      const { user, permissions, token } = data;

      // Store token in localStorage or secure storage
      localStorage.setItem('auth_token', token);

      set((state) => {
        state.auth.user = user;
        state.auth.permissions = permissions;
        state.auth.isAuthenticated = true;
        state.auth.isLoading = false;
        state.auth.error = null;
      });
    } catch (error) {
      set((state) => {
        state.auth.isLoading = false;
        state.auth.error = error instanceof Error ? error.message : 'Login failed';
      });
      throw error;
    }
  },

  logout: () => {
    // Clear token from storage
    localStorage.removeItem('auth_token');
    
    set((state) => {
      state.auth.user = null;
      state.auth.permissions = [];
      state.auth.isAuthenticated = false;
      state.auth.isLoading = false;
      state.auth.error = null;
    });

    // Clear other sensitive state
    set((state) => {
      state.compliance.controls = [];
      state.compliance.frameworks = [];
      state.compliance.baselines = [];
      state.compliance.metrics = [];
      state.ui.errors = [];
    });
  },

  setUser: (user: User | null) => {
    set((state) => {
      state.auth.user = user;
      state.auth.isAuthenticated = !!user;
    });
  },

  setPermissions: (permissions: Permission[]) => {
    set((state) => {
      state.auth.permissions = permissions;
    });
  },

  setAuthLoading: (loading: boolean) => {
    set((state) => {
      state.auth.isLoading = loading;
    });
  },

  setAuthError: (error: string | null) => {
    set((state) => {
      state.auth.error = error;
    });
  },
});

// Auth utility functions
export const authUtils = {
  // Check if user has specific permission
  hasPermission: (permissions: Permission[], permissionName: string): boolean => {
    return permissions.some(p => p.name === permissionName || p.name === 'admin');
  },

  // Check if user has any of the specified permissions
  hasAnyPermission: (permissions: Permission[], permissionNames: string[]): boolean => {
    return permissionNames.some(name => authUtils.hasPermission(permissions, name));
  },

  // Check if user has all of the specified permissions
  hasAllPermissions: (permissions: Permission[], permissionNames: string[]): boolean => {
    return permissionNames.every(name => authUtils.hasPermission(permissions, name));
  },

  // Get user's role (assuming permissions include role information)
  getUserRole: (permissions: Permission[]): string | null => {
    const rolePermission = permissions.find(p => p.name.startsWith('role:'));
    return rolePermission ? rolePermission.name.replace('role:', '') : null;
  },

  // Check if token is expired (basic implementation)
  isTokenExpired: (token: string): boolean => {
    try {
      const payload = JSON.parse(atob(token.split('.')[1]));
      const currentTime = Date.now() / 1000;
      return payload.exp < currentTime;
    } catch {
      return true;
    }
  },

  // Get token from storage
  getStoredToken: (): string | null => {
    return localStorage.getItem('auth_token');
  },

  // Validate stored token and return user data if valid
  validateStoredAuth: async (): Promise<{ user: User; permissions: Permission[] } | null> => {
    const token = authUtils.getStoredToken();
    
    if (!token || authUtils.isTokenExpired(token)) {
      return null;
    }

    try {
      const response = await fetch('/api/auth/validate', {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      if (!response.ok) {
        throw new Error('Token validation failed');
      }

      const data = await response.json();
      return { user: data.user, permissions: data.permissions };
    } catch {
      // Clear invalid token
      localStorage.removeItem('auth_token');
      return null;
    }
  },
};

// Auth middleware for API requests
export const authMiddleware = {
  // Add auth headers to requests
  addAuthHeaders: (headers: Record<string, string> = {}): Record<string, string> => {
    const token = authUtils.getStoredToken();
    
    if (token) {
      return {
        ...headers,
        'Authorization': `Bearer ${token}`,
      };
    }
    
    return headers;
  },

  // Handle auth errors in API responses
  handleAuthError: (response: Response): void => {
    if (response.status === 401) {
      // Token expired or invalid, logout user
      const store = get();
      store.logout();
    }
  },
};
