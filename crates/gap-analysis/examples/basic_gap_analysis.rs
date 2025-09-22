//! Basic Gap Analysis Example
//!
//! Demonstrates how to use the gap analysis tool to identify compliance gaps
//! and generate remediation plans.

use gap_analysis::{
    GapAnalysisService, 
    engine::{CurrentImplementation, ControlImplementation, ImplementationStatus},
    GapAnalysisServiceConfig
};
use std::collections::HashMap;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 FedRAMP Gap Analysis Tool - Basic Example");
    println!("============================================");

    // Initialize the gap analysis service with JSON baseline loader
    let mappings_path = "mappings/control_mappings.json".to_string();
    let mut gap_service = match GapAnalysisService::with_json_baselines(mappings_path) {
        Ok(service) => service,
        Err(e) => {
            println!("❌ Failed to initialize gap analysis service: {}", e);
            println!("💡 Make sure the mappings/control_mappings.json file exists");
            return Ok(());
        }
    };

    // Configure the service
    let config = GapAnalysisServiceConfig {
        default_framework: "nist_800_53_rev5".to_string(),
        default_profile: "moderate".to_string(),
        auto_prioritize: true,
        auto_generate_plans: true,
        cache_results: true,
        max_gaps_per_analysis: 100,
    };
    gap_service.update_config(config);

    // Display available frameworks and profiles
    println!("\n📋 Available Frameworks:");
    match gap_service.get_available_frameworks() {
        Ok(frameworks) => {
            for framework in &frameworks {
                println!("  • {}", framework);
                
                // Show available profiles for each framework
                if let Ok(profiles) = gap_service.get_available_profiles(framework) {
                    for profile in profiles {
                        println!("    - {}", profile);
                    }
                }
            }
        }
        Err(e) => println!("  ❌ Error loading frameworks: {}", e),
    }

    // Create a sample current implementation with some gaps
    let current_implementation = create_sample_implementation();
    
    println!("\n🔍 Analyzing compliance gaps...");
    println!("Framework: NIST 800-53 Rev 5");
    println!("Profile: Moderate");
    println!("Current Implementation: {} controls", current_implementation.controls.len());

    // Execute the gap analysis workflow
    match gap_service.execute_workflow(
        &current_implementation,
        Some("nist_800_53_rev5".to_string()),
        Some("moderate".to_string()),
    ).await {
        Ok(workflow_result) => {
            display_analysis_results(&workflow_result);
        }
        Err(e) => {
            println!("❌ Gap analysis failed: {}", e);
            println!("💡 This might be due to missing baseline data or configuration issues");
        }
    }

    // Display service statistics
    println!("\n📊 Service Statistics:");
    let stats = gap_service.get_service_statistics();
    println!("  • Available Frameworks: {}", stats.available_frameworks);
    println!("  • Cached Baselines: {}", stats.cached_baselines);
    println!("  • Total Analyses: {}", stats.total_analyses_performed);

    println!("\n✅ Gap analysis example completed!");
    Ok(())
}

/// Create a sample current implementation with various control statuses
fn create_sample_implementation() -> CurrentImplementation {
    let mut controls = HashMap::new();

    // Add some implemented controls
    controls.insert("AC-1".to_string(), ControlImplementation {
        control_id: "AC-1".to_string(),
        status: ImplementationStatus::Implemented,
        implementation_date: Some(Utc::now()),
        evidence: vec![],
        parameters: HashMap::new(),
    });

    controls.insert("AC-2".to_string(), ControlImplementation {
        control_id: "AC-2".to_string(),
        status: ImplementationStatus::Implemented,
        implementation_date: Some(Utc::now()),
        evidence: vec![],
        parameters: HashMap::new(),
    });

    // Add some partially implemented controls
    controls.insert("AU-1".to_string(), ControlImplementation {
        control_id: "AU-1".to_string(),
        status: ImplementationStatus::PartiallyImplemented,
        implementation_date: None,
        evidence: vec![],
        parameters: HashMap::new(),
    });

    // Add some planned controls
    controls.insert("CA-1".to_string(), ControlImplementation {
        control_id: "CA-1".to_string(),
        status: ImplementationStatus::Planned,
        implementation_date: None,
        evidence: vec![],
        parameters: HashMap::new(),
    });

    // Add some not implemented controls (these will show as gaps)
    controls.insert("CM-1".to_string(), ControlImplementation {
        control_id: "CM-1".to_string(),
        status: ImplementationStatus::NotImplemented,
        implementation_date: None,
        evidence: vec![],
        parameters: HashMap::new(),
    });

    CurrentImplementation {
        system_id: "sample-system".to_string(),
        controls,
        last_updated: Utc::now(),
    }
}

