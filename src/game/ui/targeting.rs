// src/game/ui/targeting.rs

use crate::{game::gamestate::Game, utils::targeting::{core::{TargetCategory, TargetCriteria, TargetRef}, requirements::TargetingRequirement}};

pub fn prompt_for_targets(
    game: &Game,
    requirements: &Vec<TargetingRequirement>
) -> Result<Vec<TargetRef>, String> {
    let mut targets = Vec::new();

    // For each requirement, get user input
    for req in requirements {
        match &req.criteria {
            TargetCriteria::Category(TargetCategory::AnyDamageable) => {
                todo!()
            },
            _ => return Err("Unsupported targeting requirement".to_string())
        }
    }

    Ok(targets)
}