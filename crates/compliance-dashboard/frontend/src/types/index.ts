// Modified: 2025-09-21

export interface User {
  id: string;
  email: string;
  name: string;
  role: UserRole;
  permissions: Permission[];
}

export type UserRole = 'admin' | 'auditor' | 'viewer';

export interface Permission {
  id: string;
  name: string;
  resource: string;
  action: string;
}

export interface Framework {
  id: string;
  name: string;
  version: string;
  description: string;
  controls: Control[];
}

export interface Control {
  id: string;
  frameworkId: string;
  identifier: string;
  title: string;
  description: string;
  implementationStatus: ImplementationStatus;
  priority: Priority;
  category: string;
  requirements?: string[];
  evidence?: string[];
  notes?: string;
  assignedTo?: string;
  dueDate?: Date;
  lastAssessment?: Date;
  lastUpdated: Date;
}

export type ImplementationStatus =
  | 'implemented'
  | 'not_implemented'
  | 'partial'
  | 'not_applicable';
export type Priority = 'high' | 'medium' | 'low';

export interface Baseline {
  id: string;
  name: string;
  description: string;
  frameworkId: string;
  controls: string[];
  completionPercentage: number;
}

export interface Metric {
  id: string;
  name: string;
  value: number;
  unit: string;
  timestamp: Date;
  category: MetricCategory;
}

export type MetricCategory =
  | 'compliance'
  | 'security'
  | 'performance'
  | 'coverage';

export interface FilterState {
  framework?: string;
  status?: ImplementationStatus[];
  priority?: Priority[];
  category?: string;
  search?: string;
}

export interface ErrorState {
  id: string;
  message: string;
  type: 'error' | 'warning' | 'info';
  code?: string;
  timestamp: Date;
  timeout?: number;
}

export interface AppState {
  auth: {
    user: User | null;
    permissions: Permission[];
    isAuthenticated: boolean;
  };
  compliance: {
    frameworks: Framework[];
    controls: Control[];
    baselines: Baseline[];
    metrics: Metric[];
  };
  ui: {
    selectedFramework: string;
    filters: FilterState;
    loading: boolean;
    errors: ErrorState[];
  };
  realtime: {
    connectionStatus: 'connected' | 'disconnected' | 'connecting';
    lastUpdate: Date;
  };
}

export interface WebSocketEvents {
  'control-updated': ControlUpdateEvent;
  'baseline-changed': BaselineChangeEvent;
  'metrics-updated': MetricsUpdateEvent;
  'user-activity': UserActivityEvent;
  'system-alert': SystemAlertEvent;
}

export interface ControlUpdateEvent {
  controlId: string;
  status: ImplementationStatus;
  timestamp: Date;
  userId: string;
}

export interface BaselineChangeEvent {
  baselineId: string;
  completionPercentage: number;
  timestamp: Date;
}

export interface MetricsUpdateEvent {
  metrics: Metric[];
  timestamp: Date;
}

export interface UserActivityEvent {
  userId: string;
  action: string;
  resource: string;
  timestamp: Date;
}

export interface SystemAlertEvent {
  id: string;
  type: 'info' | 'warning' | 'error';
  message: string;
  timestamp: Date;
}
