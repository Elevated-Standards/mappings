// Modified: 2025-09-21

import type { StateCreator } from 'zustand';
import type { Store, RealtimeState } from '../types';

// Initial realtime state
const initialRealtimeState: RealtimeState = {
  connectionStatus: 'disconnected',
  lastUpdate: null,
  reconnectAttempts: 0,
  maxReconnectAttempts: 5,
};

export const createRealtimeSlice: StateCreator<
  Store,
  [['zustand/immer', never]],
  [],
  { realtime: RealtimeState } & {
    setConnectionStatus: (status: 'connected' | 'disconnected' | 'connecting') => void;
    updateLastUpdate: () => void;
    incrementReconnectAttempts: () => void;
    resetReconnectAttempts: () => void;
    setMaxReconnectAttempts: (max: number) => void;
  }
> = (set, get) => ({
  // Realtime state
  realtime: initialRealtimeState,

  // Realtime actions
  setConnectionStatus: (status: 'connected' | 'disconnected' | 'connecting') => {
    set((state) => {
      state.realtime.connectionStatus = status;
      
      // Reset reconnect attempts on successful connection
      if (status === 'connected') {
        state.realtime.reconnectAttempts = 0;
      }
    });
  },

  updateLastUpdate: () => {
    set((state) => {
      state.realtime.lastUpdate = new Date();
    });
  },

  incrementReconnectAttempts: () => {
    set((state) => {
      state.realtime.reconnectAttempts += 1;
    });
  },

  resetReconnectAttempts: () => {
    set((state) => {
      state.realtime.reconnectAttempts = 0;
    });
  },

  setMaxReconnectAttempts: (max: number) => {
    set((state) => {
      state.realtime.maxReconnectAttempts = max;
    });
  },
});

// WebSocket manager class
export class WebSocketManager {
  private ws: WebSocket | null = null;
  private store: any;
  private reconnectTimeout: NodeJS.Timeout | null = null;
  private heartbeatInterval: NodeJS.Timeout | null = null;
  private url: string;

  constructor(url: string, store: any) {
    this.url = url;
    this.store = store;
  }

  connect(): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      return;
    }

    this.store.setConnectionStatus('connecting');

    try {
      this.ws = new WebSocket(this.url);
      this.setupEventListeners();
    } catch (error) {
      console.error('WebSocket connection failed:', error);
      this.handleConnectionError();
    }
  }

  disconnect(): void {
    this.cleanup();
    this.store.setConnectionStatus('disconnected');
  }

  private setupEventListeners(): void {
    if (!this.ws) return;

    this.ws.onopen = () => {
      console.log('WebSocket connected');
      this.store.setConnectionStatus('connected');
      this.store.resetReconnectAttempts();
      this.startHeartbeat();
    };

    this.ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        this.handleMessage(data);
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };

    this.ws.onclose = (event) => {
      console.log('WebSocket disconnected:', event.code, event.reason);
      this.store.setConnectionStatus('disconnected');
      this.cleanup();
      
      // Attempt reconnection if not a clean close
      if (event.code !== 1000) {
        this.attemptReconnect();
      }
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      this.handleConnectionError();
    };
  }

  private handleMessage(data: any): void {
    this.store.updateLastUpdate();

    switch (data.type) {
      case 'control_updated':
        this.handleControlUpdate(data.payload);
        break;
      case 'framework_updated':
        this.handleFrameworkUpdate(data.payload);
        break;
      case 'metrics_updated':
        this.handleMetricsUpdate(data.payload);
        break;
      case 'heartbeat':
        // Heartbeat response - no action needed
        break;
      default:
        console.warn('Unknown message type:', data.type);
    }
  }

  private handleControlUpdate(control: any): void {
    const state = this.store.getState();
    const existingControls = state.compliance.controls;
    const updatedControls = existingControls.map((c: any) => 
      c.id === control.id ? { ...c, ...control } : c
    );
    
    this.store.setControls(updatedControls);
  }

  private handleFrameworkUpdate(framework: any): void {
    const state = this.store.getState();
    const existingFrameworks = state.compliance.frameworks;
    const updatedFrameworks = existingFrameworks.map((f: any) => 
      f.id === framework.id ? { ...f, ...framework } : f
    );
    
    this.store.setFrameworks(updatedFrameworks);
  }

  private handleMetricsUpdate(metrics: any): void {
    this.store.setMetrics(metrics);
  }

  private startHeartbeat(): void {
    this.heartbeatInterval = setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.ws.send(JSON.stringify({ type: 'heartbeat' }));
      }
    }, 30000); // Send heartbeat every 30 seconds
  }

  private handleConnectionError(): void {
    this.store.setConnectionStatus('disconnected');
    this.cleanup();
    this.attemptReconnect();
  }

  private attemptReconnect(): void {
    const state = this.store.getState();
    const { reconnectAttempts, maxReconnectAttempts } = state.realtime;

    if (reconnectAttempts >= maxReconnectAttempts) {
      console.log('Max reconnection attempts reached');
      return;
    }

    this.store.incrementReconnectAttempts();
    
    // Exponential backoff: 1s, 2s, 4s, 8s, 16s
    const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), 16000);
    
    console.log(`Attempting to reconnect in ${delay}ms (attempt ${reconnectAttempts + 1}/${maxReconnectAttempts})`);
    
    this.reconnectTimeout = setTimeout(() => {
      this.connect();
    }, delay);
  }

  private cleanup(): void {
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }

    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }

    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  // Send message to server
  send(message: any): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    } else {
      console.warn('WebSocket not connected, message not sent:', message);
    }
  }

  // Get connection state
  getConnectionState(): string {
    if (!this.ws) return 'disconnected';
    
    switch (this.ws.readyState) {
      case WebSocket.CONNECTING:
        return 'connecting';
      case WebSocket.OPEN:
        return 'connected';
      case WebSocket.CLOSING:
      case WebSocket.CLOSED:
      default:
        return 'disconnected';
    }
  }
}

// Realtime utility functions
export const realtimeUtils = {
  // Create WebSocket URL based on current location
  createWebSocketUrl: (path: string = '/ws'): string => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.host;
    return `${protocol}//${host}${path}`;
  },

  // Format last update time
  formatLastUpdate: (lastUpdate: Date | null): string => {
    if (!lastUpdate) return 'Never';
    
    const now = new Date();
    const diff = now.getTime() - lastUpdate.getTime();
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);

    if (seconds < 60) {
      return `${seconds}s ago`;
    } else if (minutes < 60) {
      return `${minutes}m ago`;
    } else if (hours < 24) {
      return `${hours}h ago`;
    } else {
      return lastUpdate.toLocaleDateString();
    }
  },

  // Get connection status color
  getConnectionStatusColor: (status: string): string => {
    switch (status) {
      case 'connected':
        return 'green';
      case 'connecting':
        return 'yellow';
      case 'disconnected':
      default:
        return 'red';
    }
  },

  // Get connection status icon
  getConnectionStatusIcon: (status: string): string => {
    switch (status) {
      case 'connected':
        return '🟢';
      case 'connecting':
        return '🟡';
      case 'disconnected':
      default:
        return '🔴';
    }
  },
};
