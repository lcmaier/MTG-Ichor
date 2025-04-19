// src/utils/targeting/requirements.rs
use super::core::*;

#[derive(Debug, Clone, PartialEq)]
pub struct TargetingRequirement {
    pub criteria: TargetCriteria,
    pub min_targets: u8,
    pub max_targets: u8,
}

impl TargetingRequirement {
    // single target requirement (most common requirement)
    pub fn single(criteria: TargetCriteria) -> Self {
        TargetingRequirement {
            criteria,
            min_targets: 1,
            max_targets: 1,
        }
    }

    pub fn any_target() -> Self { // e.g. Lightning Bolt
        TargetingRequirement::single(
            TargetCriteria::Category(TargetCategory::AnyDamageable)
        )
    }
}

#[derive(Debug, Clone)]
pub struct TargetSet {
    pub requirements: Vec<TargetingRequirement>,
    pub targets: Vec<TargetRef>,
}

impl TargetSet {
    pub fn new(requirements: Vec<TargetingRequirement>) -> Self {
        TargetSet {
            requirements,
            targets: Vec::new(),
        }
    }
    
    // Implementation of add_target, is_complete, is_valid, etc.
}
