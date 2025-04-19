// src/utils/constants/casting.rs

use crate::utils::targeting::requirements::TargetingRequirement;

#[derive(Debug, Clone)]
pub struct CastingRequirement {
    pub is_sorcery_speed: bool,  // If true, can only be cast at sorcery speed (main phase, empty stack)
    pub target_requirements: Vec<TargetingRequirement>,
}