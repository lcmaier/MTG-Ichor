use crate::engine::keywords::{apply_deathtouch_flag, apply_lifelink};
use crate::engine::resolve::ResolutionContext;
use crate::events::event::{DamageTarget, GameEvent};
use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::zones::Zone;
use crate::ui::decision::DecisionProvider;

/// Who is asking for a mutation, and what resolution it belongs to.
///
/// `execute_action` has no `DecisionProvider` of its own, and CR 616.1 needs
/// one: when two or more replacement effects want the same event, *the affected
/// object's controller* chooses which to apply — not the controller of the
/// effect. Rather than thread a bare `&dyn DecisionProvider`, this carries the
/// second thing the pipeline will want, so the plumbing is paid for once.
///
/// **Phase RA threads it; nothing reads either field yet.** Phase RB is where
/// `apply_replacements` starts consulting them:
///
/// - `dp` answers the CR 616.1 ordering prompt.
/// - `resolution` is where CR 614.15 self-replacement effects live (they belong
///   to the resolving spell or ability, not to any registry), and it is what
///   stamps every emitted `GameEvent` with the resolution that caused it.
pub struct ActionContext<'a> {
    pub dp: &'a dyn DecisionProvider,
    pub resolution: Option<&'a ResolutionContext>,
}

impl<'a> ActionContext<'a> {
    /// A mutation that belongs to no resolution: a turn-based action, a
    /// state-based action, cost payment, combat damage.
    pub fn new(dp: &'a dyn DecisionProvider) -> Self {
        ActionContext { dp, resolution: None }
    }

    /// A mutation proposed by a resolving spell or ability.
    pub fn resolving(dp: &'a dyn DecisionProvider, resolution: &'a ResolutionContext) -> Self {
        ActionContext { dp, resolution: Some(resolution) }
    }
}

/// Why the engine is moving an object between zones.
///
/// This is the semantic carrier that makes CR 701.8b answerable: `(from, to)`
/// cannot distinguish a sacrifice from a destruction from an SBA, and 1,287
/// printed cards trigger on "dies" while 278 want "sacrifices" specifically.
///
/// **Derived from call sites, not researched from the card pool.** It records
/// what the engine was doing, so the input set is finite and readable off the
/// tree (`replacement-architecture.md` §11). No printed card asks for a cause
/// finer than a call site can name: "destroyed by" appears on 1 card in all of
/// Magic, "was sacrificed" on 3, "if it was destroyed" on 0.
///
/// Three rules, all learned the hard way elsewhere in this tree:
///
/// - **The caller sets it.** `Primitive::Sacrifice` knows it is sacrificing;
///   `perform_action` cannot recover that from `(from, to)`.
/// - **Nothing may branch on it outside the replacement pipeline and the
///   trigger matcher.** A third reader is a third place for it to drift.
/// - **No catchall variant. No `Other`, no `Unknown`, no `#[non_exhaustive]`.**
///   This is the whole of what makes the enum cheap to extend later. Widening
///   is only expensive when an existing site was labelled with a coarse variant
///   that should have been finer, and re-triaging it is guesswork that fails
///   silently — which requires a catchall to lump into. A genuinely new mutation
///   arrives with its own new call site, so it adds a variant and touches
///   nothing existing. A site with no honest reason to give is a site whose
///   reason nobody has worked out, which is the bug — see
///   `cast.rs::rollback_cast_to_hand` for what that looks like when it happens.
///
/// Several variants have no call site yet because their `Primitive` is still
/// `NotImplemented` (`resolve.rs`). They are listed anyway: the enum is the
/// statement of the vocabulary, and nothing matches on it exhaustively until
/// Phase RB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneChangeCause {
    // --- effects (CR 701), one per object-moving `Primitive` ---
    /// 701.8b way 1 — an effect using the word "destroy".
    Destroyed,
    /// 701.21 — NOT destruction. The distinction 278 cards care about.
    Sacrificed,
    /// 701.13.
    Exiled,
    /// 701.9 — includes the CR 514.1 cleanup discard.
    Discarded,
    /// 701.17.
    Milled,
    /// "return to hand" / "return to the battlefield".
    Returned,
    /// Top, bottom, or shuffled in. *Position* is a field, not a cause.
    PutIntoLibrary,

    // --- state-based actions (CR 704.5) ---
    /// 704.5g lethal damage + 704.5h deathtouch. One variant, because CR 701.8b
    /// calls both "destroyed" and no card distinguishes them as a *cause*.
    DestroyedBySba,
    /// 704.5f — NOT destruction, so regeneration and indestructible do not help.
    ZeroToughness,
    /// 704.5i.
    ZeroLoyalty,
    /// 704.5j.
    LegendRule,
    /// 704.5m. (704.5n only unattaches an Equipment; it moves nothing.)
    AuraSba,

    // --- the stack ---
    /// Hand (or elsewhere) → stack.
    Cast,
    /// 608.2m — stack → battlefield or graveyard.
    Resolved,
    /// 701.6.
    Countered,
    /// 608.2b — countered by game rules, all targets illegal.
    Fizzled,

    // --- turn structure and special actions ---
    /// CR 121.5 makes this trigger-visibly distinct from "put into hand".
    Drawn,
    /// 305.1 / 505.6b.
    PlayedAsLand,
}

