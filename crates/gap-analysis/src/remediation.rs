//! Remediation Planning
//!
//! Generates actionable remediation plans with timelines, resource estimates,
//! and implementation guidance for identified compliance gaps.

use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use crate::engine::{Gap, GapType, GapSeverity};
use crate::prioritization::{PrioritizedGap, PriorityCategory};

/// Remediation planning engine
#[derive(Debug, Clone)]
pub struct RemediationPlanner {
    /// Plan templates for different gap types
    pub plan_templates: HashMap<GapType, RemediationTemplate>,
    /// Resource estimation algorithms
    pub resource_estimator: ResourceEstimator,
    /// Timeline planning engine
    pub timeline_planner: TimelinePlanner,
    /// Configuration settings
    pub config: RemediationConfig,
}

/// Remediation plan for a set of gaps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationPlan {
    pub plan_id: String,
    pub plan_name: String,
    pub created_at: DateTime<Utc>,
    pub target_completion: DateTime<Utc>,
    pub plan_status: PlanStatus,
    pub remediation_items: Vec<RemediationItem>,
    pub resource_summary: ResourceSummary,
    pub timeline: RemediationTimeline,
    pub dependencies: Vec<Dependency>,
    pub milestones: Vec<Milestone>,
    pub risk_assessment: PlanRiskAssessment,
    pub metadata: RemediationMetadata,
}

/// Individual remediation item for a specific gap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationItem {
    pub item_id: String,
    pub gap_id: String,
    pub control_id: String,
    pub title: String,
    pub description: String,
    pub priority: PriorityCategory,
    pub status: ItemStatus,
    pub assigned_to: Option<String>,
    pub estimated_effort: EffortEstimate,
    pub actual_effort: Option<EffortActual>,
    pub start_date: Option<DateTime<Utc>>,
    pub target_date: DateTime<Utc>,
    pub completion_date: Option<DateTime<Utc>>,
    pub implementation_steps: Vec<ImplementationStep>,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub verification_methods: Vec<VerificationMethod>,
    pub dependencies: Vec<String>,
    pub deliverables: Vec<Deliverable>,
}

/// Remediation template for gap types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationTemplate {
    pub template_id: String,
    pub gap_type: GapType,
    pub template_name: String,
    pub description: String,
    pub standard_steps: Vec<StandardStep>,
    pub effort_multipliers: EffortMultipliers,
    pub resource_requirements: StandardResourceRequirements,
    pub typical_duration: Duration,
    pub complexity_factors: Vec<ComplexityFactor>,
}

/// Resource estimation engine
#[derive(Debug, Clone)]
pub struct ResourceEstimator {
    pub estimation_models: HashMap<String, EstimationModel>,
    pub historical_data: HistoricalData,
    pub adjustment_factors: AdjustmentFactors,
}

/// Timeline planning engine
#[derive(Debug, Clone)]
pub struct TimelinePlanner {
    pub scheduling_algorithms: Vec<SchedulingAlgorithm>,
    pub dependency_resolver: DependencyResolver,
    pub resource_leveling: ResourceLeveling,
}

/// Plan status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlanStatus {
    Draft,
    UnderReview,
    Approved,
    InProgress,
    OnHold,
    Completed,
    Cancelled,
}

/// Item status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ItemStatus {
    NotStarted,
    InProgress,
    Blocked,
    UnderReview,
    Completed,
    Cancelled,
}

/// Effort estimate with confidence intervals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortEstimate {
    pub optimistic_hours: f64,
    pub most_likely_hours: f64,
    pub pessimistic_hours: f64,
    pub expected_hours: f64,
    pub confidence_level: f64,
    pub estimation_method: String,
}

/// Actual effort tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortActual {
    pub actual_hours: f64,
    pub variance_percentage: f64,
    pub tracking_notes: String,
}

/// Implementation step with detailed guidance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationStep {
    pub step_id: String,
    pub step_number: u32,
    pub title: String,
    pub description: String,
    pub estimated_duration: Duration,
    pub required_skills: Vec<String>,
    pub tools_required: Vec<String>,
    pub deliverables: Vec<String>,
    pub verification_criteria: Vec<String>,
    pub dependencies: Vec<String>,
}

/// Acceptance criteria for remediation items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub criterion_id: String,
    pub description: String,
    pub verification_method: String,
    pub success_criteria: String,
    pub is_mandatory: bool,
}

/// Verification methods for completed remediation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub method_id: String,
    pub method_type: VerificationMethodType,
    pub description: String,
    pub verification_steps: Vec<String>,
    pub evidence_requirements: Vec<String>,
    pub responsible_party: String,
}

