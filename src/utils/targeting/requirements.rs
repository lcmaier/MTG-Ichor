use std::collections::HashMap;

use crate::{game::gamestate::Game, utils::constants::id_types::PlayerId};

// src/utils/targeting/requirements.rs
use super::core::*;

#[derive(Debug, Clone, PartialEq)]
pub struct TargetingRequirement {
    pub criteria: TargetCriteria,
    pub min_targets: u8,
    pub max_targets: u8,
    pub id: u8, // used to link target definitions to player choices of that target
}

// A set of requirements for a spell or ability
#[derive(Debug, Clone)]
pub struct TargetRequirementSet {
    pub requirements: Vec<TargetingRequirement>,
    pub target_map: HashMap<u8, Vec<TargetRef>>, // Maps requirement IDs to chosen targets to ensure consistency and correctness when handling
}

impl TargetingRequirement {
    // single target requirement (most common requirement)
    pub fn single(criteria: TargetCriteria, id: u8) -> Self {
        TargetingRequirement {
            criteria,
            min_targets: 1,
            max_targets: 1,
            id
        }
    }

    pub fn any_target(id: u8) -> Self { // e.g. Lightning Bolt
        TargetingRequirement::single(
            TargetCriteria::Category(TargetCategory::AnyDamageable), id
        )
    }

    // Validate if a set of targets fulfills this TargetingRequirement
    pub fn validate_targets(&self, game: &Game, targets: &[TargetRef], controller_id: PlayerId) -> bool {
        // ensure the number of targets is within this requirement's bounds
        if targets.len() < self.min_targets as usize || targets.len() > self.max_targets as usize {
            return false;
        }

        // Check each target against critieria (see validation.rs in this folder)
        for target in targets {
            if !self.criteria.is_satisfied_by(game, target, controller_id) {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct TargetSet {
    pub requirements: Vec<TargetingRequirement>,
    pub target_map: HashMap<u8, Vec<TargetRef>>, // Maps requirement IDs to chosen targets to ensure consistency and correctness when handling
}

impl TargetSet {
    pub fn new(requirements: Vec<TargetingRequirement>) -> Self {
        TargetSet {
            requirements,
            target_map: HashMap::new(),
        }
    }
    

    pub fn add_requirement(&mut self, requirement: TargetingRequirement) {
        self.requirements.push(requirement);
    }

    // Add a target for a specific requirement
    pub fn add_target(&mut self, requirement_id: u8, target: TargetRef) -> Result<(), String> {
        // Find the requirement in the requirements
        let requirement = self.requirements.iter()
            .find(|req| req.id == requirement_id)
            .ok_or_else(|| format!("No requirement with ID {}", requirement_id))?;

        // get the current targets for this targeting requirement
        let targets = self.target_map.entry(requirement_id).or_insert(Vec::new());

        // Check if we've already reached the maximum number of targets for this requirement
        if targets.len() >= requirement.max_targets as usize {
            return Err(format!("Requirement with ID {} already has maximum target(s): {}", requirement_id, requirement.max_targets));
        }

        // Add the target to this requirement's targets and return
        targets.push(target);
        Ok(())
    }

    // Validate all targets against their requirements\
    pub fn validate(&self, game: &Game, controller_id: PlayerId) -> bool  {
        // Validate each requirement's targets
        for req in &self.requirements {
            let targets = match self.target_map.get(&req.id) {
                Some(targets) => targets,
                None => return false, // all targeting requirements definitionally must have at least 1 target
            };

            if !req.validate_targets(game, targets, controller_id) {
                return false;
            }
        }
        true
    }

    // Get all targets as a flattened vector (for passing to effects)
    pub fn all_targets(&self) -> Vec<TargetRef> {
        let mut all_targets = Vec::new();
        for targets in self.target_map.values() {
            all_targets.extend(targets.clone());
        }
        all_targets
    }
}
