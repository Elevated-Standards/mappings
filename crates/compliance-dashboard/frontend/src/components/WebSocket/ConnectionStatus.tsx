// Modified: 2025-09-21

import React from 'react';
import { useComplianceWebSocket, webSocketUtils } from '../../services/websocket';
import { realtimeUtils } from '../../store/slices/realtimeSlice';
import './ConnectionStatus.css';

interface ConnectionStatusProps {
  showDetails?: boolean;
  className?: string;
}

export const ConnectionStatus: React.FC<ConnectionStatusProps> = ({
  showDetails = false,
  className = '',
}) => {
  const {
    isConnected,
    connectionStatus,
    lastUpdate,
    reconnectAttempts,
    getConnectionHealth,
    requestDataRefresh,
  } = useComplianceWebSocket();

  const health = getConnectionHealth();
  const formattedStatus = webSocketUtils.formatConnectionStatus(connectionStatus);
  const lastUpdateText = realtimeUtils.formatLastUpdate(lastUpdate);

  const handleRefresh = () => {
    requestDataRefresh();
  };

  const getStatusColor = () => {
    if (health.isHealthy) return 'success';
    if (connectionStatus === 'connecting') return 'warning';
    return 'error';
  };

  return (
    <div className={`connection-status connection-status--${getStatusColor()} ${className}`}>
      <div className="connection-status__main">
        <span className="connection-status__indicator">
          {realtimeUtils.getConnectionStatusIcon(connectionStatus)}
        </span>
        <span className="connection-status__text">
          {formattedStatus}
        </span>
        {isConnected && (
          <button
            className="connection-status__refresh"
            onClick={handleRefresh}
            title="Refresh data"
            type="button"
          >
            🔄
          </button>
        )}
      </div>

      {showDetails && (
        <div className="connection-status__details">
          <div className="connection-status__detail">
            <span className="connection-status__label">Last Update:</span>
            <span className="connection-status__value">{lastUpdateText}</span>
          </div>
          
          {reconnectAttempts > 0 && (
            <div className="connection-status__detail">
              <span className="connection-status__label">Reconnect Attempts:</span>
              <span className="connection-status__value">{reconnectAttempts}</span>
            </div>
          )}
          
          <div className="connection-status__detail">
            <span className="connection-status__label">Health:</span>
            <span className={`connection-status__value connection-status__value--${health.isHealthy ? 'healthy' : 'unhealthy'}`}>
              {health.isHealthy ? 'Healthy' : 'Unhealthy'}
            </span>
          </div>
          
          {health.timeSinceLastUpdate !== null && (
            <div className="connection-status__detail">
              <span className="connection-status__label">Time Since Update:</span>
              <span className="connection-status__value">
                {Math.round(health.timeSinceLastUpdate / 1000)}s
              </span>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

interface RealtimeIndicatorProps {
  size?: 'sm' | 'md' | 'lg';
  showText?: boolean;
  className?: string;
}

export const RealtimeIndicator: React.FC<RealtimeIndicatorProps> = ({
  size = 'md',
  showText = true,
  className = '',
}) => {
  const { isConnected, connectionStatus } = useComplianceWebSocket();

  const getIndicatorClass = () => {
    const baseClass = 'realtime-indicator';
    const sizeClass = `${baseClass}--${size}`;
    const statusClass = `${baseClass}--${connectionStatus}`;
    return `${baseClass} ${sizeClass} ${statusClass} ${className}`;
  };

  return (
    <div className={getIndicatorClass()}>
      <div className="realtime-indicator__dot" />
      {showText && (
        <span className="realtime-indicator__text">
          {isConnected ? 'Live' : 'Offline'}
        </span>
      )}
    </div>
  );
};

interface WebSocketDebugProps {
  className?: string;
}

export const WebSocketDebug: React.FC<WebSocketDebugProps> = ({
  className = '',
}) => {
  const {
    connectionStatus,
    lastUpdate,
    reconnectAttempts,
    getConnectionHealth,
    connect,
    disconnect,
    sendMessage,
  } = useComplianceWebSocket();

  const health = getConnectionHealth();

  const handleConnect = () => {
    connect();
  };

  const handleDisconnect = () => {
    disconnect();
  };

  const handleSendHeartbeat = () => {
    sendMessage('heartbeat', { debug: true });
  };

  return (
    <div className={`websocket-debug ${className}`}>
      <h3>WebSocket Debug Panel</h3>
      
      <div className="websocket-debug__section">
        <h4>Connection Status</h4>
        <div className="websocket-debug__grid">
          <div>Status: {connectionStatus}</div>
          <div>Connected: {health.isConnected ? 'Yes' : 'No'}</div>
          <div>Healthy: {health.isHealthy ? 'Yes' : 'No'}</div>
          <div>Reconnect Attempts: {reconnectAttempts}</div>
          <div>Last Update: {lastUpdate ? lastUpdate.toLocaleString() : 'Never'}</div>
          <div>Time Since Update: {
            health.timeSinceLastUpdate !== null 
              ? `${Math.round(health.timeSinceLastUpdate / 1000)}s`
              : 'N/A'
          }</div>
        </div>
      </div>

      <div className="websocket-debug__section">
        <h4>Actions</h4>
        <div className="websocket-debug__actions">
          <button onClick={handleConnect} disabled={health.isConnected}>
            Connect
          </button>
          <button onClick={handleDisconnect} disabled={!health.isConnected}>
            Disconnect
          </button>
          <button onClick={handleSendHeartbeat} disabled={!health.isConnected}>
            Send Heartbeat
          </button>
        </div>
      </div>
    </div>
  );
};
