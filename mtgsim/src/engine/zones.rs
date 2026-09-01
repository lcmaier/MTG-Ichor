use crate::engine::actions::{ActionContext, ZoneChangeCause};
use crate::events::event::GameEvent;
use crate::state::game_state::GameState;
use crate::types::card_types::CardType;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::zones::Zone;

/// Centralized zone transition logic.
///
/// ALL object movement between zones goes through this module.
/// This is the single place where:
/// - Objects are removed from their old zone's collection
/// - Objects are added to their new zone's collection
/// - Zone-specific state is initialized or cleaned up
///
/// This replaces v1's scattered `to_battlefield()`, `to_graveyard()`, etc. methods.

impl GameState {
    /// Move a game object from one zone to another.
    ///
    /// This is the fundamental zone transition operation. All higher-level operations
    /// (draw, play land, cast spell, destroy, etc.) ultimately call this.
    ///
    /// **Do not call this directly from engine modules** — use
    /// [`GameState::change_zone`] or
    /// [`GameState::execute_action`] with [`GameAction::ZoneChange`]. Both
    /// route through the replacement-effects chokepoint (CR 614). This function
    /// is `pub(crate)` so internal helpers (`draw_card`, `play_land`, the
    /// `GameAction::ZoneChange` arm itself, and existing unit tests) can still
    /// call it.
    ///
    /// **One documented class of exception, and it is permanent:**
    /// `// CAST-ROLLBACK:` — `cast_spell`'s failure paths, via
    /// `rollback_cast_to_hand`. These are **not** zone changes at all: CR 601.2
    /// rewinds the casting process, so no object legally moved and nothing may
    /// observe it. They must never be routed through the chokepoint.
    ///
    /// The `// REPLACEMENT-BYPASS:` class is gone. Its three sites in
    /// `engine/stack.rs` *were* real zone changes the pipeline had to see; they
    /// bypassed only because `resolve_top_of_stack` used to pop the object off
    /// the stack `Vec` before resolution began, so this function would have
    /// removed it twice. RA-3 closed them by naming the in-between state instead
    /// of routing around it; RC-1 then deleted the pop, so the stack removal
    /// here is now the only one and finds the object where CR 608.2 says it is.
    ///
    /// **This function performs the move and announces nothing.** The
    /// `GameEvent::ZoneChange` is emitted by `perform_action`'s own arm, which
    /// is the only place that knows the [`ZoneChangeCause`] and the only place
    /// that can capture the CR 603.10a LKI frame before the object stops being
    /// a permanent. Splitting it that way is also what finally makes the
    /// `// CAST-ROLLBACK:` tag true: a rewind now really is unobservable,
    /// where before it still pushed a Stack→Hand event into the log.
    pub(crate) fn move_object(&mut self, id: ObjectId, to: Zone) -> Result<(), String> {
        let from = {
            let obj = self.get_object(id)?;
            obj.zone
        };

        if from == to {
            return Ok(()); // no-op
        }

        // Clean up zone-specific state for the old zone (before removal,
        // so we can still read the departing entity's state)
        self.cleanup_zone_state(id, from);

        // Remove from old zone's collection
        self.remove_from_zone_collection(id, from)?;

        // Add to new zone's collection
        self.add_to_zone_collection(id, to)?;

        // Initialize zone-specific state for the new zone
        self.init_zone_state(id, to)?;

        // Update the object's zone field, and stamp *when* it moved.
        //
        // Here rather than in `perform_action` because this is the performer:
        // the epoch is a property of the move, and the CR 601.2 cast rollback
        // is a move too (it just is not an event). CR 704.6d reads the stamp;
        // CR 400.7 will read the same one.
        let epoch = self.next_zone_change_epoch;
        self.next_zone_change_epoch += 1;
        let obj = self.get_object_mut(id)?;
        obj.zone = to;
        obj.zone_change_epoch = epoch;

        Ok(())
    }

