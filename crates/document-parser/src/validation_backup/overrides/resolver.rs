// Modified: 2025-09-22

//! Conflict resolution for override rules
//!
//! This module provides functionality for resolving conflicts between
//! multiple override rules that match the same pattern.

use super::types::*;

/// Conflict resolver for handling overlapping overrides
#[derive(Debug)]
pub struct ConflictResolver {
    /// Strategy for resolving conflicts
    resolution_strategy: ConflictResolutionStrategy,
    /// Maximum number of conflicts to report
    max_conflicts_reported: usize,
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new() -> Self {
        Self {
            resolution_strategy: ConflictResolutionStrategy::HighestPriority,
            max_conflicts_reported: 10,
        }
    }

    /// Create a conflict resolver with custom strategy
    pub fn with_strategy(strategy: ConflictResolutionStrategy) -> Self {
        Self {
            resolution_strategy: strategy,
            max_conflicts_reported: 10,
        }
    }

    /// Set the resolution strategy
    pub fn set_strategy(&mut self, strategy: ConflictResolutionStrategy) {
        self.resolution_strategy = strategy;
    }

    /// Set the maximum number of conflicts to report
    pub fn set_max_conflicts_reported(&mut self, max: usize) {
        self.max_conflicts_reported = max;
    }

    /// Resolve conflicts for a new override
    pub fn resolve_conflicts(
        &self,
        _new_override: &MappingOverride,
        conflicts: &[OverrideConflict],
    ) -> Result<(), String> {
        if conflicts.is_empty() {
            return Ok(());
        }

        // Check if any conflicts are critical
        let critical_conflicts: Vec<_> = conflicts.iter()
            .filter(|c| c.severity == ConflictSeverity::Critical)
            .collect();

        if !critical_conflicts.is_empty() {
            return Err(format!("Critical conflicts detected: {}", 
                critical_conflicts.len()));
        }

        match self.resolution_strategy {
            ConflictResolutionStrategy::Manual => {
                Err("Manual resolution required".to_string())
            }
            ConflictResolutionStrategy::HighestPriority => {
                // Allow conflicts to be resolved by priority
                Ok(())
            }
            ConflictResolutionStrategy::MostRecent => {
                // Allow conflicts to be resolved by creation time
                Ok(())
            }
            ConflictResolutionStrategy::MostSpecific => {
                // Allow conflicts to be resolved by specificity
                Ok(())
            }
            ConflictResolutionStrategy::Combine => {
                // Check if conflicts can be combined
                if self.can_combine_conflicts(conflicts) {
                    Ok(())
                } else {
                    Err("Conflicts cannot be combined".to_string())
                }
            }
        }
    }

    /// Resolve multiple matching overrides
    pub fn resolve_multiple_matches(
        &self,
        matches: &[MappingOverride],
    ) -> Result<MappingOverride, String> {
        if matches.is_empty() {
            return Err("No matches to resolve".to_string());
        }

        if matches.len() == 1 {
            return Ok(matches[0].clone());
        }

        match self.resolution_strategy {
            ConflictResolutionStrategy::HighestPriority => {
                self.resolve_by_priority(matches)
            }
            ConflictResolutionStrategy::MostRecent => {
                self.resolve_by_creation_time(matches)
            }
            ConflictResolutionStrategy::MostSpecific => {
                self.resolve_by_specificity(matches)
            }
            ConflictResolutionStrategy::Combine => {
                self.resolve_by_combination(matches)
            }
            ConflictResolutionStrategy::Manual => {
                Err("Manual resolution required for multiple matches".to_string())
            }
        }
    }

    /// Resolve by highest priority
    fn resolve_by_priority(&self, matches: &[MappingOverride]) -> Result<MappingOverride, String> {
        let highest_priority = matches.iter()
            .max_by_key(|m| m.priority)
            .unwrap();
        Ok(highest_priority.clone())
    }

    /// Resolve by most recent creation time
    fn resolve_by_creation_time(&self, matches: &[MappingOverride]) -> Result<MappingOverride, String> {
        let most_recent = matches.iter()
            .max_by_key(|m| m.created_at)
            .unwrap();
        Ok(most_recent.clone())
    }

    /// Resolve by most specific scope
    fn resolve_by_specificity(&self, matches: &[MappingOverride]) -> Result<MappingOverride, String> {
        // Calculate specificity score for each match
        let mut scored_matches: Vec<_> = matches.iter()
            .map(|m| (m, self.calculate_specificity_score(m)))
            .collect();

        // Sort by specificity score (higher is more specific)
        scored_matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        Ok(scored_matches[0].0.clone())
    }

