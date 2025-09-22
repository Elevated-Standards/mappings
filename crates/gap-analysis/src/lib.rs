//! FedRAMP Gap Analysis Tool
//!
//! Comprehensive gap analysis system for identifying compliance gaps across
//! security frameworks and generating actionable remediation plans.

pub mod engine;
pub mod baseline;
pub mod prioritization;
pub mod remediation;

pub use engine::{GapAnalysisEngine, GapAnalysisResult, Gap, GapType, GapSeverity, ImplementationStatus, TargetBaseline};
pub use baseline::{BaselineManager, ValidationResult};
pub use prioritization::{PrioritizationEngine, PrioritizedGap, PriorityCategory, PrioritizationMatrix};
pub use remediation::{RemediationPlanner, RemediationPlan, RemediationItem};

use fedramp_core::Result;
use std::collections::HashMap;

/// Main gap analysis service integrating all components
#[derive(Debug, Clone)]
pub struct GapAnalysisService {
    /// Core gap analysis engine
    pub engine: GapAnalysisEngine,
    /// Baseline management
    pub baseline_manager: BaselineManager,
    /// Gap prioritization
    pub prioritization_engine: PrioritizationEngine,
    /// Remediation planning
    pub remediation_planner: RemediationPlanner,
    /// Service configuration
    pub config: GapAnalysisServiceConfig,
}

/// Configuration for the gap analysis service
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GapAnalysisServiceConfig {
    /// Default framework to use for analysis
    pub default_framework: String,
    /// Default baseline profile
    pub default_profile: String,
    /// Enable automatic prioritization
    pub auto_prioritize: bool,
    /// Enable automatic remediation planning
    pub auto_generate_plans: bool,
    /// Cache analysis results
    pub cache_results: bool,
    /// Maximum gaps to analyze in one batch
    pub max_gaps_per_analysis: usize,
}

/// Complete gap analysis workflow result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GapAnalysisWorkflowResult {
    /// Gap analysis results
    pub analysis_result: GapAnalysisResult,
    /// Prioritized gaps
    pub prioritized_gaps: Vec<PrioritizedGap>,
    /// Prioritization matrix for visualization
    pub prioritization_matrix: PrioritizationMatrix,
    /// Generated remediation plan
    pub remediation_plan: Option<RemediationPlan>,
    /// Workflow metadata
    pub workflow_metadata: WorkflowMetadata,
}

/// Workflow execution metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowMetadata {
    pub workflow_id: String,
    pub execution_time_ms: u64,
    pub steps_completed: Vec<String>,
    pub warnings: Vec<String>,
    pub performance_metrics: HashMap<String, f64>,
}

impl GapAnalysisService {
    /// Create a new gap analysis service
    pub fn new() -> Self {
        Self {
            engine: GapAnalysisEngine::new(),
            baseline_manager: BaselineManager::new(),
            prioritization_engine: PrioritizationEngine::new(),
            remediation_planner: RemediationPlanner::new(),
            config: GapAnalysisServiceConfig::default(),
        }
    }

    /// Create gap analysis service with JSON baseline loader
    pub fn with_json_baselines(mappings_path: String) -> Result<Self> {
        let baseline_manager = BaselineManager::with_json_loader(mappings_path)?;

        Ok(Self {
            engine: GapAnalysisEngine::new(),
            baseline_manager,
            prioritization_engine: PrioritizationEngine::new(),
            remediation_planner: RemediationPlanner::new(),
            config: GapAnalysisServiceConfig::default(),
        })
    }