    /// Draw a card: move top of library to hand.
    ///
    /// Returns `Ok(Some(id))` if a card was drawn, `Ok(None)` if the library
    /// was empty (the player is flagged for SBA 704.5b loss but the game
    /// continues — SBAs will handle the actual loss when checked).
    /// **This is the performer, not the entry point.** Callers propose a draw
    /// with `execute_action(GameAction::DrawCard { .. })` so CR 614.11 draw
    /// replacements (Phase RE) see it; this runs after the pipeline has decided
    /// the draw happens.
    ///
    /// The empty-library case is deliberately handled *here* rather than at the
    /// proposal: CR 121.6a says a draw-replacement applies even when there are
    /// no cards to draw, so the proposal must still reach the pipeline. What
    /// does not happen is the draw itself — no `CardDrawn`, just the CR 704.5b
    /// flag.
    pub fn draw_card(
        &mut self,
        player_id: PlayerId,
        ctx: &ActionContext,
    ) -> Result<Option<ObjectId>, String> {
        let player = self.get_player(player_id)?;

        if player.library.is_empty() {
            // Rule 704.5b: flag that this player attempted to draw from empty library.
            // The actual game loss happens when SBAs are next checked.
            let player_mut = self.get_player_mut(player_id)?;
            player_mut.has_drawn_from_empty_library = true;
            return Ok(None);
        }

        // Top of library = last element in the Vec
        let card_id = {
            let player = self.get_player(player_id)?;
            *player.library.last().unwrap()
        };

        self.change_zone(card_id, Zone::Hand, ZoneChangeCause::Drawn, ctx)?;
        // CR 121.5 — the zone change alone cannot distinguish this from a tutor.
        self.events.emit(GameEvent::CardDrawn { player_id, card_id });
        Ok(Some(card_id))
    }

    /// Draw N cards for a player.
    ///
    /// Returns the IDs of cards actually drawn. If the library runs out mid-draw,
    /// the player is flagged for SBA loss and the remaining draws are skipped.
    pub fn draw_cards(
        &mut self,
        player_id: PlayerId,
        count: u64,
        ctx: &ActionContext,
    ) -> Result<Vec<ObjectId>, String> {
        let mut drawn = Vec::new();
        for _ in 0..count {
            match self.draw_card(player_id, ctx)? {
                Some(id) => drawn.push(id),
                None => break, // library empty, flagged for SBA
            }
        }
        Ok(drawn)
    }

    /// Play a land to the battlefield (special action, not a spell).
    ///
    /// The `from` parameter specifies which zone the land is being played from.
    /// Normally this is `Zone::Hand`, but continuous effects can allow playing
    /// lands from other zones (e.g. graveyard via Crucible of Worlds).
    pub fn play_land(
        &mut self,
        player_id: PlayerId,
        card_id: ObjectId,
        from: Zone,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        // Rule 505.6b: Only the active player can play a land
        if player_id != self.active_player {
            return Err("Only the active player can play a land".to_string());
        }

        // Rule 505.6b: Lands can only be played during a main phase
        match self.phase.phase_type {
            crate::state::game_state::PhaseType::Precombat
            | crate::state::game_state::PhaseType::Postcombat => {}
            _ => return Err("Lands can only be played during a main phase".to_string()),
        }

        // Rule 505.6b: Stack must be empty to play a land
        if !self.stack.is_empty() {
            return Err("Cannot play a land while the stack is not empty".to_string());
        }

        let obj = self.get_object(card_id)?;
        if obj.zone != from {
            return Err(format!("Card is not in {:?}", from));
        }
        if obj.owner != player_id {
            return Err("Can only play your own lands".to_string());
        }
        // PRE-LAYER ZONE: reads printed types on purpose. This is cast-zone /
        // play-from-hand legality, which happens before the object is a permanent,
        // so the layer system has nothing to contribute. Same exemption as
        // engine/cast.rs -- see "Before Layers" in plans/codebase-state.md.
        if !obj.card_data.types.contains(&CardType::Land) {
            return Err("This card is not a land".to_string());
        }

        // Check land drop limit
        let player = self.get_player(player_id)?;
        if !player.can_play_land() {
            return Err("Already played maximum lands this turn".to_string());
        }

        // Move to battlefield, through the chokepoint. This was a direct
        // `move_object` until RA-3 — a fourth, undocumented bypass, and the
        // most frequent zone change in the game. CR 305.1 makes playing a land
        // a special action that still puts a permanent onto the battlefield, so
        // every ETB replacement in Phase RC has to see it.
        self.change_zone(card_id, Zone::Battlefield, ZoneChangeCause::PlayedAsLand, ctx)?;

        // Increment land drop counter
        let player = self.get_player_mut(player_id)?;
        player.lands_played_this_turn += 1;

        Ok(())
    }