/// Types of verification methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerificationMethodType {
    DocumentReview,
    TechnicalTesting,
    InterviewAssessment,
    ObservationReview,
    AutomatedScanning,
    PenetrationTesting,
}

/// Deliverable for remediation items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deliverable {
    pub deliverable_id: String,
    pub name: String,
    pub description: String,
    pub deliverable_type: DeliverableType,
    pub due_date: DateTime<Utc>,
    pub responsible_party: String,
    pub approval_required: bool,
}

/// Types of deliverables
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeliverableType {
    Policy,
    Procedure,
    TechnicalConfiguration,
    Documentation,
    TrainingMaterial,
    TestResults,
    Evidence,
}

/// Resource summary for the entire plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSummary {
    pub total_effort_hours: f64,
    pub total_cost_estimate: f64,
    pub personnel_requirements: PersonnelRequirements,
    pub tool_requirements: Vec<ToolRequirement>,
    pub external_services: Vec<ExternalService>,
    pub budget_breakdown: BudgetBreakdown,
}

/// Personnel requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonnelRequirements {
    pub roles_required: HashMap<String, RoleRequirement>,
    pub total_fte_months: f64,
    pub peak_concurrent_resources: u32,
}

/// Role requirement details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleRequirement {
    pub role_name: String,
    pub required_skills: Vec<String>,
    pub experience_level: ExperienceLevel,
    pub estimated_hours: f64,
    pub hourly_rate_range: (f64, f64),
}

/// Experience levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExperienceLevel {
    Junior,
    Intermediate,
    Senior,
    Expert,
}

/// Tool requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequirement {
    pub tool_name: String,
    pub tool_type: String,
    pub license_cost: f64,
    pub duration_months: u32,
    pub justification: String,
}

/// External service requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalService {
    pub service_name: String,
    pub service_type: String,
    pub estimated_cost: f64,
    pub duration: String,
    pub justification: String,
}

/// Budget breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetBreakdown {
    pub personnel_costs: f64,
    pub tool_costs: f64,
    pub external_service_costs: f64,
    pub training_costs: f64,
    pub contingency_percentage: f64,
    pub total_budget: f64,
}

/// Remediation timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationTimeline {
    pub phases: Vec<RemediationPhase>,
    pub critical_path: Vec<String>,
    pub total_duration_days: u32,
    pub parallel_tracks: Vec<ParallelTrack>,
}

/// Remediation phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationPhase {
    pub phase_id: String,
    pub phase_name: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub items: Vec<String>,
    pub dependencies: Vec<String>,
    pub deliverables: Vec<String>,
}

/// Parallel execution track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelTrack {
    pub track_id: String,
    pub track_name: String,
    pub items: Vec<String>,
    pub resource_requirements: String,
}

/// Dependency between remediation items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub dependency_id: String,
    pub predecessor_item: String,
    pub successor_item: String,
    pub dependency_type: DependencyType,
    pub lag_days: i32,
    pub description: String,
}

/// Types of dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DependencyType {
    FinishToStart,
    StartToStart,
    FinishToFinish,
    StartToFinish,
}

/// Milestone in remediation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub milestone_id: String,
    pub name: String,
    pub description: String,
    pub target_date: DateTime<Utc>,
    pub completion_criteria: Vec<String>,
    pub stakeholders: Vec<String>,
    pub is_critical: bool,
}

/// Risk assessment for the plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanRiskAssessment {
    pub identified_risks: Vec<PlanRisk>,
    pub overall_risk_level: RiskLevel,
    pub mitigation_strategies: Vec<MitigationStrategy>,
    pub contingency_plans: Vec<ContingencyPlan>,
}

/// Individual risk in the plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanRisk {
    pub risk_id: String,
    pub risk_description: String,
    pub probability: f64,
    pub impact: RiskLevel,
    pub risk_score: f64,
    pub mitigation_actions: Vec<String>,
}

/// Risk levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Mitigation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationStrategy {
    pub strategy_id: String,
    pub risk_ids: Vec<String>,
    pub strategy_description: String,
    pub implementation_cost: f64,
    pub effectiveness: f64,
}

/// Contingency plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContingencyPlan {
    pub plan_id: String,
    pub trigger_conditions: Vec<String>,
    pub response_actions: Vec<String>,
    pub resource_requirements: String,
    pub activation_criteria: String,
}

/// Remediation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationMetadata {
    pub planner_version: String,
    pub planning_methodology: String,
    pub confidence_level: f64,
    pub assumptions: Vec<String>,
    pub constraints: Vec<String>,
    pub success_metrics: Vec<String>,
}

