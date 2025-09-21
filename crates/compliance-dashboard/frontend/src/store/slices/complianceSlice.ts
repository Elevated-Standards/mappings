// Modified: 2025-09-21

import type { StateCreator } from 'zustand';
import type { Store, ComplianceState } from '../types';
import type { Framework, Control, Baseline, Metric } from '../../types';
import { authMiddleware } from './authSlice';

// Initial compliance state
const initialComplianceState: ComplianceState = {
  frameworks: [],
  controls: [],
  baselines: [],
  metrics: [],
  isLoading: false,
  error: null,
  lastUpdated: null,
};

export const createComplianceSlice: StateCreator<
  Store,
  [['zustand/immer', never]],
  [],
  { compliance: ComplianceState } & {
    loadFrameworks: () => Promise<void>;
    loadControls: (frameworkId?: string) => Promise<void>;
    loadBaselines: () => Promise<void>;
    loadMetrics: () => Promise<void>;
    updateControl: (controlId: string, updates: Partial<Control>) => Promise<void>;
    setFrameworks: (frameworks: Framework[]) => void;
    setControls: (controls: Control[]) => void;
    setBaselines: (baselines: Baseline[]) => void;
    setMetrics: (metrics: Metric[]) => void;
    setComplianceLoading: (loading: boolean) => void;
    setComplianceError: (error: string | null) => void;
    updateLastUpdated: () => void;
  }
> = (set, get) => ({
  // Compliance state
  compliance: initialComplianceState,

  // Compliance actions
  loadFrameworks: async () => {
    set((state) => {
      state.compliance.isLoading = true;
      state.compliance.error = null;
    });

    try {
      const response = await fetch('/api/frameworks', {
        headers: authMiddleware.addAuthHeaders(),
      });

      authMiddleware.handleAuthError(response);

      if (!response.ok) {
        throw new Error('Failed to load frameworks');
      }

      const frameworks: Framework[] = await response.json();

      set((state) => {
        state.compliance.frameworks = frameworks;
        state.compliance.isLoading = false;
        state.compliance.lastUpdated = new Date();
      });
    } catch (error) {
      set((state) => {
        state.compliance.isLoading = false;
        state.compliance.error = error instanceof Error ? error.message : 'Failed to load frameworks';
      });
      throw error;
    }
  },

  loadControls: async (frameworkId?: string) => {
    set((state) => {
      state.compliance.isLoading = true;
      state.compliance.error = null;
    });

    try {
      const url = frameworkId 
        ? `/api/controls?framework=${frameworkId}`
        : '/api/controls';

      const response = await fetch(url, {
        headers: authMiddleware.addAuthHeaders(),
      });

      authMiddleware.handleAuthError(response);

      if (!response.ok) {
        throw new Error('Failed to load controls');
      }

      const controls: Control[] = await response.json();

      set((state) => {
        if (frameworkId) {
          // Replace controls for specific framework
          state.compliance.controls = state.compliance.controls.filter(c => c.frameworkId !== frameworkId);
          state.compliance.controls.push(...controls);
        } else {
          // Replace all controls
          state.compliance.controls = controls;
        }
        state.compliance.isLoading = false;
        state.compliance.lastUpdated = new Date();
      });
    } catch (error) {
      set((state) => {
        state.compliance.isLoading = false;
        state.compliance.error = error instanceof Error ? error.message : 'Failed to load controls';
      });
      throw error;
    }
  },

  loadBaselines: async () => {
    set((state) => {
      state.compliance.isLoading = true;
      state.compliance.error = null;
    });

    try {
      const response = await fetch('/api/baselines', {
        headers: authMiddleware.addAuthHeaders(),
      });

      authMiddleware.handleAuthError(response);

      if (!response.ok) {
        throw new Error('Failed to load baselines');
      }

      const baselines: Baseline[] = await response.json();

      set((state) => {
        state.compliance.baselines = baselines;
        state.compliance.isLoading = false;
        state.compliance.lastUpdated = new Date();
      });
    } catch (error) {
      set((state) => {
        state.compliance.isLoading = false;
        state.compliance.error = error instanceof Error ? error.message : 'Failed to load baselines';
      });
      throw error;
    }
  },

  loadMetrics: async () => {
    set((state) => {
      state.compliance.isLoading = true;
      state.compliance.error = null;
    });

    try {
      const response = await fetch('/api/metrics', {
        headers: authMiddleware.addAuthHeaders(),
      });

      authMiddleware.handleAuthError(response);

      if (!response.ok) {
        throw new Error('Failed to load metrics');
      }

      const metrics: Metric[] = await response.json();

      set((state) => {
        state.compliance.metrics = metrics;
        state.compliance.isLoading = false;
        state.compliance.lastUpdated = new Date();
      });
    } catch (error) {
      set((state) => {
        state.compliance.isLoading = false;
        state.compliance.error = error instanceof Error ? error.message : 'Failed to load metrics';
      });
      throw error;
    }
  },

  updateControl: async (controlId: string, updates: Partial<Control>) => {
    try {
      const response = await fetch(`/api/controls/${controlId}`, {
        method: 'PATCH',
        headers: authMiddleware.addAuthHeaders({
          'Content-Type': 'application/json',
        }),
        body: JSON.stringify(updates),
      });

      authMiddleware.handleAuthError(response);

      if (!response.ok) {
        throw new Error('Failed to update control');
      }

      const updatedControl: Control = await response.json();

      set((state) => {
        const index = state.compliance.controls.findIndex(c => c.id === controlId);
        if (index !== -1) {
          state.compliance.controls[index] = updatedControl;
        }
        state.compliance.lastUpdated = new Date();
      });

      // Return void as expected by the interface
    } catch (error) {
      set((state) => {
        state.compliance.error = error instanceof Error ? error.message : 'Failed to update control';
      });
      throw error;
    }
  },

  // Direct state setters
  setFrameworks: (frameworks: Framework[]) => {
    set((state) => {
      state.compliance.frameworks = frameworks;
      state.compliance.lastUpdated = new Date();
    });
  },

  setControls: (controls: Control[]) => {
    set((state) => {
      state.compliance.controls = controls;
      state.compliance.lastUpdated = new Date();
    });
  },

  setBaselines: (baselines: Baseline[]) => {
    set((state) => {
      state.compliance.baselines = baselines;
      state.compliance.lastUpdated = new Date();
    });
  },

  setMetrics: (metrics: Metric[]) => {
    set((state) => {
      state.compliance.metrics = metrics;
      state.compliance.lastUpdated = new Date();
    });
  },

  setComplianceLoading: (loading: boolean) => {
    set((state) => {
      state.compliance.isLoading = loading;
    });
  },

  setComplianceError: (error: string | null) => {
    set((state) => {
      state.compliance.error = error;
    });
  },

  updateLastUpdated: () => {
    set((state) => {
      state.compliance.lastUpdated = new Date();
    });
  },
});

