// Modified: 2025-09-21

import type {
  User,
  Permission,
  Framework,
  Control,
  Baseline,
  Metric,
  FilterState,
  ErrorState,
  ImplementationStatus,
} from '../types';

export interface AuthState {
  user: User | null;
  permissions: Permission[];
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
}

export interface ComplianceState {
  frameworks: Framework[];
  controls: Control[];
  baselines: Baseline[];
  metrics: Metric[];
  isLoading: boolean;
  error: string | null;
  lastUpdated: Date | null;
}

export interface UIState {
  selectedFramework: string | null;
  filters: FilterState;
  loading: boolean;
  errors: ErrorState[];
  sidebarOpen: boolean;
  theme: 'light' | 'dark' | 'system';
}

export interface RealtimeState {
  connectionStatus: 'connected' | 'disconnected' | 'connecting';
  lastUpdate: Date | null;
  reconnectAttempts: number;
  maxReconnectAttempts: number;
}

export interface AppState {
  auth: AuthState;
  compliance: ComplianceState;
  ui: UIState;
  realtime: RealtimeState;
}

// Action types for better type safety
export interface AuthActions {
  login: (email: string, password: string) => Promise<void>;
  logout: () => void;
  setUser: (user: User | null) => void;
  setPermissions: (permissions: Permission[]) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}

export interface ComplianceActions {
  loadFrameworks: () => Promise<void>;
  loadControls: (frameworkId?: string) => Promise<void>;
  loadBaselines: () => Promise<void>;
  loadMetrics: () => Promise<void>;
  updateControl: (controlId: string, updates: Partial<Control>) => Promise<void>;
  setFrameworks: (frameworks: Framework[]) => void;
  setControls: (controls: Control[]) => void;
  setBaselines: (baselines: Baseline[]) => void;
  setMetrics: (metrics: Metric[]) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  updateLastUpdated: () => void;
}

export interface UIActions {
  setSelectedFramework: (frameworkId: string | null) => void;
  updateFilters: (filters: Partial<FilterState>) => void;
  clearFilters: () => void;
  setLoading: (loading: boolean) => void;
  addError: (error: ErrorState) => void;
  removeError: (errorId: string) => void;
  clearErrors: () => void;
  toggleSidebar: () => void;
  setSidebarOpen: (open: boolean) => void;
  setTheme: (theme: 'light' | 'dark' | 'system') => void;
}

export interface RealtimeActions {
  setConnectionStatus: (status: 'connected' | 'disconnected' | 'connecting') => void;
  updateLastUpdate: () => void;
  incrementReconnectAttempts: () => void;
  resetReconnectAttempts: () => void;
  setMaxReconnectAttempts: (max: number) => void;
}

// Combined store interface
export interface Store extends AppState {
  // Auth actions
  login: AuthActions['login'];
  logout: AuthActions['logout'];
  setUser: AuthActions['setUser'];
  setPermissions: AuthActions['setPermissions'];
  setAuthLoading: AuthActions['setLoading'];
  setAuthError: AuthActions['setError'];

  // Compliance actions
  loadFrameworks: ComplianceActions['loadFrameworks'];
  loadControls: ComplianceActions['loadControls'];
  loadBaselines: ComplianceActions['loadBaselines'];
  loadMetrics: ComplianceActions['loadMetrics'];
  updateControl: ComplianceActions['updateControl'];
  setFrameworks: ComplianceActions['setFrameworks'];
  setControls: ComplianceActions['setControls'];
  setBaselines: ComplianceActions['setBaselines'];
  setMetrics: ComplianceActions['setMetrics'];
  setComplianceLoading: ComplianceActions['setLoading'];
  setComplianceError: ComplianceActions['setError'];
  updateLastUpdated: ComplianceActions['updateLastUpdated'];

  // UI actions
  setSelectedFramework: UIActions['setSelectedFramework'];
  updateFilters: UIActions['updateFilters'];
  clearFilters: UIActions['clearFilters'];
  setUILoading: UIActions['setLoading'];
  addError: UIActions['addError'];
  removeError: UIActions['removeError'];
  clearErrors: UIActions['clearErrors'];
  toggleSidebar: UIActions['toggleSidebar'];
  setSidebarOpen: UIActions['setSidebarOpen'];
  setTheme: UIActions['setTheme'];

  // Realtime actions
  setConnectionStatus: RealtimeActions['setConnectionStatus'];
  updateLastUpdate: RealtimeActions['updateLastUpdate'];
  incrementReconnectAttempts: RealtimeActions['incrementReconnectAttempts'];
  resetReconnectAttempts: RealtimeActions['resetReconnectAttempts'];
  setMaxReconnectAttempts: RealtimeActions['setMaxReconnectAttempts'];
}

// Selector types for better type safety
export interface Selectors {
  // Auth selectors
  isAuthenticated: () => boolean;
  currentUser: () => User | null;
  userPermissions: () => Permission[];
  hasPermission: (permission: string) => boolean;

  // Compliance selectors
  getFrameworkById: (id: string) => Framework | undefined;
  getControlById: (id: string) => Control | undefined;
  getControlsByFramework: (frameworkId: string) => Control[];
  getControlsByStatus: (status: ImplementationStatus) => Control[];
  getFilteredControls: () => Control[];
  getComplianceMetrics: () => {
    total: number;
    implemented: number;
    partial: number;
    notImplemented: number;
    notApplicable: number;
    percentageComplete: number;
  };

  // UI selectors
  getActiveFilters: () => FilterState;
  hasActiveFilters: () => boolean;
  getErrorsCount: () => number;
  getCurrentTheme: () => 'light' | 'dark' | 'system';

  // Realtime selectors
  isConnected: () => boolean;
  getConnectionStatus: () => 'connected' | 'disconnected' | 'connecting';
  shouldReconnect: () => boolean;
}