/// Configuration for remediation planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationConfig {
    pub default_contingency_percentage: f64,
    pub resource_buffer_percentage: f64,
    pub max_parallel_items: u32,
    pub default_working_hours_per_day: f64,
    pub holiday_calendar: String,
}

impl RemediationPlanner {
    /// Create a new remediation planner
    pub fn new() -> Self {
        Self {
            plan_templates: HashMap::new(),
            resource_estimator: ResourceEstimator::new(),
            timeline_planner: TimelinePlanner::new(),
            config: RemediationConfig::default(),
        }
    }

    /// Generate comprehensive remediation plan
    pub async fn generate_plan(
        &self,
        prioritized_gaps: &[PrioritizedGap],
        plan_name: String,
    ) -> Result<RemediationPlan> {
        let plan_id = Uuid::new_v4().to_string();
        let created_at = Utc::now();

        // Generate remediation items for each gap
        let mut remediation_items = Vec::new();
        for prioritized_gap in prioritized_gaps {
            let item = self.generate_remediation_item(prioritized_gap).await?;
            remediation_items.push(item);
        }

        // Calculate resource summary
        let resource_summary = self.calculate_resource_summary(&remediation_items).await?;

        // Generate timeline
        let timeline = self.generate_timeline(&remediation_items).await?;

        // Calculate target completion date
        let target_completion = created_at + Duration::days(timeline.total_duration_days as i64);

        // Generate dependencies
        let dependencies = self.analyze_dependencies(&remediation_items).await?;

        // Generate milestones
        let milestones = self.generate_milestones(&remediation_items, &timeline).await?;

        // Assess plan risks
        let risk_assessment = self.assess_plan_risks(&remediation_items).await?;

        Ok(RemediationPlan {
            plan_id,
            plan_name,
            created_at,
            target_completion,
            plan_status: PlanStatus::Draft,
            remediation_items,
            resource_summary,
            timeline,
            dependencies,
            milestones,
            risk_assessment,
            metadata: RemediationMetadata {
                planner_version: "1.0.0".to_string(),
                planning_methodology: "Risk-based prioritization with resource optimization".to_string(),
                confidence_level: 0.8,
                assumptions: vec![
                    "Resources will be available as planned".to_string(),
                    "No major scope changes during implementation".to_string(),
                ],
                constraints: vec![
                    "Budget limitations may affect timeline".to_string(),
                    "Regulatory deadlines must be met".to_string(),
                ],
                success_metrics: vec![
                    "All critical gaps remediated within 6 months".to_string(),
                    "95% of planned deliverables completed on time".to_string(),
                ],
            },
        })
    }

    /// Generate remediation item for a prioritized gap
    async fn generate_remediation_item(&self, prioritized_gap: &PrioritizedGap) -> Result<RemediationItem> {
        let item_id = Uuid::new_v4().to_string();
        let gap = &prioritized_gap.gap;

        // Get template for gap type (create default template if not found)
        let default_template = RemediationTemplate {
            template_id: "default".to_string(),
            gap_type: gap.gap_type.clone(),
            template_name: "Default Template".to_string(),
            description: "Default remediation template".to_string(),
            standard_steps: Vec::new(),
            effort_multipliers: EffortMultipliers,
            resource_requirements: StandardResourceRequirements,
            typical_duration: chrono::Duration::days(30),
            complexity_factors: Vec::new(),
        };
        let template = self.plan_templates.get(&gap.gap_type).unwrap_or(&default_template);

        // Generate implementation steps
        let implementation_steps = self.generate_implementation_steps(gap, template).await?;

        // Estimate effort
        let estimated_effort = self.resource_estimator.estimate_effort(gap, template).await?;

        // Calculate target date based on priority
        let target_date = self.calculate_target_date(&prioritized_gap.priority_category);

        Ok(RemediationItem {
            item_id,
            gap_id: gap.gap_id.clone(),
            control_id: gap.control_id.clone(),
            title: format!("Remediate {} gap in {}", gap.gap_type, gap.control_id),
            description: gap.description.clone(),
            priority: prioritized_gap.priority_category.clone(),
            status: ItemStatus::NotStarted,
            assigned_to: None,
            estimated_effort,
            actual_effort: None,
            start_date: None,
            target_date,
            completion_date: None,
            implementation_steps,
            acceptance_criteria: self.generate_acceptance_criteria(gap).await?,
            verification_methods: self.generate_verification_methods(gap).await?,
            dependencies: Vec::new(), // Will be populated later
            deliverables: self.generate_deliverables(gap).await?,
        })
    }

