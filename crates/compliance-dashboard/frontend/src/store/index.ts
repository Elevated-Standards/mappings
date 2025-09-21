// Modified: 2025-09-21

import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';
import { persist, createJSONStorage } from 'zustand/middleware';
import { subscribeWithSelector } from 'zustand/middleware';
import type { Store } from './types';
import { createAuthSlice } from './slices/authSlice';
import { createComplianceSlice } from './slices/complianceSlice';
import { createUISlice } from './slices/uiSlice';
import { createRealtimeSlice } from './slices/realtimeSlice';

// Create the main store with all slices combined
export const useStore = create<Store>()(
  subscribeWithSelector(
    persist(
      immer((set, get, api) => ({
        // Combine all slices
        ...createAuthSlice(set, get, api),
        ...createComplianceSlice(set, get, api),
        ...createUISlice(set, get, api),
        ...createRealtimeSlice(set, get, api),
      })),
      {
        name: 'compliance-dashboard-store',
        storage: createJSONStorage(() => localStorage),
        // Only persist certain parts of the state
        partialize: (state) => ({
          auth: {
            user: state.auth.user,
            permissions: state.auth.permissions,
            isAuthenticated: state.auth.isAuthenticated,
          },
          ui: {
            selectedFramework: state.ui.selectedFramework,
            filters: state.ui.filters,
            sidebarOpen: state.ui.sidebarOpen,
            theme: state.ui.theme,
          },
          // Don't persist compliance data, realtime state, or loading states
        }),
        // Merge persisted state with initial state
        merge: (persistedState, currentState) => ({
          ...currentState,
          ...persistedState,
        }),
      }
    )
  )
);

// Selector hooks for better performance and type safety
export const useAuth = () => useStore((state) => state.auth);
export const useCompliance = () => useStore((state) => state.compliance);
export const useUI = () => useStore((state) => state.ui);
export const useRealtime = () => useStore((state) => state.realtime);

// Specific selector hooks
export const useIsAuthenticated = () => useStore((state) => state.auth.isAuthenticated);
export const useCurrentUser = () => useStore((state) => state.auth.user);
export const useUserPermissions = () => useStore((state) => state.auth.permissions);

export const useFrameworks = () => useStore((state) => state.compliance.frameworks);
export const useControls = () => useStore((state) => state.compliance.controls);
export const useBaselines = () => useStore((state) => state.compliance.baselines);
export const useMetrics = () => useStore((state) => state.compliance.metrics);

export const useSelectedFramework = () => useStore((state) => state.ui.selectedFramework);
export const useFilters = () => useStore((state) => state.ui.filters);
export const useTheme = () => useStore((state) => state.ui.theme);
export const useSidebarOpen = () => useStore((state) => state.ui.sidebarOpen);

export const useConnectionStatus = () => useStore((state) => state.realtime.connectionStatus);
export const useLastUpdate = () => useStore((state) => state.realtime.lastUpdate);

// Action hooks
export const useAuthActions = () => useStore((state) => ({
  login: state.login,
  logout: state.logout,
  setUser: state.setUser,
  setPermissions: state.setPermissions,
  setAuthLoading: state.setAuthLoading,
  setAuthError: state.setAuthError,
}));

export const useComplianceActions = () => useStore((state) => ({
  loadFrameworks: state.loadFrameworks,
  loadControls: state.loadControls,
  loadBaselines: state.loadBaselines,
  loadMetrics: state.loadMetrics,
  updateControl: state.updateControl,
  setFrameworks: state.setFrameworks,
  setControls: state.setControls,
  setBaselines: state.setBaselines,
  setMetrics: state.setMetrics,
  setComplianceLoading: state.setComplianceLoading,
  setComplianceError: state.setComplianceError,
  updateLastUpdated: state.updateLastUpdated,
}));

