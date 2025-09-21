//! Compliance Dashboard Server
//!
//! A standalone server for the FedRAMP compliance dashboard that serves
//! both the API endpoints and static frontend files.

use compliance_dashboard::{ComplianceDashboard, start_server};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🚀 Starting FedRAMP Compliance Dashboard Server");
    println!("================================================");

    // Get port from environment or use default
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    // Create dashboard with sample data
    let mut dashboard = ComplianceDashboard::with_sample_data();
    
    // Initialize the dashboard
    dashboard.initialize().await?;

    println!("✅ Dashboard initialized with sample data");
    println!("📊 Available endpoints:");
    println!("   GET  /api/dashboard        - Dashboard overview");
    println!("   GET  /api/dashboard/metrics - Metrics data");
    println!("   GET  /api/dashboard/widgets - Widget configuration");
    println!("   GET  /api/controls         - All controls");
    println!("   GET  /api/controls/:id     - Specific control");
    println!("   PUT  /api/controls/:id/status - Update control status");
    println!("   GET  /api/frameworks       - All frameworks");
    println!("   GET  /api/frameworks/:id/controls - Framework controls");
    println!("   GET  /api/realtime/stats   - Real-time connection stats");
    println!("   GET  /api/realtime/ws      - WebSocket endpoint");
    println!("   GET  /health               - Health check");
    println!();
    println!("🌐 Frontend available at: http://localhost:3000");
    println!("🔌 API available at: http://localhost:{}", port);
    println!();
    println!("📈 Sample data includes:");
    println!("   • 3 sample controls (AC-1, AC-2, AC-3)");
    println!("   • 2 frameworks (NIST 800-53, NIST 800-171)");
    println!("   • Real-time metrics and widgets");
    println!("   • WebSocket support for live updates");
    println!();

    // Start the server
    start_server(dashboard, port).await?;

    Ok(())
}
