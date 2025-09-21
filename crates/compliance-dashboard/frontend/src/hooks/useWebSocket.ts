// Modified: 2025-09-21

import { useEffect, useRef } from 'react';
import { useStore, useIsAuthenticated, useRealtimeActions } from '../store';
import { WebSocketManager, realtimeUtils } from '../store/slices/realtimeSlice';

interface UseWebSocketOptions {
  url?: string;
  autoConnect?: boolean;
  reconnectOnAuth?: boolean;
}

export const useWebSocket = (options: UseWebSocketOptions = {}) => {
  const {
    url = realtimeUtils.createWebSocketUrl(),
    autoConnect = true,
    reconnectOnAuth = true,
  } = options;

  const wsManagerRef = useRef<WebSocketManager | null>(null);
  const isAuthenticated = useIsAuthenticated();
  const store = useStore();
  const realtimeActions = useRealtimeActions();

  // Initialize WebSocket manager
  useEffect(() => {
    if (!wsManagerRef.current) {
      wsManagerRef.current = new WebSocketManager(url, {
        ...realtimeActions,
        getState: () => store,
        setControls: store.setControls,
        setFrameworks: store.setFrameworks,
        setMetrics: store.setMetrics,
      });
    }

    return () => {
      if (wsManagerRef.current) {
        wsManagerRef.current.disconnect();
        wsManagerRef.current = null;
      }
    };
  }, [url]);

  // Handle authentication changes
  useEffect(() => {
    if (!wsManagerRef.current) return;

    if (isAuthenticated && autoConnect) {
      wsManagerRef.current.connect();
    } else {
      wsManagerRef.current.disconnect();
    }
  }, [isAuthenticated, autoConnect]);

  // Reconnect when authentication status changes
  useEffect(() => {
    if (!wsManagerRef.current || !reconnectOnAuth) return;

    if (isAuthenticated) {
      // Small delay to ensure auth token is available
      const timer = setTimeout(() => {
        wsManagerRef.current?.connect();
      }, 100);

      return () => clearTimeout(timer);
    }
  }, [isAuthenticated, reconnectOnAuth]);

  const connect = () => {
    wsManagerRef.current?.connect();
  };

  const disconnect = () => {
    wsManagerRef.current?.disconnect();
  };

  const send = (message: any) => {
    wsManagerRef.current?.send(message);
  };

  const getConnectionState = () => {
    return wsManagerRef.current?.getConnectionState() || 'disconnected';
  };

  return {
    connect,
    disconnect,
    send,
    getConnectionState,
    isConnected: store.realtime.connectionStatus === 'connected',
    connectionStatus: store.realtime.connectionStatus,
    lastUpdate: store.realtime.lastUpdate,
    reconnectAttempts: store.realtime.reconnectAttempts,
  };
};
