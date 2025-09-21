//! Real-time updates and WebSocket support
//!
//! This module provides real-time communication capabilities for the dashboard,
//! including WebSocket connections, event broadcasting, and live data updates.

use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Real-time service for managing WebSocket connections and events
#[derive(Debug, Clone)]
pub struct RealtimeService {
    /// Active connections
    connections: Arc<RwLock<HashMap<String, Connection>>>,
    /// Event broadcaster
    event_sender: broadcast::Sender<RealtimeEvent>,
    /// Subscription manager
    subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>, // connection_id -> event_types
}

/// WebSocket connection information
#[derive(Debug, Clone)]
pub struct Connection {
    pub id: String,
    pub user_id: Option<String>,
    pub connected_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub subscriptions: Vec<String>,
}

/// Real-time event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RealtimeEvent {
    /// Control status updated
    ControlStatusUpdated {
        control_id: String,
        old_status: String,
        new_status: String,
        updated_by: Option<String>,
        timestamp: DateTime<Utc>,
    },
    /// Metrics updated
    MetricsUpdated {
        metrics: MetricsUpdate,
        timestamp: DateTime<Utc>,
    },
    /// New alert generated
    AlertGenerated {
        alert: AlertEvent,
        timestamp: DateTime<Utc>,
    },
    /// Framework data updated
    FrameworkUpdated {
        framework_id: String,
        changes: Vec<String>,
        timestamp: DateTime<Utc>,
    },
    /// System status change
    SystemStatusChanged {
        status: SystemStatus,
        message: Option<String>,
        timestamp: DateTime<Utc>,
    },
    /// User activity
    UserActivity {
        user_id: String,
        activity: String,
        details: HashMap<String, String>,
        timestamp: DateTime<Utc>,
    },
}

/// Metrics update event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsUpdate {
    pub implementation_percentage: f64,
    pub total_controls: u32,
    pub implemented_controls: u32,
    pub in_progress_controls: u32,
    pub not_implemented_controls: u32,
    pub changed_metrics: Vec<String>,
}

/// Alert event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub id: String,
    pub title: String,
    pub message: String,
    pub severity: AlertSeverity,
    pub category: AlertCategory,
    pub related_control: Option<String>,
    pub auto_generated: bool,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Alert categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AlertCategory {
    ComplianceGap,
    OverdueControl,
    SystemIssue,
    DataQuality,
    SecurityConcern,
}

/// System status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SystemStatus {
    Healthy,
    Warning,
    Error,
    Maintenance,
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// Subscribe to event types
    Subscribe {
        event_types: Vec<String>,
    },
    /// Unsubscribe from event types
    Unsubscribe {
        event_types: Vec<String>,
    },
    /// Ping for connection health
    Ping {
        timestamp: DateTime<Utc>,
    },
    /// Pong response
    Pong {
        timestamp: DateTime<Utc>,
    },
    /// Event notification
    Event {
        event: RealtimeEvent,
    },
    /// Error message
    Error {
        message: String,
        code: Option<String>,
    },
}

impl RealtimeService {
    /// Create a new real-time service
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new WebSocket connection
    pub async fn register_connection(&self, user_id: Option<String>) -> Result<String> {
        let connection_id = Uuid::new_v4().to_string();
        let connection = Connection {
            id: connection_id.clone(),
            user_id,
            connected_at: Utc::now(),
            last_activity: Utc::now(),
            subscriptions: Vec::new(),
        };

        let mut connections = self.connections.write().await;
        connections.insert(connection_id.clone(), connection);

        Ok(connection_id)
    }

    /// Unregister a WebSocket connection
    pub async fn unregister_connection(&self, connection_id: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        connections.remove(connection_id);

        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(connection_id);

        Ok(())
    }

    /// Subscribe connection to event types
    pub async fn subscribe(&self, connection_id: &str, event_types: Vec<String>) -> Result<()> {
        // Update connection subscriptions
        {
            let mut connections = self.connections.write().await;
            if let Some(connection) = connections.get_mut(connection_id) {
                for event_type in &event_types {
                    if !connection.subscriptions.contains(event_type) {
                        connection.subscriptions.push(event_type.clone());
                    }
                }
                connection.last_activity = Utc::now();
            } else {
                return Err(Error::not_found(format!("Connection not found: {}", connection_id)));
            }
        }

        // Update subscription index
        {
            let mut subscriptions = self.subscriptions.write().await;
            let connection_subscriptions = subscriptions.entry(connection_id.to_string())
                .or_insert_with(Vec::new);
            
            for event_type in event_types {
                if !connection_subscriptions.contains(&event_type) {
                    connection_subscriptions.push(event_type);
                }
            }
        }

        Ok(())
    }

