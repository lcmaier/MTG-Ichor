//! CR 800.4 — a player leaves the game, and what becomes of what they left.
//!
//! CR 800.4a's four sentences, in the order it writes them, plus CR 800.4c,
//! which is the same predicate as its fourth asked at a different moment.
//! Everything here runs from inside the `GameAction::PlayerLoses` performer or
//! from a duration expiry: "this is not a state-based action. It happens as
//! soon as the player leaves the game."
//!
//! What is **not** here: CR 800.4j and 800.4k, which are rules at the priority
//! loop and the turn rotation; CR 800.4b and 800.4d,
//! which are refusals at the site that would have created or moved the object;
//! and CR 800.4f–i and CR 802, which are `codebase-state.md`'s "Before
//! Commander" item 4.

use crate::engine::actions::{ActionContext, GameAction, ZoneChangeAct};
use crate::events::cause::{EventCause, GameRule};
use crate::events::event::{DepartedObject, GameEvent};
use crate::oracle::characteristics::get_effective_controller;
use crate::state::game_state::GameState;
use crate::types::effects::PlayerRef;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::zones::Zone;
use crate::engine::layers::types::{EffectModification, EffectOrigin};
use crate::ui::decision::DecisionProvider;

impl GameState {
    /// CR 800.4a — whether `object` goes when `player` leaves the game: what
    /// they own leaves the game with them, and what they still control once
    /// the effects giving them control have ended is exiled. That second half
    /// is what they control by default (CR 110.2, 110.2b), which is the one
    /// set both of a batch's readers ask before the leaving performs: its
    /// snapshot of the survivors' abilities and its frames (CR 603.10a).
    ///
    /// Asked before the leaving, so it may include an object that stays: a
    /// permanent another player holds by an effect that outlives the leaving.
    /// Framing one costs a memo read; missing one that goes is a wrong
    /// look-back. Empty in a two-player game, which ends instead (CR 104.2a).
    pub(crate) fn departs_with(&self, object: ObjectId, player: PlayerId) -> bool {
        self.is_multiplayer()
            && (self.objects.get(&object).is_some_and(|obj| obj.owner == player)
                || crate::engine::layers::compute::base_controller(self, object, None) == Some(player))
    }

    /// CR 800.4a — `player` has just left the game; take their objects with
    /// them, end the control they were given, and exile what is left behind.
    ///
    /// The four clauses run in the rule's own order, and the order is load
    /// bearing twice. Clause 1 before clause 2 is the rule's Mind Control
    /// example: the Aura is owned by the departing player, so it leaves the
    /// game and the creature it enchanted reverts to its own controller. And
    /// clause 2 before clause 4 is the Act of Treason example: the steal ends
    /// first, so the stolen creature goes home rather than being exiled.
    pub(crate) fn player_left_the_game(
        &mut self,
        player: PlayerId,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        if !self.is_multiplayer() {
            return Ok(());
        }
        self.owned_objects_leave(player, ctx.cause)?;
        self.end_control_given_to(player);
        self.uncarded_stack_objects_cease(player)?;
        self.exile_objects_no_player_in_game_controls(ctx.dp)
    }

    /// CR 800.4a's first sentence — "all objects (see rule 109) owned by that
    /// player leave the game".
    ///
    /// **Read off the zone collections, never off `objects`.** The log records
    /// one `LeftTheGame` per object, so the order is observable, and
    /// `GameState::objects` is a `HashMap` whose iteration order differs per
    /// process. The collections are the ordered view of the same set: three of
    /// them belong to the owner by CR 400.3 and four are shared and filtered.
    /// An object in the battlefield *zone* with no entity — a token whose
    /// entry was substituted, for the width of one statement — is in no
    /// collection and is nobody's to find; it is removed by CR 704.5d or by
    /// `create_tokens`' own CR 111.5 sweep.
    ///
    /// The order among the seven is this function's and not the CR's, which
    /// names none: the board first, because that is what a reader is watching.
    fn owned_objects_leave(&mut self, player: PlayerId, cause: EventCause) -> Result<(), String> {
        let mut leaving: Vec<(ObjectId, Zone)> = Vec::new();
        let owned = |game: &GameState, id: &ObjectId| {
            game.objects.get(id).is_some_and(|obj| obj.owner == player)
        };
        for id in self.battlefield_ids_ordered() {
            if owned(self, &id) {
                leaving.push((id, Zone::Battlefield));
            }
        }
        for (zone, ids) in [
            (Zone::Stack, self.stack.iter().copied().collect()),
            (Zone::Exile, self.exile.clone()),
            (Zone::Command, self.command.clone()),
        ] {
            leaving.extend(ids.into_iter().filter(|id| owned(self, id)).map(|id| (id, zone)));
        }
        let p = self.get_player(player)?;
        for (zone, ids) in [
            (Zone::Graveyard, p.graveyard.clone()),
            (Zone::Hand, p.hand.clone()),
            (Zone::Library, p.library.clone()),
        ] {
            leaving.extend(ids.into_iter().map(|id| (id, zone)));
        }

        for (id, from) in leaving {
            // CR 603.6c wants the frame: a leaves-the-battlefield ability
            // triggers "when a phased-in permanent leaves the game because its
            // owner leaves the game".
            let left = self.object_ref(id).ok_or_else(|| format!("{id} left the game from no zone"))?;
            let departure = self.depart(id, from, None);
            self.remove_from_game(id)?;
            let departed = DepartedObject::new(left, player, departure.frame.clone());
            self.emit_event(GameEvent::LeftTheGame { departed, from }, cause)?;
            self.announce_unattached_by_leaving(departure, cause)?;
        }
        Ok(())
    }