    // --- Internal helpers ---

    /// Remove an object ID from the zone's collection
    pub(crate) fn remove_from_zone_collection(&mut self, id: ObjectId, zone: Zone) -> Result<(), String> {
        match zone {
            Zone::Library => {
                let owner = self.get_object(id)?.owner;
                let player = self.get_player_mut(owner)?;
                if let Some(pos) = player.library.iter().position(|&x| x == id) {
                    player.library.remove(pos);
                    Ok(())
                } else {
                    Err(format!("Object {} not found in player {}'s library", id, owner))
                }
            }
            Zone::Hand => {
                let owner = self.get_object(id)?.owner;
                let player = self.get_player_mut(owner)?;
                if let Some(pos) = player.hand.iter().position(|&x| x == id) {
                    player.hand.remove(pos);
                    Ok(())
                } else {
                    Err(format!("Object {} not found in player {}'s hand", id, owner))
                }
            }
            Zone::Battlefield => {
                self.battlefield.remove(&id);
                Ok(())
            }
            Zone::Graveyard => {
                let owner = self.get_object(id)?.owner;
                let player = self.get_player_mut(owner)?;
                if let Some(pos) = player.graveyard.iter().position(|&x| x == id) {
                    player.graveyard.remove(pos);
                    Ok(())
                } else {
                    Err(format!("Object {} not found in player {}'s graveyard", id, owner))
                }
            }
            Zone::Stack => {
                if let Some(pos) = self.stack.iter().position(|&x| x == id) {
                    self.stack.remove(pos);
                    self.stack_entries.remove(&id);
                    Ok(())
                } else {
                    Err(format!("Object {} not found on stack", id))
                }
            }
            Zone::Exile => {
                if let Some(pos) = self.exile.iter().position(|&x| x == id) {
                    self.exile.remove(pos);
                    Ok(())
                } else {
                    Err(format!("Object {} not found in exile", id))
                }
            }
            Zone::Command => {
                if let Some(pos) = self.command.iter().position(|&x| x == id) {
                    self.command.remove(pos);
                    Ok(())
                } else {
                    Err(format!("Object {} not found in command zone", id))
                }
            }
        }
    }

    /// Add an object ID to the zone's collection.
    ///
    /// Per rule 400.3, objects moving to library, hand, or graveyard always go
    /// to their *owner's* zone, regardless of who controlled them. This is why
    /// we use `obj.owner` here, not the controller from `BattlefieldEntity`.
    fn add_to_zone_collection(&mut self, id: ObjectId, zone: Zone) -> Result<(), String> {
        let owner = self.get_object(id)?.owner;

        match zone {
            Zone::Library => {
                let player = self.get_player_mut(owner)?;
                player.library.push(id);
                Ok(())
            }
            Zone::Hand => {
                let player = self.get_player_mut(owner)?;
                player.hand.push(id);
                Ok(())
            }
            Zone::Battlefield => {
                // BattlefieldEntity is created in init_zone_state
                Ok(())
            }
            Zone::Graveyard => {
                let player = self.get_player_mut(owner)?;
                player.graveyard.push(id);
                Ok(())
            }
            Zone::Stack => {
                self.stack.push(id);
                Ok(())
            }
            Zone::Exile => {
                self.exile.push(id);
                Ok(())
            }
            Zone::Command => {
                self.command.push(id);
                Ok(())
            }
        }
    }

    /// Initialize zone-specific state when entering a zone.
    /// Default controller is the object's owner (correct for play_land, tokens, etc.).
    /// Initialize zone-specific state when entering a zone.
    ///
    /// The entering permanent's controller is its owner (correct for a land play
    /// and for tokens) *except* when it is a resolving permanent spell, where
    /// CR 110.2b makes it the player who put that spell onto the stack.
    ///
    /// This is the **default** controller — the value Layer 2 modifies, not the
    /// answer `get_effective_controller` gives. A stolen permanent spell enters
    /// under its caster's control here and is moved to the thief by the Layer 2
    /// row CR 400.7a keeps alive. `GameState::resolving` is where the answer
    /// survives the `StackEntry` being taken.
    pub(crate) fn init_zone_state(&mut self, id: ObjectId, zone: Zone) -> Result<(), String> {
        if zone == Zone::Battlefield {
            let controller = match self.resolving {
                Some(r) if r.id == id => r.default_controller,
                _ => self.get_object(id)?.owner,
            };
            self.place_on_battlefield(id, controller);
        }
        Ok(())
    }