    /// Unsubscribe connection from event types
    pub async fn unsubscribe(&self, connection_id: &str, event_types: Vec<String>) -> Result<()> {
        // Update connection subscriptions
        {
            let mut connections = self.connections.write().await;
            if let Some(connection) = connections.get_mut(connection_id) {
                connection.subscriptions.retain(|sub| !event_types.contains(sub));
                connection.last_activity = Utc::now();
            }
        }

        // Update subscription index
        {
            let mut subscriptions = self.subscriptions.write().await;
            if let Some(connection_subscriptions) = subscriptions.get_mut(connection_id) {
                connection_subscriptions.retain(|sub| !event_types.contains(sub));
            }
        }

        Ok(())
    }

    /// Broadcast an event to all subscribed connections
    pub async fn broadcast_event(&self, event: RealtimeEvent) -> Result<()> {
        // Send to broadcast channel
        if let Err(_) = self.event_sender.send(event.clone()) {
            // Channel might be full or have no receivers, which is okay
        }

        Ok(())
    }

    /// Get event receiver for a connection
    pub fn get_event_receiver(&self) -> broadcast::Receiver<RealtimeEvent> {
        self.event_sender.subscribe()
    }

    /// Update connection activity
    pub async fn update_activity(&self, connection_id: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(connection_id) {
            connection.last_activity = Utc::now();
            Ok(())
        } else {
            Err(Error::not_found(format!("Connection not found: {}", connection_id)))
        }
    }

    /// Get connection statistics
    pub async fn get_connection_stats(&self) -> ConnectionStats {
        let connections = self.connections.read().await;
        let total_connections = connections.len();
        let active_connections = connections.values()
            .filter(|conn| {
                let inactive_threshold = Utc::now() - chrono::Duration::minutes(5);
                conn.last_activity > inactive_threshold
            })
            .count();

        ConnectionStats {
            total_connections,
            active_connections,
            inactive_connections: total_connections - active_connections,
        }
    }

    /// Clean up inactive connections
    pub async fn cleanup_inactive_connections(&self, inactive_threshold_minutes: i64) -> Result<usize> {
        let threshold = Utc::now() - chrono::Duration::minutes(inactive_threshold_minutes);
        let mut connections = self.connections.write().await;
        let mut subscriptions = self.subscriptions.write().await;

        let inactive_connections: Vec<String> = connections.iter()
            .filter(|(_, conn)| conn.last_activity < threshold)
            .map(|(id, _)| id.clone())
            .collect();

        let cleanup_count = inactive_connections.len();

        for connection_id in inactive_connections {
            connections.remove(&connection_id);
            subscriptions.remove(&connection_id);
        }

        Ok(cleanup_count)
    }

    /// Generate sample events for testing
    pub async fn generate_sample_events(&self) -> Result<()> {
        // Control status update
        self.broadcast_event(RealtimeEvent::ControlStatusUpdated {
            control_id: "ac-1".to_string(),
            old_status: "in-progress".to_string(),
            new_status: "implemented".to_string(),
            updated_by: Some("admin".to_string()),
            timestamp: Utc::now(),
        }).await?;

        // Metrics update
        self.broadcast_event(RealtimeEvent::MetricsUpdated {
            metrics: MetricsUpdate {
                implementation_percentage: 76.5,
                total_controls: 100,
                implemented_controls: 77,
                in_progress_controls: 15,
                not_implemented_controls: 8,
                changed_metrics: vec!["implementation_percentage".to_string()],
            },
            timestamp: Utc::now(),
        }).await?;

        // Alert generation
        self.broadcast_event(RealtimeEvent::AlertGenerated {
            alert: AlertEvent {
                id: Uuid::new_v4().to_string(),
                title: "Overdue Control Detected".to_string(),
                message: "Control AC-2 is overdue by 5 days".to_string(),
                severity: AlertSeverity::Warning,
                category: AlertCategory::OverdueControl,
                related_control: Some("ac-2".to_string()),
                auto_generated: true,
            },
            timestamp: Utc::now(),
        }).await?;

        Ok(())
    }
}

/// Connection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub inactive_connections: usize,
}

impl Default for RealtimeService {
    fn default() -> Self {
        Self::new()
    }
}
