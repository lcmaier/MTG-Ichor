use crate::engine::resolve::ResolvedTarget;
use crate::oracle::characteristics::get_effective_power;
use crate::state::game_state::GameState;
use crate::types::card_types::CardType;
use crate::types::effects::{PermanentFilter, EffectRecipient, SelectionFilter, TargetCount};
use crate::types::ids::ObjectId;

impl GameState {
    /// Validate that chosen targets are legal for the given EffectRecipient.
    ///
    /// Called at cast/activation time (rule 601.2c) and again at resolution
    /// time (rule 608.2b) to check if targets are still legal.
    pub fn validate_targets(
        &self,
        recipient: &EffectRecipient,
        targets: &[ResolvedTarget],
    ) -> Result<(), String> {
        match recipient {
            EffectRecipient::Implicit => {
                if !targets.is_empty() {
                    return Err("Spell has no targets but targets were provided".to_string());
                }
                Ok(())
            }

            EffectRecipient::Controller => {
                // "You" doesn't use the targets list — the controller is implicit
                Ok(())
            }

            // Target and Choose validate identically for now.
            // T22 will add hexproof/shroud/protection checks for Target only.
            EffectRecipient::Target(filter, count)
            | EffectRecipient::Choose(filter, count) => {
                self.validate_target_count(count, targets.len())?;
                for t in targets {
                    self.validate_selection(filter, t)?;
                }
                Ok(())
            }
        }
    }

    /// Check that the number of targets matches the TargetCount spec.
    fn validate_target_count(
        &self,
        count: &TargetCount,
        actual: usize,
    ) -> Result<(), String> {
        match count {
            TargetCount::Exactly(n) => {
                if actual != *n as usize {
                    return Err(format!(
                        "Expected exactly {} target(s), got {}", n, actual
                    ));
                }
            }
            TargetCount::UpTo(n) => {
                if actual > *n as usize {
                    return Err(format!(
                        "Expected up to {} target(s), got {}", n, actual
                    ));
                }
            }
        }
        Ok(())
    }

    /// Validate a single selected object/player against a SelectionFilter.
    pub(crate) fn validate_selection(
        &self,
        filter: &SelectionFilter,
        target: &ResolvedTarget,
    ) -> Result<(), String> {
        match filter {
            SelectionFilter::Creature => self.validate_creature_target(target),
            SelectionFilter::Player => self.validate_player_target(target),
            SelectionFilter::Any => self.validate_any_target(target),
            SelectionFilter::Permanent(pf) => self.validate_permanent_target(target, pf),
            SelectionFilter::Spell => self.validate_spell_target(target),
        }
    }

    /// Validate a target is a creature on the battlefield.
    fn validate_creature_target(&self, target: &ResolvedTarget) -> Result<(), String> {
        match target {
            ResolvedTarget::Object(id) => {
                self.require_on_battlefield(*id)?;
                let obj = self.get_object(*id)?;
                if !obj.card_data.types.contains(&CardType::Creature) {
                    return Err(format!("Target {} is not a creature", id));
                }
                Ok(())
            }
            ResolvedTarget::Player(_) => {
                Err("Expected a creature target, got a player".to_string())
            }
        }
    }

    /// Validate a target is a valid player.
    fn validate_player_target(&self, target: &ResolvedTarget) -> Result<(), String> {
        match target {
            ResolvedTarget::Player(pid) => {
                if *pid >= self.players.len() {
                    return Err(format!("Player {} does not exist", pid));
                }
                Ok(())
            }
            ResolvedTarget::Object(_) => {
                Err("Expected a player target, got an object".to_string())
            }
        }
    }

    /// Validate "any target" — creature or planeswalker on battlefield, or player.
    fn validate_any_target(&self, target: &ResolvedTarget) -> Result<(), String> {
        match target {
            ResolvedTarget::Player(pid) => {
                if *pid >= self.players.len() {
                    return Err(format!("Player {} does not exist", pid));
                }
                Ok(())
            }
            ResolvedTarget::Object(id) => {
                self.require_on_battlefield(*id)?;
                let obj = self.get_object(*id)?;
                if obj.card_data.types.contains(&CardType::Creature)
                    || obj.card_data.types.contains(&CardType::Planeswalker)
                {
                    Ok(())
                } else {
                    Err(format!(
                        "Target {} is not a creature or planeswalker", id
                    ))
                }
            }
        }
    }

