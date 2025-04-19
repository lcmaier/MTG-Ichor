// src/utils/targeting/validation.rs

use crate::{game::gamestate::Game, utils::constants::{card_types::CardType, id_types::PlayerId}};

use super::core::*;

// Implementation of the main validation function
impl TargetCriteria {
    // Takes a target and game context and determines if the target satisfies this criteria
    // Game is a MTG Game, target is a TargetRef, and controller_id is the PlayerId of the spell/ability that's doing this targeting with this TargetCriteria
    pub fn is_satisfied_by(&self, game: &Game, target: &TargetRef, controller_id: PlayerId) -> bool {
        match self {  
            TargetCriteria::CardType(card_type, zone) => {
                    // if we match a CardType, the target is an object, not a player
                    if let TargetRefId::Object(obj_id) = &target.ref_id {
                        match zone {
                            TargetZone::Battlefield => {
                                if let Some(permanent) = game.battlefield.iter().find(|obj| &obj.id == obj_id) {
                                    permanent.has_card_type(card_type)
                                } else {
                                    false
                                }
                            },
                            TargetZone::Stack => {
                                if let Some(spell) = game.stack.iter().find(|obj| &obj.id == obj_id) {
                                    spell.has_card_type(card_type)
                                } else {
                                    false
                                }
                            },
                            TargetZone::Exile => {
                                if let Some(card) = game.exile.iter().find(|obj| &obj.id == obj_id) {
                                    card.has_card_type(card_type)
                                } else {
                                    false
                                }
                            },
                            _ => false // we'll deal with graveyards later since those are attached to Player structs, makes it more complicated
                        }
                    } else {
                        false // Players don't have card types
                    }
            },

            // Non-card type targets (targets for damage are here)
            TargetCriteria::Category(category) => {
                match category {
                    TargetCategory::Player => {
                        target.is_player()
                    },
                    TargetCategory::Opponent => {
                        if let Some(pid) = target.get_player_id() {
                            // opponents are defined as "the player(s) not casting this spell/activating this ability"
                            pid != controller_id
                        } else {
                            false
                        }
                    },
                    TargetCategory::AnyDamageable => {
                        // Lightning Bolt-style "any target"
                        match &target.ref_id {
                            TargetRefId::Player(_) => true, // Players can be damaged (by default)
                            TargetRefId::Object(obj_id) => {
                                // ensure it's a permanent (i.e. on the battlefield) that's a creature, planeswalker, or battle
                                game.battlefield.iter()
                                    .find(|obj| &obj.id == obj_id)
                                    .map(|obj| {
                                        obj.has_card_type(&CardType::Creature) ||
                                        obj.has_card_type(&CardType::Planeswalker) ||
                                        obj.has_card_type(&CardType::Battle)
                                    })
                                    .unwrap_or(false)
                            }
                        }
                    },
                    TargetCategory::Permanent => {
                        // to satisfy a Permanent category, the target must be on the battlefield
                        if let TargetRefId::Object(obj_id) = &target.ref_id {
                            game.battlefield.iter()
                                .any(|obj| &obj.id == obj_id)
                        } else {
                            false
                        }
                    },
                }
            },

            // for And criteria, all criteria in the vector must be satisfied
            TargetCriteria::And(criteria) => {
                criteria.iter().all(|c| c.is_satisfied_by(game, target, controller_id))
            },

            TargetCriteria::Or(criteria) => {
                criteria.iter().any(|c| c.is_satisfied_by(game, target, controller_id))
            },

            TargetCriteria::Not(criteria) => {
                !criteria.is_satisfied_by(game, target, controller_id)
            },

            // Placeholder for other TargetCriteria implementations
            _ => false,
        }
    }
}