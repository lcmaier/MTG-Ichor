// game/gamestate/event_handlers/state_based_actions.rs

use crate::{game::gamestate::Game, utils::constants::events::GameEvent};
use crate::utils::constants::card_types::CardType;
use crate::utils::constants::events::DeathReason;
use crate::utils::constants::events::EventHandler;

impl Game {
    // Handle checking state-based actions
    pub fn handle_check_state_based_actions(&mut self) -> Result<(), String> {
        // Proceed all state-based action checks (will be filled in as needed)

        // 704.5a If a player has 0 or less life, that player loses the game.
        for player_id in 0..self.players.len() {
            let player = match self.get_player_ref(player_id) {
                Ok(p) => p,
                Err(_) => continue,
            };

            if player.life_total <= 0 {
                println!("Player {} has {} life and loses the game.", player_id, player.life_total);
                // Handle game loss - to be implemented
                // probably need to call a function because there are game loss prevention effects that might need to be handled
            }
        }

        // 704.5b If a player attempted to draw a card from a library with no cards in it since the last time state-based actions were checked, that player loses the game.

        // 704.5c If a player has ten or more poison counters, that player loses the game. Ignore this rule in Two-Headed Giant games; see rule 704.6b instead.

        // 704.5d If a token is in a zone other than the battlefield, it ceases to exist.

        // 704.5e If a copy of a spell is in a zone other than the stack, it ceases to exist. If a copy of a card is in any zone other than the stack or the battlefield, it ceases to exist.

        // 704.5f If a creature has toughness 0 or less, it’s put into its owner’s graveyard. Regeneration can’t replace this event.
        let mut zero_toughness_creature_events = Vec::new();

        for permanent in &self.battlefield {
            if permanent.has_card_type(&CardType::Creature) {
                if let Some(toughness) = permanent.characteristics.toughness {
                    if toughness <= 0 {
                        // create an event for this creature and push it to the vector
                        let zero_toughness_event = GameEvent::CreatureZeroToughness { creature_id: permanent.id };
                        zero_toughness_creature_events.push(zero_toughness_event);
                    }
                }
            }
        }

        // Process all zero toughness events
        for zero_toughness_event in zero_toughness_creature_events {
            self.handle_event(&zero_toughness_event)?;
        }

        // 704.5g If a creature has toughness greater than 0, it has damage marked on it, and the total damage marked on it is greater than or equal to its toughness, that creature has been dealt lethal damage and is destroyed. Regeneration can replace this event.
        let mut lethal_damage_destruction_events = Vec::new();

        for permanent in &self.battlefield {
            if permanent.has_card_type(&CardType::Creature) {
                // Check if the creature has lethal damage
                if let (Some(damageable), Some(toughness)) = (
                    &permanent.state.damageable,
                    permanent.characteristics.toughness
                ) {
                    // Only check creatures with toughness >0
                    if toughness > 0 && damageable.damage_marked >= toughness as u32 {
                        // Create a destruction event for this creature and add it to the vector
                        let destroy_event = GameEvent::PermanentDestroyed { permanent_id: permanent.id, reason: DeathReason::LethalDamage };
                        lethal_damage_destruction_events.push(destroy_event);
                    }
                }
            }
        }
        // Process all lethal damage events
        for destroy_event in lethal_damage_destruction_events {
            self.handle_event(&destroy_event)?;
        }

        // 704.5h If a creature has toughness greater than 0, and it’s been dealt damage by a source with deathtouch since the last time state-based actions were checked, that creature is destroyed. Regeneration can replace this event.

        // 704.5i If a planeswalker has loyalty 0, it’s put into its owner’s graveyard.

        // 704.5j If two or more legendary permanents with the same name are controlled by the same player, that player chooses one of them, and the rest are put into their owners’ graveyards. This is called the “legend rule.”

        // 704.5k If two or more permanents have the supertype world, all except the one that has had the world supertype for the shortest amount of time are put into their owners’ graveyards. In the event of a tie for the shortest amount of time, all are put into their owners’ graveyards. This is called the “world rule.”

        // 704.5m If an Aura is attached to an illegal object or player, or is not attached to an object or player, that Aura is put into its owner’s graveyard.

        // 704.5n If an Equipment or Fortification is attached to an illegal permanent or to a player, it becomes unattached from that permanent or player. It remains on the battlefield.

        // 704.5p If a battle or creature is attached to an object or player, it becomes unattached and remains on the battlefield. Similarly, if any nonbattle, noncreature permanent that’s neither an Aura, an Equipment, nor a Fortification is attached to an object or player, it becomes unattached and remains on the battlefield.

        // 704.5q If a permanent has both a +1/+1 counter and a -1/-1 counter on it, N +1/+1 and N -1/-1 counters are removed from it, where N is the smaller of the number of +1/+1 and -1/-1 counters on it.

        // 704.5r If a permanent with an ability that says it can’t have more than N counters of a certain kind on it has more than N counters of that kind on it, all but N of those counters are removed from it.

        // 704.5s If the number of lore counters on a Saga permanent is greater than or equal to its final chapter number and it isn’t the source of a chapter ability that has triggered but not yet left the stack, that Saga’s controller sacrifices it. See rule 714, “Saga Cards.”

        // 704.5t If a player’s venture marker is on the bottommost room of a dungeon card, and that dungeon card isn’t the source of a room ability that has triggered but not yet left the stack, the dungeon card’s owner removes it from the game. See rule 309, “Dungeons.”

        // 704.5u If a permanent with space sculptor and any creatures without a sector designation are on the battlefield, each player who controls one or more of those creatures and doesn’t control a permanent with space sculptor chooses a sector designation for each of those creatures they control. Then, each other player who controls one or more of those creatures chooses a sector designation for each of those creatures they control. See rule 702.158, “Space Sculptor.”

        // 704.5v If a battle has defense 0 and it isn’t the source of an ability that has triggered but not yet left the stack, it’s put into its owner’s graveyard.

        // 704.5w If a battle has no player in the game designated as its protector and no attacking creatures are currently attacking that battle, that battle’s controller chooses an appropriate player to be its protector based on its battle type. If no player can be chosen this way, the battle is put into its owner’s graveyard. See rule 310, “Battles.”

        // 704.5x If a Siege’s controller is also its designated protector, that player chooses an opponent to become its protector. If no player can be chosen this way, the battle is put into its owner’s graveyard. See rule 310, “Battles.”

        // 704.5y If a permanent has more than one Role controlled by the same player attached to it, each of those Roles except the one with the most recent timestamp is put into its owner’s graveyard.

        // 704.5z If a player controls a permanent with start your engines! and that player has no speed, that player’s speed becomes 1. See rule 702.179, “Start Your Engines!”

        Ok(())

    }
}