    /// Validate a target is a permanent on the battlefield matching the filter.
    fn validate_permanent_target(
        &self,
        target: &ResolvedTarget,
        filter: &PermanentFilter,
    ) -> Result<(), String> {
        match target {
            ResolvedTarget::Object(id) => {
                self.require_on_battlefield(*id)?;
                if !self.permanent_matches_filter(*id, filter)? {
                    return Err(format!(
                        "Target {} does not match permanent filter {:?}", id, filter
                    ));
                }
                Ok(())
            }
            ResolvedTarget::Player(_) => {
                Err("Expected a permanent target, got a player".to_string())
            }
        }
    }

    /// Validate a target is a spell on the stack.
    fn validate_spell_target(&self, target: &ResolvedTarget) -> Result<(), String> {
        match target {
            ResolvedTarget::Object(id) => {
                if !self.stack.contains(id) {
                    return Err(format!("Target {} is not on the stack", id));
                }
                Ok(())
            }
            ResolvedTarget::Player(_) => {
                Err("Expected a spell target, got a player".to_string())
            }
        }
    }

    /// Check whether an object is on the battlefield.
    fn require_on_battlefield(&self, id: ObjectId) -> Result<(), String> {
        if !self.battlefield.contains_key(&id) {
            return Err(format!("Object {} is not on the battlefield", id));
        }
        Ok(())
    }

    /// Check whether a permanent matches a PermanentFilter.
    fn permanent_matches_filter(
        &self,
        id: ObjectId,
        filter: &PermanentFilter,
    ) -> Result<bool, String> {
        let obj = self.get_object(id)?;
        match filter {
            PermanentFilter::All => Ok(true),
            PermanentFilter::ByType(card_type) => {
                Ok(obj.card_data.types.contains(card_type))
            }
            PermanentFilter::BySubtype(subtype) => {
                Ok(obj.card_data.subtypes.contains(subtype))
            }
            PermanentFilter::ByColor(color) => {
                Ok(obj.card_data.colors.contains(color))
            }
            PermanentFilter::ByController(player_ref) => {
                let entry = self.battlefield.get(&id)
                    .ok_or_else(|| format!("Object {} not on battlefield", id))?;
                match player_ref {
                    crate::types::effects::PlayerRef::Player(pid) => {
                        Ok(entry.controller == *pid)
                    }
                    // Other PlayerRef variants would need resolution context;
                    // for now just match Player explicitly
                    _ => Err(format!(
                        "PlayerRef {:?} not supported in permanent filter validation", player_ref
                    )),
                }
            }
            PermanentFilter::PowerLE(max_power) => {
                get_effective_power(self, id)
                    .map(|p| p <= *max_power)
                    .ok_or_else(|| format!("Object {} has no power", id))
            }
            PermanentFilter::And(a, b) => {
                let matches_a = self.permanent_matches_filter(id, a)?;
                let matches_b = self.permanent_matches_filter(id, b)?;
                Ok(matches_a && matches_b)
            }
            PermanentFilter::Not(inner) => {
                let matches = self.permanent_matches_filter(id, inner)?;
                Ok(!matches)
            }
        }
    }

    /// Re-validate targets at resolution time (rule 608.2b).
    /// Returns true if at least one target is still legal.
    /// Returns false if ALL targets are illegal (spell fizzles).
    pub fn any_targets_still_legal(
        &self,
        recipient: &EffectRecipient,
        targets: &[ResolvedTarget],
    ) -> bool {
        match recipient {
            // Choose effects don't target — they never fizzle.
            EffectRecipient::Implicit
            | EffectRecipient::Controller
            | EffectRecipient::Choose(_, _) => true,
            EffectRecipient::Target(_, _) => {
                targets.iter().any(|t| {
                    self.is_single_target_legal(recipient, t)
                })
            }
        }
    }

    /// Check whether there is at least one legal choice on the battlefield
    /// (or among players) for the given `SelectionFilter`.
    ///
    /// `exclude_id` is typically the Aura itself — it can't enchant itself.
    /// For player filters, all players are considered (hexproof/shroud will
    /// be added here when T22 lands).
    pub(crate) fn has_any_legal_choice(
        &self,
        filter: &SelectionFilter,
        exclude_id: Option<ObjectId>,
    ) -> bool {
        match filter {
            SelectionFilter::Player => {
                // TODO: filter by hexproof/shroud once T22 lands
                !self.players.is_empty()
            }
            _ => self.battlefield.keys()
                .filter(|&&id| Some(id) != exclude_id)
                .any(|&id| {
                    let candidate = ResolvedTarget::Object(id);
                    self.validate_selection(filter, &candidate).is_ok()
                }),
        }
    }