// Compliance utility functions
export const complianceUtils = {
  // Calculate compliance percentage for a framework
  calculateFrameworkCompliance: (controls: Control[]): number => {
    if (controls.length === 0) return 0;
    
    const implemented = controls.filter(c => c.implementationStatus === 'implemented').length;
    const partial = controls.filter(c => c.implementationStatus === 'partial').length;
    
    return Math.round(((implemented + (partial * 0.5)) / controls.length) * 100);
  },

  // Get controls by priority
  getControlsByPriority: (controls: Control[], priority: string) => {
    return controls.filter(c => c.priority === priority);
  },

  // Get overdue controls (if lastAssessment is older than required frequency)
  getOverdueControls: (controls: Control[]): Control[] => {
    const now = new Date();
    return controls.filter(c => {
      if (!c.lastAssessment) return true;
      
      const lastAssessment = new Date(c.lastAssessment);
      const daysSinceAssessment = Math.floor((now.getTime() - lastAssessment.getTime()) / (1000 * 60 * 60 * 24));
      
      // Assume controls should be assessed every 90 days
      return daysSinceAssessment > 90;
    });
  },

  // Group controls by status
  groupControlsByStatus: (controls: Control[]) => {
    return controls.reduce((acc, control) => {
      const status = control.implementationStatus;
      if (!acc[status]) {
        acc[status] = [];
      }
      acc[status].push(control);
      return acc;
    }, {} as Record<string, Control[]>);
  },

  // Find controls that need attention (high priority + not implemented)
  getControlsNeedingAttention: (controls: Control[]): Control[] => {
    return controls.filter(c => 
      c.priority === 'high' && 
      (c.implementationStatus === 'not_implemented' || c.implementationStatus === 'partial')
    );
  },

  // Calculate trend data (requires historical data)
  calculateTrend: (currentMetrics: Metric[], previousMetrics: Metric[]) => {
    // This would calculate trends based on historical data
    // Implementation depends on the structure of metrics data
    return {
      compliance: 0, // percentage change
      implemented: 0, // count change
      partial: 0, // count change
      notImplemented: 0, // count change
    };
  },
};
