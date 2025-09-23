// Modified: 2025-09-22

//! Conflict resolution for override rules
//!
//! This module provides functionality for resolving conflicts between
//! multiple matching override rules using various resolution strategies.

use super::types::*;
use crate::Result;
use tracing::{debug, warn};

/// Conflict resolver for handling overlapping override rules
#[derive(Debug, Clone)]
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

    /// Create a conflict resolver with specific strategy
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
    pub fn set_max_conflicts_reported(&mut self, max_conflicts: usize) {
        self.max_conflicts_reported = max_conflicts;
    }

    /// Resolve conflicts between multiple matching overrides
    pub fn resolve_conflicts(
        &self,
        matching_overrides: Vec<(MappingOverride, f64)>,
    ) -> Result<ConflictResolution> {
        if matching_overrides.is_empty() {
            return Ok(ConflictResolution {
                selected_override: None,
                alternatives: Vec::new(),
                conflicts: Vec::new(),
            });
        }

        if matching_overrides.len() == 1 {
            let (override_rule, _confidence) = matching_overrides.into_iter().next().unwrap();
            return Ok(ConflictResolution {
                selected_override: Some(override_rule),
                alternatives: Vec::new(),
                conflicts: Vec::new(),
            });
        }

        debug!("Resolving conflicts between {} matching overrides", matching_overrides.len());

        match self.resolution_strategy {
            ConflictResolutionStrategy::HighestPriority => {
                self.resolve_by_highest_priority(matching_overrides)
            }
            ConflictResolutionStrategy::MostRecent => {
                self.resolve_by_most_recent(matching_overrides)
            }
            ConflictResolutionStrategy::MostSpecific => {
                self.resolve_by_most_specific(matching_overrides)
            }
            ConflictResolutionStrategy::Combine => {
                self.resolve_by_combining(matching_overrides)
            }
            ConflictResolutionStrategy::ReportAndFallback => {
                self.resolve_with_fallback(matching_overrides)
            }
        }
    }

    /// Resolve conflicts by selecting the highest priority override
    fn resolve_by_highest_priority(
        &self,
        matching_overrides: Vec<(MappingOverride, f64)>,
    ) -> Result<ConflictResolution> {
        let mut sorted = matching_overrides;
        sorted.sort_by(|a, b| {
            // Sort by priority (descending), then by confidence (descending)
            b.0.priority.cmp(&a.0.priority)
                .then_with(|| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
        });

        let selected = sorted.first().map(|(o, _)| o.clone());
        let alternatives: Vec<MappingOverride> = sorted.into_iter().skip(1).map(|(o, _)| o).collect();

        // Check for priority ties
        let conflicts = self.detect_priority_conflicts(&alternatives, &selected);

        Ok(ConflictResolution {
            selected_override: selected,
            alternatives,
            conflicts,
        })
    }

    /// Resolve conflicts by selecting the most recently created override
    fn resolve_by_most_recent(
        &self,
        matching_overrides: Vec<(MappingOverride, f64)>,
    ) -> Result<ConflictResolution> {
        let mut sorted = matching_overrides;
        sorted.sort_by(|a, b| {
            // Sort by creation time (descending), then by priority (descending)
            b.0.created_at.cmp(&a.0.created_at)
                .then_with(|| b.0.priority.cmp(&a.0.priority))
        });

        let selected = sorted.first().map(|(o, _)| o.clone());
        let alternatives = sorted.into_iter().skip(1).map(|(o, _)| o).collect();

        Ok(ConflictResolution {
            selected_override: selected,
            alternatives,
            conflicts: Vec::new(),
        })
    }

    /// Resolve conflicts by selecting the most specific override
    fn resolve_by_most_specific(
        &self,
        matching_overrides: Vec<(MappingOverride, f64)>,
    ) -> Result<ConflictResolution> {
        let mut scored_overrides: Vec<_> = matching_overrides
            .into_iter()
            .map(|(override_rule, confidence)| {
                let specificity_score = self.calculate_specificity_score(&override_rule);
                (override_rule, confidence, specificity_score)
            })
            .collect();

        // Sort by specificity (descending), then by priority (descending)
        scored_overrides.sort_by(|a, b| {
            b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.0.priority.cmp(&a.0.priority))
        });

        let selected = scored_overrides.first().map(|(o, _, _)| o.clone());
        let alternatives = scored_overrides.into_iter().skip(1).map(|(o, _, _)| o).collect();

        Ok(ConflictResolution {
            selected_override: selected,
            alternatives,
            conflicts: Vec::new(),
        })
    }

    /// Attempt to combine multiple overrides (not implemented)
    fn resolve_by_combining(
        &self,
        matching_overrides: Vec<(MappingOverride, f64)>,
    ) -> Result<ConflictResolution> {
        warn!("Combine resolution strategy not implemented, falling back to highest priority");
        self.resolve_by_highest_priority(matching_overrides)
    }

    /// Report conflicts and use fallback strategy
    fn resolve_with_fallback(
        &self,
        matching_overrides: Vec<(MappingOverride, f64)>,
    ) -> Result<ConflictResolution> {
        let conflicts = self.generate_conflict_reports(&matching_overrides);
        
        // Use highest priority as fallback
        let mut resolution = self.resolve_by_highest_priority(matching_overrides)?;
        resolution.conflicts.extend(conflicts);

        Ok(resolution)
    }

    /// Calculate specificity score for an override
    fn calculate_specificity_score(&self, override_rule: &MappingOverride) -> f64 {
        let mut score = 0.0;

        // Scope specificity
        score += match &override_rule.scope {
            OverrideScope::Global => 1.0,
            OverrideScope::DocumentType(_) => 2.0,
            OverrideScope::Organization(_) => 3.0,
            OverrideScope::Project(_) => 4.0,
            OverrideScope::Session(_) => 5.0,
            OverrideScope::User(_) => 6.0,
        };

        // Pattern specificity
        score += match override_rule.rule_type {
            OverrideType::ExactMatch => 6.0,
            OverrideType::RegexPattern => 5.0,
            OverrideType::PositionalMatch => 4.0,
            OverrideType::ContainsMatch => 3.0,
            OverrideType::PrefixSuffixMatch => 2.0,
            OverrideType::FuzzyMatch => 1.0,
            OverrideType::ConditionalMatch => 5.0,
        };

        // Condition specificity
        score += override_rule.conditions.len() as f64 * 0.5;

        // Pattern complexity
        if override_rule.pattern.case_sensitive {
            score += 0.5;
        }
        if override_rule.pattern.whole_word {
            score += 0.5;
        }

        score
    }

    /// Detect priority conflicts between overrides
    fn detect_priority_conflicts(
        &self,
        alternatives: &[MappingOverride],
        selected: &Option<MappingOverride>,
    ) -> Vec<OverrideConflict> {
        let mut conflicts = Vec::new();

        if let Some(ref selected_override) = selected {
            for alternative in alternatives {
                if alternative.priority == selected_override.priority {
                    conflicts.push(OverrideConflict {
                        conflicting_overrides: vec![selected_override.id, alternative.id],
                        conflict_type: ConflictType::PriorityTie,
                        severity: ConflictSeverity::Medium,
                        description: format!(
                            "Overrides '{}' and '{}' have the same priority ({})",
                            selected_override.name, alternative.name, selected_override.priority
                        ),
                        suggested_resolution: Some("Adjust priority levels to resolve conflict".to_string()),
                        resolution_applied: Some(ConflictResolutionStrategy::HighestPriority),
                    });
                }
            }
        }

        conflicts
    }

    /// Generate conflict reports for all matching overrides
    fn generate_conflict_reports(
        &self,
        matching_overrides: &[(MappingOverride, f64)],
    ) -> Vec<OverrideConflict> {
        let mut conflicts = Vec::new();

        for i in 0..matching_overrides.len() {
            for j in (i + 1)..matching_overrides.len() {
                let (override1, _) = &matching_overrides[i];
                let (override2, _) = &matching_overrides[j];

                // Check for various conflict types
                if override1.scope == override2.scope {
                    conflicts.push(OverrideConflict {
                        conflicting_overrides: vec![override1.id, override2.id],
                        conflict_type: ConflictType::ScopeOverlap,
                        severity: ConflictSeverity::Low,
                        description: format!(
                            "Overrides '{}' and '{}' have overlapping scopes",
                            override1.name, override2.name
                        ),
                        suggested_resolution: Some("Consider narrowing scope or adjusting priorities".to_string()),
                        resolution_applied: None,
                    });
                }

                if override1.priority == override2.priority {
                    conflicts.push(OverrideConflict {
                        conflicting_overrides: vec![override1.id, override2.id],
                        conflict_type: ConflictType::PriorityTie,
                        severity: ConflictSeverity::Medium,
                        description: format!(
                            "Overrides '{}' and '{}' have the same priority ({})",
                            override1.name, override2.name, override1.priority
                        ),
                        suggested_resolution: Some("Adjust priority levels to resolve conflict".to_string()),
                        resolution_applied: None,
                    });
                }
            }
        }

        // Limit the number of conflicts reported
        conflicts.truncate(self.max_conflicts_reported);
        conflicts
    }

    /// Get the current resolution strategy
    pub fn get_strategy(&self) -> &ConflictResolutionStrategy {
        &self.resolution_strategy
    }

    /// Get the maximum number of conflicts reported
    pub fn get_max_conflicts_reported(&self) -> usize {
        self.max_conflicts_reported
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;

    fn create_test_override(name: &str, priority: i32) -> MappingOverride {
        MappingOverride {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: format!("Test override {}", name),
            rule_type: OverrideType::ExactMatch,
            pattern: OverridePattern {
                pattern: name.to_string(),
                case_sensitive: true,
                whole_word: false,
                regex_flags: None,
                fuzzy_threshold: None,
                position_constraints: None,
            },
            target_field: "test_field".to_string(),
            priority,
            conditions: Vec::new(),
            scope: OverrideScope::Global,
            created_by: "test_user".to_string(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
            active: true,
            version: 1,
            tags: Vec::new(),
        }
    }

    #[test]
    fn test_highest_priority_resolution() {
        let resolver = ConflictResolver::new();
        let overrides = vec![
            (create_test_override("low", 1), 0.8),
            (create_test_override("high", 10), 0.9),
            (create_test_override("medium", 5), 0.7),
        ];

        let result = resolver.resolve_conflicts(overrides).unwrap();
        assert!(result.selected_override.is_some());
        assert_eq!(result.selected_override.unwrap().name, "high");
        assert_eq!(result.alternatives.len(), 2);
    }

    #[test]
    fn test_single_override_no_conflict() {
        let resolver = ConflictResolver::new();
        let overrides = vec![(create_test_override("single", 5), 0.8)];

        let result = resolver.resolve_conflicts(overrides).unwrap();
        assert!(result.selected_override.is_some());
        assert_eq!(result.alternatives.len(), 0);
        assert_eq!(result.conflicts.len(), 0);
    }

    #[test]
    fn test_empty_overrides() {
        let resolver = ConflictResolver::new();
        let overrides = vec![];

        let result = resolver.resolve_conflicts(overrides).unwrap();
        assert!(result.selected_override.is_none());
        assert_eq!(result.alternatives.len(), 0);
        assert_eq!(result.conflicts.len(), 0);
    }
}