    /// Execute complete gap analysis workflow
    pub async fn execute_workflow(
        &mut self,
        current_implementation: &engine::CurrentImplementation,
        framework_id: Option<String>,
        profile: Option<String>,
    ) -> Result<GapAnalysisWorkflowResult> {
        let workflow_id = uuid::Uuid::new_v4().to_string();
        let start_time = std::time::Instant::now();
        let mut steps_completed = Vec::new();
        let mut warnings = Vec::new();
        let mut performance_metrics = HashMap::new();

        // Step 1: Load target baseline
        let framework = framework_id.unwrap_or_else(|| self.config.default_framework.clone());
        let profile_name = profile.unwrap_or_else(|| self.config.default_profile.clone());

        let step_start = std::time::Instant::now();
        let target_baseline = self.baseline_manager.get_baseline(&framework, &profile_name).await?;
        performance_metrics.insert("baseline_loading_ms".to_string(), step_start.elapsed().as_millis() as f64);
        steps_completed.push("baseline_loading".to_string());

        // Step 2: Perform gap analysis
        let step_start = std::time::Instant::now();
        let analysis_result = self.engine.analyze_gaps(current_implementation, &target_baseline).await?;
        performance_metrics.insert("gap_analysis_ms".to_string(), step_start.elapsed().as_millis() as f64);
        steps_completed.push("gap_analysis".to_string());

        // Step 3: Prioritize gaps
        let step_start = std::time::Instant::now();
        let prioritized_gaps = if self.config.auto_prioritize {
            self.prioritization_engine.prioritize_gaps(&analysis_result.gaps).await?
        } else {
            // Convert gaps to prioritized gaps with default priority
            analysis_result.gaps.iter().enumerate().map(|(index, gap)| {
                PrioritizedGap {
                    gap: gap.clone(),
                    priority_score: 0.5,
                    priority_category: PriorityCategory::Medium,
                    priority_rank: index + 1,
                    scoring_breakdown: prioritization::ScoringBreakdown {
                        risk_score: 0.5,
                        business_impact_score: 0.5,
                        effort_score: 0.5,
                        roi_score: 0.5,
                        compliance_urgency_score: 0.5,
                        stakeholder_priority_score: 0.5,
                        weighted_contributions: HashMap::new(),
                    },
                    metadata: prioritization::PrioritizationMetadata {
                        algorithm_used: prioritization::PrioritizationAlgorithm::SeverityBased,
                        criteria_weights: prioritization::PrioritizationCriteria::default(),
                        confidence: 0.5,
                        alternative_rankings: HashMap::new(),
                    },
                }
            }).collect()
        };
        performance_metrics.insert("prioritization_ms".to_string(), step_start.elapsed().as_millis() as f64);
        steps_completed.push("prioritization".to_string());

        // Step 4: Generate prioritization matrix
        let step_start = std::time::Instant::now();
        let prioritization_matrix = self.prioritization_engine.generate_prioritization_matrix(&prioritized_gaps)?;
        performance_metrics.insert("matrix_generation_ms".to_string(), step_start.elapsed().as_millis() as f64);
        steps_completed.push("matrix_generation".to_string());

        // Step 5: Generate remediation plan (if enabled)
        let remediation_plan = if self.config.auto_generate_plans {
            let step_start = std::time::Instant::now();
            let plan = self.remediation_planner.generate_plan(
                &prioritized_gaps,
                format!("Remediation Plan for {} - {}", framework, profile_name),
            ).await?;
            performance_metrics.insert("remediation_planning_ms".to_string(), step_start.elapsed().as_millis() as f64);
            steps_completed.push("remediation_planning".to_string());
            Some(plan)
        } else {
            None
        };

        let total_execution_time = start_time.elapsed();
        performance_metrics.insert("total_execution_ms".to_string(), total_execution_time.as_millis() as f64);

        // Add performance warnings if needed
        if total_execution_time.as_secs() > 30 {
            warnings.push("Gap analysis took longer than expected (>30s)".to_string());
        }

        if analysis_result.gaps.len() > self.config.max_gaps_per_analysis {
            warnings.push(format!(
                "Large number of gaps identified ({}), consider filtering or batching",
                analysis_result.gaps.len()
            ));
        }

        Ok(GapAnalysisWorkflowResult {
            analysis_result,
            prioritized_gaps,
            prioritization_matrix,
            remediation_plan,
            workflow_metadata: WorkflowMetadata {
                workflow_id,
                execution_time_ms: total_execution_time.as_millis() as u64,
                steps_completed,
                warnings,
                performance_metrics,
            },
        })
    }

    /// Get available frameworks for analysis
    pub fn get_available_frameworks(&self) -> Result<Vec<String>> {
        self.baseline_manager.get_available_frameworks()
    }

    /// Get available profiles for a framework
    pub fn get_available_profiles(&self, framework_id: &str) -> Result<Vec<String>> {
        self.baseline_manager.get_available_profiles(framework_id)
    }

    /// Validate baseline data
    pub async fn validate_baselines(&self) -> Result<HashMap<String, ValidationResult>> {
        self.baseline_manager.validate_baselines().await
    }

    /// Update service configuration
    pub fn update_config(&mut self, config: GapAnalysisServiceConfig) {
        self.config = config;
    }

    /// Get service statistics
    pub fn get_service_statistics(&self) -> ServiceStatistics {
        ServiceStatistics {
            cached_baselines: 0, // TODO: implement actual counting
            available_frameworks: self.get_available_frameworks().unwrap_or_default().len(),
            total_analyses_performed: 0, // TODO: implement tracking
            average_analysis_time_ms: 0.0, // TODO: implement tracking
        }
    }
}

/// Service statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceStatistics {
    pub cached_baselines: usize,
    pub available_frameworks: usize,
    pub total_analyses_performed: usize,
    pub average_analysis_time_ms: f64,
}

impl Default for GapAnalysisServiceConfig {
    fn default() -> Self {
        Self {
            default_framework: "nist-800-53".to_string(),
            default_profile: "moderate".to_string(),
            auto_prioritize: true,
            auto_generate_plans: true,
            cache_results: true,
            max_gaps_per_analysis: 1000,
        }
    }
}

impl Default for GapAnalysisService {
    fn default() -> Self {
        Self::new()
    }
}