    /// Resolve by combining rules (if possible)
    fn resolve_by_combination(&self, matches: &[MappingOverride]) -> Result<MappingOverride, String> {
        // For now, just return the highest priority rule
        // In the future, this could create a combined rule
        self.resolve_by_priority(matches)
    }

    /// Calculate specificity score for an override
    fn calculate_specificity_score(&self, override_rule: &MappingOverride) -> f64 {
        let mut score = 0.0;

        // Scope specificity
        score += match &override_rule.scope {
            OverrideScope::Global => 1.0,
            OverrideScope::DocumentType(_) => 2.0,
            OverrideScope::FilePattern(_) => 3.0,
            OverrideScope::User(_) => 4.0,
            OverrideScope::Organization(_) => 3.5,
            OverrideScope::Project(_) => 3.5,
        };

        // Pattern specificity
        score += match override_rule.override_type {
            OverrideType::ExactMatch => 5.0,
            OverrideType::RegexMatch => 4.0,
            OverrideType::StartsWith => 3.0,
            OverrideType::EndsWith => 3.0,
            OverrideType::Contains => 2.0,
            OverrideType::FuzzyMatch => 1.0,
            OverrideType::WordBoundary => 3.5,
        };

        // Condition specificity
        score += override_rule.conditions.len() as f64 * 0.5;

        // Position constraint specificity
        if override_rule.position_constraints.is_some() {
            score += 1.0;
        }

        score
    }

    /// Check if conflicts can be combined
    fn can_combine_conflicts(&self, conflicts: &[OverrideConflict]) -> bool {
        // Simple heuristic: conflicts can be combined if they're all low severity
        conflicts.iter().all(|c| c.severity == ConflictSeverity::Low)
    }

    /// Analyze conflicts and provide recommendations
    pub fn analyze_conflicts(&self, conflicts: &[OverrideConflict]) -> ConflictAnalysis {
        let mut analysis = ConflictAnalysis {
            total_conflicts: conflicts.len(),
            critical_conflicts: 0,
            high_conflicts: 0,
            medium_conflicts: 0,
            low_conflicts: 0,
            resolvable_automatically: false,
            recommendations: Vec::new(),
        };

        for conflict in conflicts {
            match conflict.severity {
                ConflictSeverity::Critical => analysis.critical_conflicts += 1,
                ConflictSeverity::High => analysis.high_conflicts += 1,
                ConflictSeverity::Medium => analysis.medium_conflicts += 1,
                ConflictSeverity::Low => analysis.low_conflicts += 1,
            }
        }

        // Determine if conflicts can be resolved automatically
        analysis.resolvable_automatically = analysis.critical_conflicts == 0 &&
            matches!(self.resolution_strategy, 
                ConflictResolutionStrategy::HighestPriority |
                ConflictResolutionStrategy::MostRecent |
                ConflictResolutionStrategy::MostSpecific);

        // Generate recommendations
        if analysis.critical_conflicts > 0 {
            analysis.recommendations.push(
                "Critical conflicts require immediate manual review".to_string()
            );
        }

        if analysis.high_conflicts > 0 {
            analysis.recommendations.push(
                "High severity conflicts should be reviewed and resolved".to_string()
            );
        }

        if !analysis.resolvable_automatically {
            analysis.recommendations.push(
                "Consider changing resolution strategy to enable automatic resolution".to_string()
            );
        }

        analysis
    }

    /// Get current resolution strategy
    pub fn get_strategy(&self) -> &ConflictResolutionStrategy {
        &self.resolution_strategy
    }

    /// Get maximum conflicts reported
    pub fn get_max_conflicts_reported(&self) -> usize {
        self.max_conflicts_reported
    }
}

/// Analysis of conflicts
#[derive(Debug, Clone)]
pub struct ConflictAnalysis {
    /// Total number of conflicts
    pub total_conflicts: usize,
    /// Number of critical conflicts
    pub critical_conflicts: usize,
    /// Number of high severity conflicts
    pub high_conflicts: usize,
    /// Number of medium severity conflicts
    pub medium_conflicts: usize,
    /// Number of low severity conflicts
    pub low_conflicts: usize,
    /// Whether conflicts can be resolved automatically
    pub resolvable_automatically: bool,
    /// Recommendations for resolving conflicts
    pub recommendations: Vec<String>,
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}