    /// Clean up zone-specific state when leaving a zone.
    ///
    /// Called BEFORE remove_from_zone_collection so we can still read
    /// the departing entity's state. The BattlefieldEntity itself is
    /// removed afterwards by remove_from_zone_collection.
    fn cleanup_zone_state(&mut self, id: ObjectId, zone: Zone) {
        if zone == Zone::Battlefield {
            // Remove any continuous effects generated by this source — CR 611.3b,
            // a static ability applies only while its source is on the battlefield.
            self.continuous_effects.remove_by_source(id);
            // Deliberately *not* the replacement registry. Every row in it was
            // made by a resolution, and CR 611.2a gives those the duration the
            // spell or ability stated, not the source's lifetime — a regeneration
            // shield outlives the permanent whose ability made it. Unspent
            // `Uses::Once` rows expire at the CR 514.2 cleanup instead.
            //
            // The leaving permanent does stop being a gather candidate.
            // Idempotent, so one that never had a replacement ability is a no-op.
            self.replacement_ability_sources.remove(&id);
            // Same rule, same reason (`cant-effects-architecture.md` §3.4): the
            // leaving permanent stops being a restriction-sweep candidate. Its
            // *registry* rows are deliberately untouched for the reason above —
            // a resolution's "can't" has the duration that resolution stated,
            // not the source's lifetime.
            self.restriction_ability_sources.remove(&id);

            // Collect attachment info before mutating
            let (attached_to, attached_by) = {
                if let Some(entry) = self.battlefield.get(&id) {
                    (entry.attached_to, entry.attached_by.clone())
                } else {
                    return;
                }
            };

            // If this permanent was attached to a host, remove it from the host's attached_by
            if let Some(host_id) = attached_to {
                if let Some(host) = self.battlefield.get_mut(&host_id) {
                    host.attached_by.retain(|&aid| aid != id);
                }
            }

            // If this permanent had things attached to it, clear their attached_to
            // (Aura SBAs will handle the resulting unattached auras)
            for attachment_id in attached_by {
                if let Some(attachment) = self.battlefield.get_mut(&attachment_id) {
                    attachment.attached_to = None;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::test_ctx;
    use crate::objects::object::GameObject;
    use crate::state::game_state::GameState;
    use crate::types::zones::Zone;
    use crate::test_support::{forest as make_forest, stock_libraries};

    /// Put one card in player 0's library and return its id.
    fn game_with_one_card_library() -> (GameState, crate::types::ids::ObjectId) {
        let mut game = GameState::new(2, 20);
        let forest = GameObject::in_library(make_forest(), 0);
        let forest_id = game.add_object(forest);
        game.players[0].library.push(forest_id);
        (game, forest_id)
    }

    fn card_drawn_events(game: &GameState) -> Vec<crate::types::ids::ObjectId> {
        game.events.events().filter_map(|e| match e {
            crate::events::event::GameEvent::CardDrawn { card_id, .. } => Some(*card_id),
            _ => None,
        }).collect()
    }

    #[test]
    fn test_draw_emits_card_drawn_alongside_the_zone_change() {
        let (mut game, forest_id) = game_with_one_card_library();
        game.draw_card(0, &test_ctx()).unwrap();

        // Both, not either: CR 121.1 is a library→hand move, and CR 121.5 makes
        // "was it a draw" a separate, trigger-visible fact about that move.
        assert_eq!(card_drawn_events(&game), vec![forest_id]);
        let zone_changes = game.events.events().filter(|e| matches!(
            e, crate::events::event::GameEvent::ZoneChange { from: Zone::Library, to: Zone::Hand, .. }
        )).count();
        assert_eq!(zone_changes, 1);
    }

    // COVERS-PARTIAL: ATOM-121.5-001
    // Builds the "no draw event emitted" half. The atom's trigger half ("no draw
    // triggers fire") needs CR 603, and its empty-library half needs a library→hand
    // effect that can run on an empty library — neither exists yet.
    #[test]
    fn test_library_to_hand_without_drawing_is_not_a_draw() {
        let (mut game, forest_id) = game_with_one_card_library();

        // The same movement a draw makes, proposed as a plain zone change —
        // this is what Nadu, Winged Wisdom lowers to ("reveal the top card of
        // your library … otherwise, put it into your hand"), and what 857 cards
        // that move library→hand without the word "draw" lower to. CR 121.5:
        // the player has not drawn it.
        game.change_zone(
            forest_id,
            Zone::Hand,
            crate::engine::actions::ZoneChangeCause::PutIntoHand,
            &test_ctx(),
        ).unwrap();

        assert_eq!(game.players[0].hand, vec![forest_id]);
        assert!(
            card_drawn_events(&game).is_empty(),
            "a library→hand move without the word \"draw\" is not a draw (CR 121.5)",
        );
    }

    #[test]
    fn test_draw_from_empty_library_emits_no_card_drawn() {
        let mut game = GameState::new(2, 20);
        assert!(game.players[0].library.is_empty());

        let drawn = game.draw_card(0, &test_ctx()).unwrap();

        // Nothing was drawn, so nothing is announced. CR 704.5b picks the
        // attempt up as a state-based action instead. (A *replacement* on the
        // draw would still have applied — CR 121.6a — which is why the empty
        // check lives in the performer and not at the proposal.)
        assert_eq!(drawn, None);
        assert!(card_drawn_events(&game).is_empty());
        assert!(game.players[0].has_drawn_from_empty_library);
    }

    #[test]
    fn test_draw_card() {
        let mut game = GameState::new(2, 20);

        // Put a forest in player 0's library
        let forest = GameObject::in_library(make_forest(), 0);
        let forest_id = game.add_object(forest);
        game.players[0].library.push(forest_id);

        // Draw it
        let drawn = game.draw_card(0, &test_ctx()).unwrap();
        assert_eq!(drawn, Some(forest_id));
        assert!(game.players[0].library.is_empty());
        assert_eq!(game.players[0].hand.len(), 1);
        assert_eq!(game.players[0].hand[0], forest_id);

        // Verify the object's zone was updated
        let obj = game.get_object(forest_id).unwrap();
        assert_eq!(obj.zone, Zone::Hand);
    }

    #[test]
    fn test_draw_from_empty_library() {
        let mut game = GameState::new(2, 20);
        let result = game.draw_card(0, &test_ctx()).unwrap();
        assert_eq!(result, None);
        assert!(game.players[0].has_drawn_from_empty_library);
    }

    #[test]
    fn test_play_land() {
        let mut game = GameState::new(2, 20);
        stock_libraries(&mut game, 5);

        // Advance to Precombat main phase (Untap -> Upkeep -> Draw -> Precombat)
        for _ in 0..3 {
            game.advance_turn(&test_ctx()).unwrap();
        }
        assert_eq!(game.phase.phase_type, crate::state::game_state::PhaseType::Precombat);

        // Create a forest in hand
        let forest = GameObject::new(make_forest(), 0, Zone::Hand);
        let forest_id = game.add_object(forest);
        game.players[0].hand.push(forest_id);

        // Play it
        game.play_land(0, forest_id, Zone::Hand, &test_ctx()).unwrap();

        assert!(game.players[0].hand.len() == 1); // drew 1 card during draw step
        assert!(game.battlefield.contains_key(&forest_id));
        assert_eq!(game.players[0].lands_played_this_turn, 1);

        let obj = game.get_object(forest_id).unwrap();
        assert_eq!(obj.zone, Zone::Battlefield);

        // Should not be able to play a second land
        let forest2 = GameObject::new(make_forest(), 0, Zone::Hand);
        let forest2_id = game.add_object(forest2);
        game.players[0].hand.push(forest2_id);

        let result = game.play_land(0, forest2_id, Zone::Hand, &test_ctx());
        assert!(result.is_err());
    }

    #[test]
    fn test_play_land_wrong_phase() {
        let mut game = GameState::new(2, 20);
        stock_libraries(&mut game, 5);

        // Stay in Beginning phase (Untap step)
        let forest = GameObject::new(make_forest(), 0, Zone::Hand);
        let forest_id = game.add_object(forest);
        game.players[0].hand.push(forest_id);

        let result = game.play_land(0, forest_id, Zone::Hand, &test_ctx());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("main phase"));
    }

    #[test]
    fn test_play_land_wrong_player() {
        let mut game = GameState::new(2, 20);
        stock_libraries(&mut game, 5);

        // Advance to Precombat main (player 0 is active)
        for _ in 0..3 {
            game.advance_turn(&test_ctx()).unwrap();
        }

        // Player 1 tries to play a land during player 0's turn
        let forest = GameObject::new(make_forest(), 1, Zone::Hand);
        let forest_id = game.add_object(forest);
        game.players[1].hand.push(forest_id);

        let result = game.play_land(1, forest_id, Zone::Hand, &test_ctx());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("active player"));
    }

    #[test]
    fn test_zone_transition_battlefield_to_graveyard() {
        let mut game = GameState::new(2, 20);

        // Create a forest on the battlefield
        let forest = GameObject::new(make_forest(), 0, Zone::Battlefield);
        let forest_id = game.add_object(forest);
        game.place_on_battlefield(forest_id, 0);

        // Move to graveyard
        game.move_object(forest_id, Zone::Graveyard).unwrap();

        assert!(!game.battlefield.contains_key(&forest_id));
        assert_eq!(game.players[0].graveyard.len(), 1);
        assert_eq!(game.get_object(forest_id).unwrap().zone, Zone::Graveyard);
    }

    #[test]
    fn test_zone_exit_detaches() {
        let mut game = GameState::new(2, 20);

        // Create a host creature on the battlefield
        let host = GameObject::new(make_forest(), 0, Zone::Battlefield);
        let host_id = game.add_object(host);
        game.place_on_battlefield(host_id, 0);

        // Create an "attachment" (e.g. Equipment) on the battlefield
        let equip = GameObject::new(make_forest(), 0, Zone::Battlefield);
        let equip_id = game.add_object(equip);
        game.place_on_battlefield(equip_id, 0);

        // Wire up attachment relationship
        game.battlefield.get_mut(&equip_id).unwrap().attach_to(host_id);
        game.battlefield.get_mut(&host_id).unwrap().attached_by.push(equip_id);

        // Verify setup
        assert_eq!(game.battlefield.get(&equip_id).unwrap().attached_to, Some(host_id));
        assert_eq!(game.battlefield.get(&host_id).unwrap().attached_by, vec![equip_id]);

        // Host leaves the battlefield — attachment's attached_to should be cleared
        game.move_object(host_id, Zone::Graveyard).unwrap();

        assert!(!game.battlefield.contains_key(&host_id));
        // Equipment stays on battlefield but is no longer attached
        let equip_entry = game.battlefield.get(&equip_id).unwrap();
        assert_eq!(equip_entry.attached_to, None);
    }

    #[test]
    fn test_zone_exit_attachment_leaves() {
        let mut game = GameState::new(2, 20);

        // Create a host creature on the battlefield
        let host = GameObject::new(make_forest(), 0, Zone::Battlefield);
        let host_id = game.add_object(host);
        game.place_on_battlefield(host_id, 0);

        // Create an attachment on the battlefield
        let aura = GameObject::new(make_forest(), 0, Zone::Battlefield);
        let aura_id = game.add_object(aura);
        game.place_on_battlefield(aura_id, 0);

        // Wire up attachment relationship
        game.battlefield.get_mut(&aura_id).unwrap().attach_to(host_id);
        game.battlefield.get_mut(&host_id).unwrap().attached_by.push(aura_id);

        // Attachment leaves the battlefield — host's attached_by should be updated
        game.move_object(aura_id, Zone::Graveyard).unwrap();

        assert!(!game.battlefield.contains_key(&aura_id));
        // Host stays, but no longer has anything attached
        let host_entry = game.battlefield.get(&host_id).unwrap();
        assert!(host_entry.attached_by.is_empty());
    }
}
