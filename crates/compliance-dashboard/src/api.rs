//! API endpoints for the compliance dashboard
//!
//! This module provides REST API endpoints for serving dashboard data,
//! handling control updates, and managing real-time connections.

use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use crate::{ComplianceDashboard, ImplementationStatus, DashboardData};

/// API state containing the dashboard instance
pub type ApiState = Arc<RwLock<ComplianceDashboard>>;

/// Create the API router
pub fn create_router(dashboard: ComplianceDashboard) -> Router {
    let state = Arc::new(RwLock::new(dashboard));

    Router::new()
        .route("/api/dashboard", get(get_dashboard_overview))
        .route("/api/dashboard/metrics", get(get_metrics))
        .route("/api/dashboard/widgets", get(get_widgets))
        .route("/api/controls", get(get_controls))
        .route("/api/controls/:id", get(get_control))
        .route("/api/controls/:id/status", put(update_control_status))
        .route("/api/frameworks", get(get_frameworks))
        .route("/api/frameworks/:id/controls", get(get_framework_controls))
        .route("/api/realtime/stats", get(get_realtime_stats))
        .route("/api/realtime/ws", get(websocket_handler))
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Get dashboard overview
async fn get_dashboard_overview(State(state): State<ApiState>) -> Result<Json<DashboardData>, ApiError> {
    let mut dashboard = state.write().await;
    let data = dashboard.get_dashboard_data().await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(data))
}

/// Get metrics data
async fn get_metrics(State(state): State<ApiState>) -> Result<Json<serde_json::Value>, ApiError> {
    let dashboard = state.read().await;
    let overview = dashboard.dashboard.get_overview()
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let metrics = serde_json::json!({
        "implementation_percentage": overview.implementation_percentage,
        "total_controls": overview.total_controls,
        "implemented_controls": overview.implemented_controls,
        "in_progress_controls": overview.in_progress_controls,
        "not_implemented_controls": overview.not_implemented_controls,
        "last_updated": overview.last_updated
    });
    
    Ok(Json(metrics))
}

/// Get widgets configuration
async fn get_widgets(State(state): State<ApiState>) -> Result<Json<serde_json::Value>, ApiError> {
    let dashboard = state.read().await;
    let widgets = dashboard.widgets.get_widgets();
    Ok(Json(serde_json::json!({ "widgets": widgets })))
}

/// Get all controls
async fn get_controls(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<ApiState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let dashboard = state.read().await;
    let overview = dashboard.dashboard.get_overview()
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let mut controls = overview.recent_updates;
    
    // Apply filters if provided
    if let Some(framework_id) = params.get("framework") {
        controls.retain(|c| c.framework_id == *framework_id);
    }
    
    if let Some(status) = params.get("status") {
        controls.retain(|c| format!("{:?}", c.implementation_status).to_lowercase() == status.to_lowercase());
    }
    
    Ok(Json(serde_json::json!({ "controls": controls })))
}

/// Get specific control
async fn get_control(
    Path(control_id): Path<String>,
    State(state): State<ApiState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let dashboard = state.read().await;
    let overview = dashboard.dashboard.get_overview()
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let control = overview.recent_updates.iter()
        .find(|c| c.id == control_id)
        .ok_or_else(|| ApiError::NotFound("Control not found".to_string()))?;
    
    Ok(Json(serde_json::json!({ "control": control })))
}

/// Update control status
async fn update_control_status(
    Path(control_id): Path<String>,
    State(state): State<ApiState>,
    Json(payload): Json<UpdateControlStatusRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut dashboard = state.write().await;
    
    let status = match payload.status.as_str() {
        "not-implemented" => ImplementationStatus::NotImplemented,
        "in-progress" => ImplementationStatus::InProgress,
        "implemented" => ImplementationStatus::Implemented,
        "not-applicable" => ImplementationStatus::NotApplicable,
        _ => return Err(ApiError::BadRequest("Invalid status".to_string())),
    };
    
    dashboard.update_control_status(&control_id, status).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Control status updated successfully"
    })))
}

/// Get frameworks
async fn get_frameworks(State(state): State<ApiState>) -> Result<Json<serde_json::Value>, ApiError> {
    let dashboard = state.read().await;
    let overview = dashboard.dashboard.get_overview()
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    Ok(Json(serde_json::json!({ "frameworks": overview.frameworks })))
}

/// Get controls for a specific framework
async fn get_framework_controls(
    Path(framework_id): Path<String>,
    State(state): State<ApiState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let dashboard = state.read().await;
    let controls = dashboard.dashboard.get_controls_by_framework(&framework_id)
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    Ok(Json(serde_json::json!({ "controls": controls })))
}

/// Get real-time connection statistics
async fn get_realtime_stats(State(state): State<ApiState>) -> Result<Json<serde_json::Value>, ApiError> {
    let dashboard = state.read().await;
    let stats = dashboard.realtime.get_connection_stats().await;
    Ok(Json(serde_json::json!({ "stats": stats })))
}

/// WebSocket handler for real-time updates
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(_state): State<ApiState>,
) -> axum::response::Response {
    ws.on_upgrade(|socket| async move {
        // In a real implementation, this would handle WebSocket connections
        // For now, we'll just close the connection
        let _ = socket;
        println!("WebSocket connection established (placeholder implementation)");
    })
}

/// Health check endpoint
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "service": "compliance-dashboard"
    }))
}

/// Request payload for updating control status
#[derive(Debug, Deserialize)]
struct UpdateControlStatusRequest {
    status: String,
}

/// API error types
#[derive(Debug)]
enum ApiError {
    BadRequest(String),
    NotFound(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(serde_json::json!({
            "error": message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

/// Start the API server
pub async fn start_server(dashboard: ComplianceDashboard, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_router(dashboard);
    let addr = format!("0.0.0.0:{}", port);
    
    println!("🚀 Compliance Dashboard API server starting on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
