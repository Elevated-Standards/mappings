// API service for connecting to the compliance dashboard backend
import type { Control, Framework, ImplementationStatus } from '../types';

const API_BASE_URL = 'http://localhost:8080/api';

export interface DashboardOverview {
  total_controls: number;
  implemented_controls: number;
  in_progress_controls: number;
  not_implemented_controls: number;
  implementation_percentage: number;
  frameworks: Framework[];
  recent_updates: Control[];
  key_metrics: any[];
  last_updated: string;
}

export interface DashboardData {
  overview: DashboardOverview;
  widgets: any[];
  connection_stats: {
    total_connections: number;
    active_connections: number;
    inactive_connections: number;
  };
}

export interface ApiResponse<T> {
  data?: T;
  error?: string;
  status?: number;
}

class ApiService {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    try {
      const url = `${this.baseUrl}${endpoint}`;
      const response = await fetch(url, {
        headers: {
          'Content-Type': 'application/json',
          ...options.headers,
        },
        ...options,
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        return {
          error: errorData.error || `HTTP ${response.status}: ${response.statusText}`,
          status: response.status,
        };
      }

      const data = await response.json();
      return { data, status: response.status };
    } catch (error) {
      return {
        error: error instanceof Error ? error.message : 'Network error',
        status: 0,
      };
    }
  }

  // Dashboard endpoints
  async getDashboard(): Promise<ApiResponse<DashboardData>> {
    return this.request<DashboardData>('/dashboard');
  }

  async getMetrics(): Promise<ApiResponse<any>> {
    return this.request<any>('/dashboard/metrics');
  }

  async getWidgets(): Promise<ApiResponse<{ widgets: any[] }>> {
    return this.request<{ widgets: any[] }>('/dashboard/widgets');
  }

  // Control endpoints
  async getControls(params?: {
    framework?: string;
    status?: string;
  }): Promise<ApiResponse<{ controls: Control[] }>> {
    const searchParams = new URLSearchParams();
    if (params?.framework) searchParams.set('framework', params.framework);
    if (params?.status) searchParams.set('status', params.status);
    
    const query = searchParams.toString();
    const endpoint = `/controls${query ? `?${query}` : ''}`;
    
    return this.request<{ controls: Control[] }>(endpoint);
  }

  async getControl(controlId: string): Promise<ApiResponse<{ control: Control }>> {
    return this.request<{ control: Control }>(`/controls/${controlId}`);
  }

  async updateControlStatus(
    controlId: string,
    status: ImplementationStatus
  ): Promise<ApiResponse<{ success: boolean; message: string }>> {
    return this.request<{ success: boolean; message: string }>(
      `/controls/${controlId}/status`,
      {
        method: 'PUT',
        body: JSON.stringify({ status }),
      }
    );
  }

  // Framework endpoints
  async getFrameworks(): Promise<ApiResponse<{ frameworks: Framework[] }>> {
    return this.request<{ frameworks: Framework[] }>('/frameworks');
  }

  async getFrameworkControls(frameworkId: string): Promise<ApiResponse<{ controls: Control[] }>> {
    return this.request<{ controls: Control[] }>(`/frameworks/${frameworkId}/controls`);
  }

  // Real-time endpoints
  async getRealtimeStats(): Promise<ApiResponse<any>> {
    return this.request<any>('/realtime/stats');
  }

  // Health check
  async healthCheck(): Promise<ApiResponse<{ status: string; timestamp: string; service: string }>> {
    return this.request<{ status: string; timestamp: string; service: string }>('/health', {
      method: 'GET',
    });
  }

  // WebSocket connection for real-time updates
  connectWebSocket(onMessage?: (event: MessageEvent) => void): WebSocket | null {
    try {
      const wsUrl = this.baseUrl.replace('http://', 'ws://').replace('https://', 'wss://');
      const ws = new WebSocket(`${wsUrl}/realtime/ws`);

      ws.onopen = () => {
        console.log('WebSocket connected');
      };

      ws.onmessage = (event) => {
        console.log('WebSocket message:', event.data);
        if (onMessage) {
          onMessage(event);
        }
      };

      ws.onclose = () => {
        console.log('WebSocket disconnected');
      };

      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };

      return ws;
    } catch (error) {
      console.error('Failed to create WebSocket connection:', error);
      return null;
    }
  }
}

// Create and export a singleton instance
export const apiService = new ApiService();

// Export the class for testing or custom instances
export { ApiService };

// Utility functions for common operations
export const dashboardUtils = {
  // Calculate implementation percentage
  calculateImplementationPercentage(overview: DashboardOverview): number {
    if (overview.total_controls === 0) return 0;
    return (overview.implemented_controls / overview.total_controls) * 100;
  },

  // Get status color for UI
  getStatusColor(status: ImplementationStatus): string {
    switch (status) {
      case 'implemented':
        return '#10b981'; // green
      case 'in-progress':
        return '#f59e0b'; // yellow
      case 'not-implemented':
        return '#ef4444'; // red
      case 'not-applicable':
        return '#6b7280'; // gray
      default:
        return '#6b7280';
    }
  },

  // Format date for display
  formatDate(dateString: string): string {
    try {
      const date = new Date(dateString);
      return date.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return 'Invalid date';
    }
  },

  // Get priority color
  getPriorityColor(priority: string): string {
    switch (priority.toLowerCase()) {
      case 'critical':
        return '#dc2626'; // red
      case 'high':
        return '#ea580c'; // orange
      case 'medium':
        return '#d97706'; // amber
      case 'low':
        return '#65a30d'; // lime
      default:
        return '#6b7280'; // gray
    }
  },
};

// Mock data for development when backend is not available
export const mockData = {
  dashboard: {
    overview: {
      total_controls: 100,
      implemented_controls: 65,
      in_progress_controls: 20,
      not_implemented_controls: 15,
      implementation_percentage: 65.0,
      frameworks: [
        {
          id: 'nist-800-53',
          name: 'NIST 800-53',
          version: 'Rev 5',
          description: 'Security and Privacy Controls for Federal Information Systems',
          control_count: 1000,
          implemented_count: 650,
          in_progress_count: 200,
          not_implemented_count: 150,
          last_updated: new Date().toISOString(),
        },
      ],
      recent_updates: [
        {
          id: 'ac-1',
          framework_id: 'nist-800-53',
          identifier: 'AC-1',
          title: 'Access Control Policy and Procedures',
          description: 'Develop, document, and disseminate access control policy and procedures.',
          implementation_status: 'implemented' as ImplementationStatus,
          priority: 'high',
          category: 'Access Control',
          last_updated: new Date(),
        },
      ],
      key_metrics: [],
      last_updated: new Date().toISOString(),
    },
    widgets: [],
    connection_stats: {
      total_connections: 0,
      active_connections: 0,
      inactive_connections: 0,
    },
  },
};