/// A game action that is *about to happen*.
///
/// This is the pre-mutation counterpart to `GameEvent` (which records what
/// *did* happen). The engine builds a `GameAction`, passes it through
/// `execute_action`, which performs the mutation and emits the corresponding
/// `GameEvent`.
///
/// In Phase 6, a replacement-effect pipeline will sit between "build action"
/// and "execute action", potentially modifying or replacing the action before
/// it is carried out. For now, `execute_action` is a direct passthrough.
#[derive(Debug, Clone)]
pub enum GameAction {
    /// Deal damage from a source to a target.
    DealDamage {
        source: ObjectId,
        target: DamageTarget,
        amount: u64,
        is_combat: bool,
    },

    /// A single card draw for a player.
    ///
    /// Drawing N cards is N individual `DrawCard` actions (rule 121.2).
    DrawCard {
        player: PlayerId,
    },

    /// A player gains life.
    GainLife {
        player: PlayerId,
        amount: u64,
        source: ObjectId,
    },

    /// A player loses life (not from damage).
    LoseLife {
        player: PlayerId,
        amount: u64,
    },

    /// Move an object from one zone to another.
    ///
    /// `cause` is set by the caller and is required — see [`ZoneChangeCause`].
    ZoneChange {
        object: ObjectId,
        from: Zone,
        to: Zone,
        cause: ZoneChangeCause,
    },

    /// Untap a permanent.
    Untap {
        object: ObjectId,
    },

    /// Tap a permanent.
    Tap {
        object: ObjectId,
    },

    // === Phase 3+ actions — add variants here as primitives are implemented ===
    // Sacrifice { object: ObjectId },
    // Exile { object: ObjectId },
    // CreateToken { def: TokenDef, controller: PlayerId, count: u32 },
    // AddCounters { target: ObjectId, counter_type: CounterType, count: u32 },
    // etc.
}

impl GameState {
    /// Execute a game action: mutate state and emit the corresponding event.
    ///
    /// This is the central chokepoint for all game-state mutations that are
    /// observable (i.e., that triggered abilities and replacement effects care
    /// about).
    ///
    /// **Current behavior (through Phase RA):** direct passthrough — performs
    /// the mutation immediately and emits the event. `ctx` is threaded but not
    /// yet read; RA's job is to make sure it is *available* everywhere a
    /// mutation happens.
    ///
    /// **Phase RB:** an `apply_replacements(action, ctx, ...)` call goes in
    /// between, potentially modifying or dropping the action before execution.
    /// The replacement pipeline handles rule 614 (replacement effects),
    /// rule 615 (prevention effects), and rule 616 (interaction ordering).
    pub fn execute_action(
        &mut self,
        action: GameAction,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        // Phase RB: let Some(action) = self.apply_replacements(action, ctx, ...) else { return Ok(()) };
        self.perform_action(action, ctx)
    }

    /// Convenience wrapper for the most common zone change: caller knows the
    /// destination but doesn't want to hand-roll the `from` lookup.
    ///
    /// This is the intended public path for zone changes. Routes through
    /// `execute_action(GameAction::ZoneChange)` so the future replacement
    /// pipeline (CR 614) will see every movement. Internal helpers like
    /// `draw_card` and `play_land` still call `move_object` directly — they
    /// live inside `engine/zones.rs` and go through the same chokepoint
    /// transitively via `execute_action`'s ZoneChange arm.
    pub fn change_zone(
        &mut self,
        object: ObjectId,
        to: Zone,
        cause: ZoneChangeCause,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        let from = self.get_object(object)?.zone;
        self.execute_action(GameAction::ZoneChange { object, from, to, cause }, ctx)
    }