    // Placeholder implementations for remaining methods
    async fn generate_implementation_steps(&self, _gap: &Gap, _template: &RemediationTemplate) -> Result<Vec<ImplementationStep>> {
        Ok(Vec::new())
    }

    async fn calculate_resource_summary(&self, _items: &[RemediationItem]) -> Result<ResourceSummary> {
        Ok(ResourceSummary {
            total_effort_hours: 0.0,
            total_cost_estimate: 0.0,
            personnel_requirements: PersonnelRequirements {
                roles_required: HashMap::new(),
                total_fte_months: 0.0,
                peak_concurrent_resources: 0,
            },
            tool_requirements: Vec::new(),
            external_services: Vec::new(),
            budget_breakdown: BudgetBreakdown {
                personnel_costs: 0.0,
                tool_costs: 0.0,
                external_service_costs: 0.0,
                training_costs: 0.0,
                contingency_percentage: 15.0,
                total_budget: 0.0,
            },
        })
    }

    async fn generate_timeline(&self, _items: &[RemediationItem]) -> Result<RemediationTimeline> {
        Ok(RemediationTimeline {
            phases: Vec::new(),
            critical_path: Vec::new(),
            total_duration_days: 90,
            parallel_tracks: Vec::new(),
        })
    }

    async fn analyze_dependencies(&self, _items: &[RemediationItem]) -> Result<Vec<Dependency>> {
        Ok(Vec::new())
    }

    async fn generate_milestones(&self, _items: &[RemediationItem], _timeline: &RemediationTimeline) -> Result<Vec<Milestone>> {
        Ok(Vec::new())
    }

    async fn assess_plan_risks(&self, _items: &[RemediationItem]) -> Result<PlanRiskAssessment> {
        Ok(PlanRiskAssessment {
            identified_risks: Vec::new(),
            overall_risk_level: RiskLevel::Medium,
            mitigation_strategies: Vec::new(),
            contingency_plans: Vec::new(),
        })
    }

    async fn generate_acceptance_criteria(&self, _gap: &Gap) -> Result<Vec<AcceptanceCriterion>> {
        Ok(Vec::new())
    }

    async fn generate_verification_methods(&self, _gap: &Gap) -> Result<Vec<VerificationMethod>> {
        Ok(Vec::new())
    }

    async fn generate_deliverables(&self, _gap: &Gap) -> Result<Vec<Deliverable>> {
        Ok(Vec::new())
    }

    fn calculate_target_date(&self, priority: &PriorityCategory) -> DateTime<Utc> {
        let days_offset = match priority {
            PriorityCategory::Critical => 30,
            PriorityCategory::High => 60,
            PriorityCategory::Medium => 120,
            PriorityCategory::Low => 180,
        };
        Utc::now() + Duration::days(days_offset)
    }
}

// Placeholder implementations for supporting types
impl ResourceEstimator {
    pub fn new() -> Self {
        Self {
            estimation_models: HashMap::new(),
            historical_data: HistoricalData,
            adjustment_factors: AdjustmentFactors,
        }
    }

    pub async fn estimate_effort(&self, _gap: &Gap, _template: &RemediationTemplate) -> Result<EffortEstimate> {
        Ok(EffortEstimate {
            optimistic_hours: 40.0,
            most_likely_hours: 80.0,
            pessimistic_hours: 160.0,
            expected_hours: 88.0,
            confidence_level: 0.7,
            estimation_method: "Three-point estimation".to_string(),
        })
    }
}

impl TimelinePlanner {
    pub fn new() -> Self {
        Self {
            scheduling_algorithms: Vec::new(),
            dependency_resolver: DependencyResolver,
            resource_leveling: ResourceLeveling,
        }
    }
}

impl Default for RemediationConfig {
    fn default() -> Self {
        Self {
            default_contingency_percentage: 15.0,
            resource_buffer_percentage: 10.0,
            max_parallel_items: 5,
            default_working_hours_per_day: 8.0,
            holiday_calendar: "US".to_string(),
        }
    }
}

// Placeholder types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)] pub struct StandardStep;
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)] pub struct EffortMultipliers;
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)] pub struct StandardResourceRequirements;
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)] pub struct ComplexityFactor;
#[derive(Debug, Clone)] pub struct EstimationModel;
#[derive(Debug, Clone)] pub struct HistoricalData;
#[derive(Debug, Clone)] pub struct AdjustmentFactors;
#[derive(Debug, Clone)] pub struct SchedulingAlgorithm;
#[derive(Debug, Clone)] pub struct DependencyResolver;
#[derive(Debug, Clone)] pub struct ResourceLeveling;