    /// CR 800.4a's second sentence — "any effects which give that player
    /// control of any objects or players end".
    ///
    /// **The residual after clause 1, and it is small on purpose.** A Layer 2
    /// row from a static ability dies with its source, and clause 1 or clause
    /// 4 has just taken every source the departing player owned or controlled
    /// — `cleanup_zone_state`'s `remove_rows_ending_with` is what ends those.
    /// What is left is a row a *resolution* created, which outlives its source
    /// (CR 611.2a): Act of Treason's, whose source is a sorcery in a graveyard.
    ///
    /// `Owner` and `Opponent` name no single beneficiary a row can be judged
    /// by — Homeward Path hands each creature to a different player — so they
    /// are not ended here. They need nothing: a creature the departing player
    /// owns has left with clause 1, and one they merely control is exiled by
    /// clause 4 if the row still points at them.
    ///
    /// "Or players" has no rows at all: nothing in this engine controls a
    /// player (CR 800.4b's fourth sentence, recorded and not built).
    ///
    /// **What deleting a row means is CR 110.2's, and that is why this can be
    /// a deletion.** "A permanent's controller is, by default, the player under
    /// whose control it entered the battlefield", and 110.2b makes that the
    /// player who put the spell on the stack — a fact `PermanentState`
    /// *stores*, so control falls back to it with nothing to undo. A row's
    /// duration is not consulted: CR 800.4a ends the effect whether or not it
    /// had one, which is what separates this from the CR 514.2 cleanup. An
    /// Aethersnatch-shaped row (no duration, so nothing ends it at cleanup)
    /// ends here and nowhere else.
    fn end_control_given_to(&mut self, player: PlayerId) {
        let doomed: Vec<_> = self
            .continuous_effects
            .iter()
            .filter(|effect| {
                let EffectModification::SetController(player_ref) = &effect.modification else {
                    return false;
                };
                let beneficiary = match player_ref {
                    PlayerRef::Player(pid) => Some(*pid),
                    // CR 109.5 — a static ability's "you" is its source's
                    // current controller; a resolution's is the player who
                    // controlled the spell or ability, which the row stores.
                    PlayerRef::You => match effect.origin {
                        EffectOrigin::Resolution => Some(effect.controller),
                        EffectOrigin::StaticAbility { .. } => {
                            get_effective_controller(self, effect.source)
                        }
                    },
                    PlayerRef::Owner
                    | PlayerRef::OwnerOf(_)
                    | PlayerRef::TheEventsPlayer
                    | PlayerRef::ReferredPlayer
                    | PlayerRef::Opponent
                    | PlayerRef::DefendingPlayer(_)
                    | PlayerRef::AnyDefendingPlayer => None,
                };
                beneficiary == Some(player)
            })
            .map(|effect| effect.id)
            .collect();
        for id in doomed {
            self.continuous_effects.remove(id);
        }
    }

    /// CR 800.4a's third sentence — "if that player controlled any objects on
    /// the stack not represented by cards, those objects cease to exist".
    ///
    /// **Empty today, and written anyway.** An ability on the stack carries
    /// its activator as its `GameObject::owner`, so clause 1 has already taken
    /// every one the departing player controlled by the time this looks. The
    /// customer that will make it bite is a copy of a spell (CR 707.10,
    /// `is_copy`), which is owned by whoever created it and controlled by
    /// whoever it was made for. Eight lines is cheaper than the ledger entry
    /// that would have stood in for them.
    ///
    /// It announces nothing. An ability ceasing to exist is silent everywhere
    /// in this engine — CR 608.2n's is, and CR 701.6b's is announced as the
    /// *countering* rather than as the removal — and nothing printed watches
    /// one.
    fn uncarded_stack_objects_cease(&mut self, player: PlayerId) -> Result<(), String> {
        let ceasing: Vec<ObjectId> = self
            .stack
            .iter()
            .copied()
            .filter(|id| {
                // CR 707.10's "a copy of a spell is not a card", plus what is
                // no spell at all: `is_spell` is false for an ability, activated
                // or triggered; `is_copy` is the spell copies' leg, with no
                // writer until `spell-copies` (`copy-effects-architecture.md`).
                let not_a_card = self.stack_entries.get(id).is_some_and(|e| !e.is_spell)
                    || self.objects.get(id).is_some_and(|obj| obj.is_copy);
                not_a_card && get_effective_controller(self, *id) == Some(player)
            })
            .collect();
        for id in ceasing {
            self.remove_from_game(id)?;
        }
        Ok(())
    }