/// Display the gap analysis results in a user-friendly format
fn display_analysis_results(workflow_result: &gap_analysis::GapAnalysisWorkflowResult) {
    let analysis = &workflow_result.analysis_result;
    let prioritized_gaps = &workflow_result.prioritized_gaps;
    let matrix = &workflow_result.prioritization_matrix;

    println!("\n📊 Gap Analysis Results:");
    println!("========================");
    
    // Summary statistics
    println!("📈 Summary:");
    println!("  • Total Gaps Found: {}", analysis.summary.total_gaps);
    println!("  • Overall Compliance Score: {:.1}%", analysis.summary.overall_compliance_score);
    println!("  • Readiness Assessment: {:?}", analysis.summary.readiness_assessment);

    // Gap breakdown by severity
    println!("\n🚨 Gaps by Severity:");
    for (severity, count) in &analysis.summary.gaps_by_severity {
        println!("  • {:?}: {}", severity, count);
    }

    // Gap breakdown by type
    println!("\n📋 Gaps by Type:");
    for (gap_type, count) in &analysis.summary.gaps_by_type {
        println!("  • {}: {}", gap_type, count);
    }

    // Top priority gaps
    println!("\n🎯 Top Priority Gaps:");
    for (i, prioritized_gap) in prioritized_gaps.iter().take(5).enumerate() {
        println!("  {}. {} ({})", 
            i + 1, 
            prioritized_gap.gap.control_id,
            prioritized_gap.gap.gap_type
        );
        println!("     Priority: {:?} (Score: {:.2})", 
            prioritized_gap.priority_category,
            prioritized_gap.priority_score
        );
        println!("     Description: {}", prioritized_gap.gap.description);
    }

    // Prioritization matrix summary
    println!("\n📊 Prioritization Matrix:");
    println!("  • Quick Wins: {} gaps", matrix.quadrants.quick_wins.len());
    println!("  • Major Projects: {} gaps", matrix.quadrants.major_projects.len());
    println!("  • Fill-ins: {} gaps", matrix.quadrants.fill_ins.len());
    println!("  • Questionable: {} gaps", matrix.quadrants.questionable.len());

    // Remediation plan summary (if generated)
    if let Some(plan) = &workflow_result.remediation_plan {
        println!("\n🛠️  Remediation Plan:");
        println!("  • Plan ID: {}", plan.plan_id);
        println!("  • Total Items: {}", plan.remediation_items.len());
        println!("  • Target Completion: {}", plan.target_completion.format("%Y-%m-%d"));
        println!("  • Estimated Effort: {:.0} hours", plan.resource_summary.total_effort_hours);
        println!("  • Estimated Cost: ${:.0}", plan.resource_summary.total_cost_estimate);
    }

    // Workflow performance
    let metadata = &workflow_result.workflow_metadata;
    println!("\n⚡ Performance Metrics:");
    println!("  • Total Execution Time: {}ms", metadata.execution_time_ms);
    println!("  • Steps Completed: {}", metadata.steps_completed.len());
    
    if !metadata.warnings.is_empty() {
        println!("  • Warnings: {}", metadata.warnings.len());
        for warning in &metadata.warnings {
            println!("    ⚠️  {}", warning);
        }
    }

    // Recommendations
    println!("\n💡 Recommendations:");
    for (i, recommendation) in analysis.recommendations.iter().take(3).enumerate() {
        println!("  {}. {}", i + 1, recommendation.title);
        println!("     {}", recommendation.description);
        println!("     Priority: {:?}", recommendation.priority);
    }

    println!("\n🎯 Next Steps:");
    println!("  1. Review the top priority gaps and their remediation guidance");
    println!("  2. Focus on 'Quick Wins' from the prioritization matrix");
    println!("  3. Develop detailed implementation plans for critical gaps");
    println!("  4. Establish timelines and assign resources");
    println!("  5. Monitor progress and update the analysis regularly");
}