    /// Check if a single target is still legal for the given spec.
    /// Only meaningful for `Target` — `Choose` doesn't participate in fizzle.
    fn is_single_target_legal(
        &self,
        recipient: &EffectRecipient,
        target: &ResolvedTarget,
    ) -> bool {
        match recipient {
            EffectRecipient::Target(filter, _) => {
                self.validate_selection(filter, target).is_ok()
            }
            // Choose, Implicit, Controller — always "legal" (no fizzle).
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::BattlefieldEntity;
    use crate::types::card_types::{CardType, Supertype, Subtype, LandType};
    use crate::types::effects::{PermanentFilter, TargetCount};
    use crate::types::zones::Zone;

    fn setup_game_with_land() -> (GameState, ObjectId) {
        let mut game = GameState::new(2, 20);
        let land_data = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .subtype(Subtype::Land(LandType::Forest))
            .build();
        let obj = GameObject::new(land_data, 0, Zone::Battlefield);
        let id = obj.id;
        let ts = game.allocate_timestamp();
        game.add_object(obj);
        game.battlefield.insert(id, BattlefieldEntity::new(id, 0, ts, 1));
        (game, id)
    }

    #[test]
    fn test_validate_permanent_target_all() {
        let (game, land_id) = setup_game_with_land();
        let targets = vec![ResolvedTarget::Object(land_id)];
        let spec = EffectRecipient::Target(SelectionFilter::Permanent(PermanentFilter::All), TargetCount::Exactly(1));
        assert!(game.validate_targets(&spec, &targets).is_ok());
    }

    #[test]
    fn test_validate_permanent_target_by_type_land() {
        let (game, land_id) = setup_game_with_land();
        let targets = vec![ResolvedTarget::Object(land_id)];
        let spec = EffectRecipient::Target(SelectionFilter::Permanent(
            PermanentFilter::ByType(CardType::Land)),
            TargetCount::Exactly(1),
        );
        assert!(game.validate_targets(&spec, &targets).is_ok());
    }

    #[test]
    fn test_validate_permanent_target_wrong_type() {
        let (game, land_id) = setup_game_with_land();
        let targets = vec![ResolvedTarget::Object(land_id)];
        let spec = EffectRecipient::Target(SelectionFilter::Permanent(
            PermanentFilter::ByType(CardType::Creature)),
            TargetCount::Exactly(1),
        );
        assert!(game.validate_targets(&spec, &targets).is_err());
    }

    #[test]
    fn test_validate_player_target() {
        let game = GameState::new(2, 20);
        let targets = vec![ResolvedTarget::Player(1)];
        let spec = EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1));
        assert!(game.validate_targets(&spec, &targets).is_ok());
    }

    #[test]
    fn test_validate_player_target_invalid() {
        let game = GameState::new(2, 20);
        let targets = vec![ResolvedTarget::Player(5)];
        let spec = EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1));
        assert!(game.validate_targets(&spec, &targets).is_err());
    }

    #[test]
    fn test_validate_spell_target_not_on_stack() {
        let game = GameState::new(2, 20);
        let fake_id = crate::types::ids::new_object_id();
        let targets = vec![ResolvedTarget::Object(fake_id)];
        let spec = EffectRecipient::Target(SelectionFilter::Spell, TargetCount::Exactly(1));
        assert!(game.validate_targets(&spec, &targets).is_err());
    }

    #[test]
    fn test_validate_no_targets() {
        let game = GameState::new(2, 20);
        let spec = EffectRecipient::Implicit;
        assert!(game.validate_targets(&spec, &[]).is_ok());
        assert!(game.validate_targets(&spec, &[ResolvedTarget::Player(0)]).is_err());
    }

    #[test]
    fn test_validate_wrong_target_count() {
        let (game, land_id) = setup_game_with_land();
        let targets = vec![
            ResolvedTarget::Object(land_id),
            ResolvedTarget::Object(land_id),
        ];
        let spec = EffectRecipient::Target(SelectionFilter::Permanent(PermanentFilter::All), TargetCount::Exactly(1));
        assert!(game.validate_targets(&spec, &targets).is_err());
    }

    #[test]
    fn test_any_targets_still_legal_object_gone() {
        let (mut game, land_id) = setup_game_with_land();
        let targets = vec![ResolvedTarget::Object(land_id)];
        let spec = EffectRecipient::Target(SelectionFilter::Permanent(PermanentFilter::All), TargetCount::Exactly(1));

        // Target is legal while on battlefield
        assert!(game.any_targets_still_legal(&spec, &targets));

        // Remove from battlefield — target is no longer legal
        game.battlefield.remove(&land_id);
        assert!(!game.any_targets_still_legal(&spec, &targets));
    }
}