    /// CR 800.4a's fourth sentence and CR 800.4c — every object no player
    /// still in the game controls is exiled.
    ///
    /// **One predicate for two rules, which is what they both reduce to.**
    /// 800.4a asks it the moment a player leaves ("if there are any objects
    /// still controlled by that player, those objects are exiled") and 800.4c
    /// asks it the moment a control-changing effect ends ("the player who
    /// controlled that object by default has left the game"). Both are "the
    /// effective controller is not in the game", and neither is a state-based
    /// action — each says so in the same words — so this runs at the event
    /// rather than at the next sweep.
    ///
    /// CR 800.4c's third condition, "there is no other effect giving control
    /// of that object to another player in the game", is the layer walk's
    /// answer and not a second test: the effective controller *is* the top
    /// Layer 2 row's, so a surviving row that names a live player is exactly
    /// the case this does not exile.
    ///
    /// **One event**: "those objects are exiled" is one instruction over a
    /// set, as a spell's is (CR 608.2f), so every exile is decided against
    /// the board before any of them (CR 614.4). A replacement whose source is
    /// among the exiled replaces every exile in it, whichever the batch
    /// performs first; one at a time, it was gone for the ones after it.
    ///
    /// Free while everyone is still playing, which is the two-player game and
    /// the part of every other one before the first departure.
    pub(crate) fn exile_objects_no_player_in_game_controls(
        &mut self,
        dp: &dyn DecisionProvider,
    ) -> Result<(), String> {
        if !self.is_multiplayer() || self.players_in_game().count() == self.num_players() {
            return Ok(());
        }
        let orphaned: Vec<GameAction> = self
            .battlefield_ids_ordered()
            .into_iter()
            .chain(self.stack.iter().copied())
            .filter(|&id| get_effective_controller(self, id).is_some_and(|p| self.left_a_multiplayer_game(p)))
            .filter_map(|object| {
                let from = self.objects.get(&object)?.zone;
                Some(GameAction::ZoneChange { object, from, to: Zone::Exile, act: ZoneChangeAct::Put })
            })
            .collect();
        if orphaned.is_empty() {
            return Ok(());
        }
        let rule = ActionContext::game_rule(dp, GameRule::ControllerLeftTheGame);
        self.execute_actions(orphaned, &rule).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::actions::ZoneChangeAct;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::game_state::GameState;
    use crate::test_support::{setup_game, test_ctx, vanilla_creature};
    use crate::types::zones::Zone;

    /// CR 800.4b's third sentence — "if an object would be put onto the
    /// battlefield ... under the control of a player who has left the game,
    /// that object remains in its current zone".
    ///
    /// A unit test rather than an integration one because `propose_entry` is
    /// the site and it is `pub(crate)`: the production route to it with a
    /// departed controller is a permanent spell whose CR 110.2b default
    /// controller left, and no registered card puts a spell on the stack under
    /// a player who does not own it.
    #[test]
    fn an_object_is_not_put_onto_the_battlefield_under_a_departed_players_control() {
        let mut game = setup_game(4);
        game.player_lost[1] = true;
        let card = GameObject::new(vanilla_creature(2, 2, &[]), 2, Zone::Hand);
        let id = game.add_object(card);
        game.players[2].hand.push(id);

        let entered = game
            .propose_entry(id, Some(Zone::Hand), 1, Some(ZoneChangeAct::Put), crate::types::replacement::EnterMods::NONE, &test_ctx())
            .unwrap();

        assert!(!entered);
        assert_eq!(game.get_object(id).unwrap().zone, Zone::Hand, "it remains in its current zone");
        assert!(game.battlefield.is_empty());
    }

    /// The same call in a two-player game, where CR 800.1 puts the rule out of
    /// scope and the entry happens.
    #[test]
    fn a_two_player_game_does_not_refuse_the_entry() {
        let mut game = GameState::new(2, 20);
        game.player_lost[1] = true;
        let card = GameObject::new(CardDataBuilder::new("Fixture").build(), 0, Zone::Hand);
        let id = game.add_object(card);
        game.players[0].hand.push(id);

        let entered = game
            .propose_entry(id, Some(Zone::Hand), 1, Some(ZoneChangeAct::Put), crate::types::replacement::EnterMods::NONE, &test_ctx())
            .unwrap();

        assert!(entered);
        assert_eq!(game.get_object(id).unwrap().zone, Zone::Battlefield);
    }
}
