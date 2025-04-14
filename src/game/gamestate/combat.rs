// src/game/gamestate/combat.rs
use crate::game::gamestate::Game;
use crate::utils::constants::combat::{AttackingCreature, AttackTarget, BlockingCreature};
use crate::utils::constants::id_types::ObjectId;
// We're dealing with combat LATER, this is currently incomplete
impl Game {
    // find or create a BlockingCreature object based on a creature_id
    fn get_or_create_blocker(&mut self, creature_id: ObjectId) -> &mut BlockingCreature {

        // If we can find the blocker, return it
        if let Some(index) = self.blocking_creatures.iter().position(|b| b.creature_id == creature_id) {
            &mut self.blocking_creatures[index]
        } else {
            // Otherwise, create a new blocker with default values
            self.blocking_creatures.push(BlockingCreature {
                creature_id,
                blocking: Vec::new(),
                max_can_block: 1, // Default restriction - can only block one creature
            });
            self.blocking_creatures.last_mut().unwrap()
        }
    }
}