export const useUIActions = () => useStore((state) => ({
  setSelectedFramework: state.setSelectedFramework,
  updateFilters: state.updateFilters,
  clearFilters: state.clearFilters,
  setUILoading: state.setUILoading,
  addError: state.addError,
  removeError: state.removeError,
  clearErrors: state.clearErrors,
  toggleSidebar: state.toggleSidebar,
  setSidebarOpen: state.setSidebarOpen,
  setTheme: state.setTheme,
}));

export const useRealtimeActions = () => useStore((state) => ({
  setConnectionStatus: state.setConnectionStatus,
  updateLastUpdate: state.updateLastUpdate,
  incrementReconnectAttempts: state.incrementReconnectAttempts,
  resetReconnectAttempts: state.resetReconnectAttempts,
  setMaxReconnectAttempts: state.setMaxReconnectAttempts,
}));

// Computed selectors
export const useHasPermission = (permission: string) =>
  useStore((state) => 
    state.auth.permissions.some(p => p.name === permission || p.name === 'admin')
  );

export const useFrameworkById = (id: string) =>
  useStore((state) => 
    state.compliance.frameworks.find(f => f.id === id)
  );

export const useControlById = (id: string) =>
  useStore((state) => 
    state.compliance.controls.find(c => c.id === id)
  );

export const useControlsByFramework = (frameworkId: string) =>
  useStore((state) => 
    state.compliance.controls.filter(c => c.frameworkId === frameworkId)
  );

export const useFilteredControls = () =>
  useStore((state) => {
    let controls = state.compliance.controls;
    const filters = state.ui.filters;

    // Apply framework filter
    if (state.ui.selectedFramework) {
      controls = controls.filter(c => c.frameworkId === state.ui.selectedFramework);
    }

    // Apply status filter
    if (filters.status && filters.status.length > 0) {
      controls = controls.filter(c => filters.status!.includes(c.implementationStatus));
    }

    // Apply priority filter
    if (filters.priority && filters.priority.length > 0) {
      controls = controls.filter(c => filters.priority!.includes(c.priority));
    }

    // Apply search filter
    if (filters.search) {
      const searchLower = filters.search.toLowerCase();
      controls = controls.filter(c => 
        c.title.toLowerCase().includes(searchLower) ||
        c.description.toLowerCase().includes(searchLower) ||
        c.id.toLowerCase().includes(searchLower)
      );
    }

    return controls;
  });

export const useComplianceMetrics = () =>
  useStore((state) => {
    const controls = state.ui.selectedFramework
      ? state.compliance.controls.filter(c => c.frameworkId === state.ui.selectedFramework)
      : state.compliance.controls;

    const total = controls.length;
    const implemented = controls.filter(c => c.implementationStatus === 'implemented').length;
    const partial = controls.filter(c => c.implementationStatus === 'partial').length;
    const notImplemented = controls.filter(c => c.implementationStatus === 'not_implemented').length;
    const notApplicable = controls.filter(c => c.implementationStatus === 'not_applicable').length;

    const percentageComplete = total > 0 
      ? Math.round(((implemented + (partial * 0.5)) / total) * 100)
      : 0;

    return {
      total,
      implemented,
      partial,
      notImplemented,
      notApplicable,
      percentageComplete,
    };
  });

export const useHasActiveFilters = () =>
  useStore((state) => {
    const filters = state.ui.filters;
    return !!(
      filters.search ||
      (filters.status && filters.status.length > 0) ||
      (filters.priority && filters.priority.length > 0)
    );
  });

export const useIsConnected = () =>
  useStore((state) => state.realtime.connectionStatus === 'connected');

export const useShouldReconnect = () =>
  useStore((state) => 
    state.realtime.connectionStatus === 'disconnected' &&
    state.realtime.reconnectAttempts < state.realtime.maxReconnectAttempts
  );

// Export the store type for external use
export type { Store } from './types';

// Export utility functions
export { authUtils, authMiddleware } from './slices/authSlice';
export { complianceUtils } from './slices/complianceSlice';
export { uiUtils } from './slices/uiSlice';
export { realtimeUtils, WebSocketManager } from './slices/realtimeSlice';