    /// Perform the actual state mutation and emit the event.
    ///
    /// This is separated from `execute_action` so that the replacement pipeline
    /// (Phase RB) can call this with the final, possibly-modified action.
    ///
    /// `_ctx` is the one place in the RA sweep where the context is threaded but
    /// has nothing to read yet. It is a parameter here rather than absent
    /// because this is where it is used *first*: RA-2 routes lifelink's life
    /// gain through `execute_action`, and that proposal is made from inside the
    /// `DealDamage` arm below.
    fn perform_action(
        &mut self,
        action: GameAction,
        _ctx: &ActionContext,
    ) -> Result<(), String> {
        match action {
            GameAction::DealDamage { source, target, amount, is_combat } => {
                if amount == 0 {
                    // Rule 614.7a: 0 damage is not dealt at all.
                    return Ok(());
                }

                match &target {
                    DamageTarget::Object(id) => {
                        if let Some(entry) = self.battlefield.get_mut(id) {
                            entry.damage_marked += amount as u32;
                        } else {
                            return Err(format!(
                                "Target object {} not on battlefield", id
                            ));
                        }
                    }
                    DamageTarget::Player(pid) => {
                        let player = self.get_player_mut(*pid)?;
                        player.life_total -= amount as i64;
                    }
                }

                // Keyword hooks (delegated to engine/keywords.rs)
                apply_deathtouch_flag(self, source, &target);
                apply_lifelink(self, source, amount)?;

                // Rule 903.10a — if a commander deals combat damage to a
                // player, accumulate it per-commander on the damaged player.
                // The 21-damage loss check happens in SBA 704.5u / 903.10a.
                if is_combat {
                    if let DamageTarget::Player(pid) = &target {
                        let is_cmdr = self.objects.get(&source)
                            .map(|o| o.is_commander)
                            .unwrap_or(false);
                        if is_cmdr {
                            let entry = self.get_player_mut(*pid)?
                                .commander_damage_taken
                                .entry(source)
                                .or_insert(0);
                            *entry = entry.saturating_add(amount as u32);
                        }
                    }
                }

                self.events.emit(GameEvent::DamageDealt {
                    source_id: source,
                    target: target.clone(),
                    amount,
                });

                // Emit LifeChanged for player damage
                if let DamageTarget::Player(pid) = &target {
                    let new_life = self.get_player(*pid)?.life_total;
                    self.events.emit(GameEvent::LifeChanged {
                        player_id: *pid,
                        old: new_life + amount as i64,
                        new: new_life,
                        source: Some(source),
                    });
                }

                Ok(())
            }

            GameAction::DrawCard { player } => {
                // Delegate to the existing draw_card method which handles
                // empty-library flagging and zone transitions.
                // draw_card already emits ZoneChange events via move_object.
                self.draw_card(player, _ctx)?;
                Ok(())
            }

            GameAction::GainLife { player, amount, source } => {
                if amount == 0 {
                    return Ok(());
                }
                let old_life = self.get_player(player)?.life_total;
                let p = self.get_player_mut(player)?;
                p.life_total += amount as i64;
                let new_life = p.life_total;

                self.events.emit(GameEvent::LifeChanged {
                    player_id: player,
                    old: old_life,
                    new: new_life,
                    source: Some(source),
                });

                Ok(())
            }

            GameAction::LoseLife { player, amount } => {
                if amount == 0 {
                    return Ok(());
                }
                let old_life = self.get_player(player)?.life_total;
                let p = self.get_player_mut(player)?;
                p.life_total -= amount as i64;
                let new_life = p.life_total;

                self.events.emit(GameEvent::LifeChanged {
                    player_id: player,
                    old: old_life,
                    new: new_life,
                    source: None,
                });

                Ok(())
            }

            // `cause` is carried, not consumed: Phase RB's `apply_replacements`
            // matches on it (CR 701.8b), and RA-3 stamps it onto the emitted
            // `GameEvent`. Nothing else may read it — see `ZoneChangeCause`.
            GameAction::ZoneChange { object, from: _, to, cause: _ } => {
                // Delegate to move_object which handles all zone bookkeeping
                // and emits its own ZoneChange event.
                self.move_object(object, to)?;
                Ok(())
            }

            GameAction::Untap { object } => {
                // Loud: an untap proposed for something not on the battlefield
                // is a caller bug, not a no-op. The caller checks legality —
                // see `Primitive::Untap` in resolve.rs and CR 608.2b.
                let entry = self.battlefield.get_mut(&object).ok_or_else(|| {
                    format!("Cannot untap {}: not on the battlefield", object)
                })?;
                // CR 701.26b — only tapped permanents can be untapped — and
                // CR 603.2e makes "becomes untapped" a transition, not a state.
                if !entry.tapped {
                    return Ok(());
                }
                entry.tapped = false;
                self.events.emit(GameEvent::Untapped { object_id: object });
                Ok(())
            }

            GameAction::Tap { object } => {
                let entry = self.battlefield.get_mut(&object).ok_or_else(|| {
                    format!("Cannot tap {}: not on the battlefield", object)
                })?;
                // CR 701.26a — only untapped permanents can be tapped — and
                // CR 603.2e makes "becomes tapped" a transition, not a state.
                if entry.tapped {
                    return Ok(());
                }
                entry.tapped = true;
                self.events.emit(GameEvent::Tapped { object_id: object });
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::test_ctx;
    use crate::events::event::DamageTarget;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::types::card_types::*;
    use crate::types::keywords::KeywordFlag;
    use crate::types::mana::ManaType;

    fn setup_game_with_creature() -> (GameState, ObjectId) {
        let mut game = GameState::new(2, 20);

        let bears = CardDataBuilder::new("Grizzly Bears")
            .mana_cost(crate::types::mana::ManaCost::build(&[ManaType::Green], 1))
            .color(crate::types::colors::Color::Green)
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(CreatureType::Bear))
            .power_toughness(2, 2)
            .build();

        let obj = GameObject::new(bears, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        (game, id)
    }

    fn tap_events(game: &GameState) -> Vec<&'static str> {
        game.events.events().iter().filter_map(|e| match e {
            GameEvent::Tapped { .. } => Some("tapped"),
            GameEvent::Untapped { .. } => Some("untapped"),
            _ => None,
        }).collect()
    }

    #[test]
    fn test_tap_emits_only_on_the_transition() {
        let (mut game, bears_id) = setup_game_with_creature();

        game.execute_action(GameAction::Tap { object: bears_id }, &test_ctx()).unwrap();
        assert!(game.battlefield.get(&bears_id).unwrap().tapped);
        assert_eq!(tap_events(&game), vec!["tapped"]);

        // CR 701.26a: only untapped permanents can be tapped. CR 603.2e: the
        // trigger event is the *change*, so a redundant tap announces nothing.
        game.execute_action(GameAction::Tap { object: bears_id }, &test_ctx()).unwrap();
        assert!(game.battlefield.get(&bears_id).unwrap().tapped);
        assert_eq!(tap_events(&game), vec!["tapped"], "a redundant tap is not a second event");
    }

    #[test]
    fn test_untap_emits_only_on_the_transition() {
        let (mut game, bears_id) = setup_game_with_creature();

        // Already untapped — CR 701.26b, nothing to untap, nothing announced.
        game.execute_action(GameAction::Untap { object: bears_id }, &test_ctx()).unwrap();
        assert!(tap_events(&game).is_empty());

        game.battlefield.get_mut(&bears_id).unwrap().tapped = true;
        game.execute_action(GameAction::Untap { object: bears_id }, &test_ctx()).unwrap();
        assert!(!game.battlefield.get(&bears_id).unwrap().tapped);
        assert_eq!(tap_events(&game), vec!["untapped"]);
    }

    #[test]
    fn test_tap_and_untap_are_loud_off_the_battlefield() {
        let (mut game, bears_id) = setup_game_with_creature();
        game.battlefield.remove(&bears_id);

        // Previously a silent no-op, against the loud-lowering doctrine. The
        // caller checks legality (CR 608.2b partial resolution); the performer
        // asserts its precondition.
        assert!(game.execute_action(GameAction::Tap { object: bears_id }, &test_ctx()).is_err());
        assert!(game.execute_action(GameAction::Untap { object: bears_id }, &test_ctx()).is_err());
        assert!(tap_events(&game).is_empty());
    }

    #[test]
    fn test_execute_deal_damage_to_creature() {
        let (mut game, bears_id) = setup_game_with_creature();

        game.execute_action(GameAction::DealDamage {
            source: bears_id,
            target: DamageTarget::Object(bears_id),
            amount: 3,
            is_combat: false,
        }, &test_ctx()).unwrap();

        assert_eq!(game.battlefield.get(&bears_id).unwrap().damage_marked, 3);
        // Should have emitted a DamageDealt event
        assert_eq!(game.events.len(), 1);
    }

    #[test]
    fn test_execute_deal_damage_to_player() {
        let (mut game, bears_id) = setup_game_with_creature();

        game.execute_action(GameAction::DealDamage {
            source: bears_id,
            target: DamageTarget::Player(1),
            amount: 3,
            is_combat: false,
        }, &test_ctx()).unwrap();

        assert_eq!(game.players[1].life_total, 17);
        // DamageDealt + LifeChanged
        assert_eq!(game.events.len(), 2);
    }

    #[test]
    fn test_execute_zero_damage_is_noop() {
        let (mut game, bears_id) = setup_game_with_creature();

        game.execute_action(GameAction::DealDamage {
            source: bears_id,
            target: DamageTarget::Player(1),
            amount: 0,
            is_combat: false,
        }, &test_ctx()).unwrap();

        assert_eq!(game.players[1].life_total, 20);
        assert_eq!(game.events.len(), 0);
    }

    #[test]
    fn test_execute_gain_life() {
        let (mut game, bears_id) = setup_game_with_creature();

        game.execute_action(GameAction::GainLife {
            player: 0,
            amount: 5,
            source: bears_id,
        }, &test_ctx()).unwrap();

        assert_eq!(game.players[0].life_total, 25);
        assert_eq!(game.events.len(), 1);
    }

    #[test]
    fn test_execute_lose_life() {
        let (mut game, _bears_id) = setup_game_with_creature();

        game.execute_action(GameAction::LoseLife {
            player: 0,
            amount: 3,
        }, &test_ctx()).unwrap();

        assert_eq!(game.players[0].life_total, 17);
        assert_eq!(game.events.len(), 1);
    }

    #[test]
    fn test_execute_untap() {
        let (mut game, bears_id) = setup_game_with_creature();
        game.battlefield.get_mut(&bears_id).unwrap().tapped = true;

        game.execute_action(GameAction::Untap {
            object: bears_id,
        }, &test_ctx()).unwrap();

        assert!(!game.battlefield.get(&bears_id).unwrap().tapped);
    }

    // --- Lifelink tests (4h) ---

    fn setup_game_with_lifelink_creature() -> (GameState, ObjectId) {
        let mut game = GameState::new(2, 20);

        let data = CardDataBuilder::new("Lifelink Creature")
            .mana_cost(crate::types::mana::ManaCost::build(&[ManaType::White], 1))
            .color(crate::types::colors::Color::White)
            .card_type(CardType::Creature)
            .power_toughness(2, 3)
            .keyword(KeywordFlag::Lifelink)
            .build();

        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        (game, id)
    }

    #[test]
    fn test_lifelink_combat_damage_gains_life() {
        let (mut game, lifelinker) = setup_game_with_lifelink_creature();

        game.execute_action(GameAction::DealDamage {
            source: lifelinker,
            target: DamageTarget::Player(1),
            amount: 2,
            is_combat: true,
        }, &test_ctx()).unwrap();

        // Player 1 took 2 damage: 20 - 2 = 18
        assert_eq!(game.players[1].life_total, 18);
        // Player 0 (controller) gained 2 life: 20 + 2 = 22
        assert_eq!(game.players[0].life_total, 22);
    }

    #[test]
    fn test_lifelink_noncombat_damage_gains_life() {
        let (mut game, lifelinker) = setup_game_with_lifelink_creature();

        game.execute_action(GameAction::DealDamage {
            source: lifelinker,
            target: DamageTarget::Player(1),
            amount: 3,
            is_combat: false,
        }, &test_ctx()).unwrap();

        assert_eq!(game.players[1].life_total, 17);
        assert_eq!(game.players[0].life_total, 23);
    }

    #[test]
    fn test_no_lifelink_no_life_gain() {
        let (mut game, bears_id) = setup_game_with_creature(); // no lifelink

        game.execute_action(GameAction::DealDamage {
            source: bears_id,
            target: DamageTarget::Player(1),
            amount: 2,
            is_combat: true,
        }, &test_ctx()).unwrap();

        assert_eq!(game.players[1].life_total, 18);
        // Player 0 should NOT have gained life
        assert_eq!(game.players[0].life_total, 20);
    }

    // --- T11: LifeChanged source field tests ---

    #[test]
    fn test_life_changed_event_includes_source() {
        // Deal combat damage with a lifelink creature; the resulting
        // LifeChanged events should carry the creature as source.
        let (mut game, lifelinker) = setup_game_with_lifelink_creature();

        game.execute_action(GameAction::DealDamage {
            source: lifelinker,
            target: DamageTarget::Player(1),
            amount: 2,
            is_combat: true,
        }, &test_ctx()).unwrap();

        // Events: DamageDealt, LifeChanged (damage to P1), LifeChanged (lifelink gain for P0)
        let life_events: Vec<_> = game.events.events().iter().filter_map(|e| {
            if let GameEvent::LifeChanged { player_id, old, new, source } = e {
                Some((*player_id, *old, *new, *source))
            } else {
                None
            }
        }).collect();

        assert_eq!(life_events.len(), 2);

        // P0 gained 2 life from lifelink (emitted first, inside apply_lifelink)
        let (pid, old, new, src) = life_events[0];
        assert_eq!(pid, 0);
        assert_eq!(old, 20);
        assert_eq!(new, 22);
        assert_eq!(src, Some(lifelinker));

        // P1 lost 2 life from damage (emitted after keyword hooks)
        let (pid, old, new, src) = life_events[1];
        assert_eq!(pid, 1);
        assert_eq!(old, 20);
        assert_eq!(new, 18);
        assert_eq!(src, Some(lifelinker));
    }

    #[test]
    fn test_simultaneous_lifelink() {
        // Two lifelink creatures deal damage; each produces its own LifeChanged event.
        let mut game = GameState::new(2, 20);

        let make_lifelinker = |game: &mut GameState, name: &str| -> ObjectId {
            let data = CardDataBuilder::new(name)
                .mana_cost(crate::types::mana::ManaCost::build(&[ManaType::White], 1))
                .color(crate::types::colors::Color::White)
                .card_type(CardType::Creature)
                .power_toughness(2, 2)
                .keyword(KeywordFlag::Lifelink)
                .build();
            let obj = GameObject::new(data, 0, Zone::Battlefield);
            let id = obj.id;
            game.add_object(obj);
            game.place_on_battlefield(id, 0);
            id
        };

        let creature_a = make_lifelinker(&mut game, "Lifelinker A");
        let creature_b = make_lifelinker(&mut game, "Lifelinker B");

        // Both deal combat damage to opponent
        game.execute_action(GameAction::DealDamage {
            source: creature_a,
            target: DamageTarget::Player(1),
            amount: 2,
            is_combat: true,
        }, &test_ctx()).unwrap();
        game.execute_action(GameAction::DealDamage {
            source: creature_b,
            target: DamageTarget::Player(1),
            amount: 2,
            is_combat: true,
        }, &test_ctx()).unwrap();

        // P1 took 4 total damage
        assert_eq!(game.players[1].life_total, 16);
        // P0 gained 4 total life from lifelink
        assert_eq!(game.players[0].life_total, 24);

        // Collect all LifeChanged events for P0 (lifelink gains)
        let lifelink_gains: Vec<_> = game.events.events().iter().filter_map(|e| {
            if let GameEvent::LifeChanged { player_id: 0, source, .. } = e {
                Some(*source)
            } else {
                None
            }
        }).collect();

        assert_eq!(lifelink_gains.len(), 2);
        assert_eq!(lifelink_gains[0], Some(creature_a));
        assert_eq!(lifelink_gains[1], Some(creature_b));
    }

    // --- Commander damage tracking (rule 903.11a) ---

    fn setup_game_with_commander() -> (GameState, ObjectId) {
        let mut game = GameState::new(2, 40);

        let general = CardDataBuilder::new("Test General")
            .mana_cost(crate::types::mana::ManaCost::build(&[ManaType::Red], 2))
            .color(crate::types::colors::Color::Red)
            .card_type(CardType::Creature)
            .supertype(crate::types::card_types::Supertype::Legendary)
            .power_toughness(4, 4)
            .build();

        let mut obj = GameObject::new(general, 0, Zone::Battlefield);
        obj.is_commander = true;
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        (game, id)
    }

    #[test]
    fn test_commander_combat_damage_accumulates() {
        let (mut game, cmdr) = setup_game_with_commander();

        game.execute_action(GameAction::DealDamage {
            source: cmdr,
            target: DamageTarget::Player(1),
            amount: 4,
            is_combat: true,
        }, &test_ctx()).unwrap();

        assert_eq!(game.players[1].life_total, 36);
        assert_eq!(game.players[1].commander_damage_taken.get(&cmdr).copied(), Some(4));
    }

    #[test]
    fn test_commander_combat_damage_stacks_across_hits() {
        let (mut game, cmdr) = setup_game_with_commander();

        for _ in 0..3 {
            game.execute_action(GameAction::DealDamage {
                source: cmdr,
                target: DamageTarget::Player(1),
                amount: 7,
                is_combat: true,
            }, &test_ctx()).unwrap();
        }

        // 3 × 7 = 21 — triggers the loss SBA when checked.
        assert_eq!(game.players[1].commander_damage_taken.get(&cmdr).copied(), Some(21));
    }

    #[test]
    fn test_commander_noncombat_damage_not_tracked() {
        // Rule 903.11a applies only to combat damage.
        let (mut game, cmdr) = setup_game_with_commander();

        game.execute_action(GameAction::DealDamage {
            source: cmdr,
            target: DamageTarget::Player(1),
            amount: 4,
            is_combat: false,
        }, &test_ctx()).unwrap();

        assert_eq!(game.players[1].life_total, 36);
        assert!(game.players[1].commander_damage_taken.get(&cmdr).is_none());
    }

    #[test]
    fn test_noncommander_combat_damage_not_tracked() {
        // Only sources flagged `is_commander` contribute.
        let (mut game, bears_id) = setup_game_with_creature();

        game.execute_action(GameAction::DealDamage {
            source: bears_id,
            target: DamageTarget::Player(1),
            amount: 2,
            is_combat: true,
        }, &test_ctx()).unwrap();

        assert!(game.players[1].commander_damage_taken.get(&bears_id).is_none());
    }

    #[test]
    fn test_commander_damage_per_source_not_shared() {
        // Each commander accumulates its own counter on the damaged player.
        let mut game = GameState::new(2, 40);

        let build_cmdr = |game: &mut GameState, name: &str| -> ObjectId {
            let data = CardDataBuilder::new(name)
                .card_type(CardType::Creature)
                .supertype(crate::types::card_types::Supertype::Legendary)
                .power_toughness(3, 3)
                .build();
            let mut obj = GameObject::new(data, 0, Zone::Battlefield);
            obj.is_commander = true;
            let id = obj.id;
            game.add_object(obj);
            game.place_on_battlefield(id, 0);
            id
        };

        let cmdr_a = build_cmdr(&mut game, "General A");
        let cmdr_b = build_cmdr(&mut game, "General B");

        game.execute_action(GameAction::DealDamage {
            source: cmdr_a, target: DamageTarget::Player(1), amount: 3, is_combat: true,
        }, &test_ctx()).unwrap();
        game.execute_action(GameAction::DealDamage {
            source: cmdr_b, target: DamageTarget::Player(1), amount: 3, is_combat: true,
        }, &test_ctx()).unwrap();

        assert_eq!(game.players[1].commander_damage_taken.get(&cmdr_a).copied(), Some(3));
        assert_eq!(game.players[1].commander_damage_taken.get(&cmdr_b).copied(), Some(3));
    }

    #[test]
    fn test_lose_life_event_has_no_source() {
        // LoseLife (e.g., paying life as a cost) has no source object.
        let (mut game, _) = setup_game_with_creature();

        game.execute_action(GameAction::LoseLife {
            player: 0,
            amount: 3,
        }, &test_ctx()).unwrap();

        let life_events: Vec<_> = game.events.events().iter().filter_map(|e| {
            if let GameEvent::LifeChanged { source, .. } = e {
                Some(*source)
            } else {
                None
            }
        }).collect();

        assert_eq!(life_events.len(), 1);
        assert_eq!(life_events[0], None);
    }
}
