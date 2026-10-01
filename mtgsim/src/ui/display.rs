// Text formatting helpers for CLI output and logging.
//
// All functions are pure formatters over &GameState — no mutations.
// Lives in ui/ because these are presentation helpers, not game-state queries.

use std::sync::Arc;

use crate::engine::layers::compute_characteristics;
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::events::event::{GameEvent, NamesAsAnnounced};
use crate::objects::card_data::AbilityType;
use crate::oracle::characteristics::{
    get_effective_power, get_effective_toughness, is_creature,
};
use crate::state::game_state::{GameState, PhaseType, StepType};
use crate::types::ids::ObjectId;
use crate::types::keywords::KeywordFlag;

/// The name the object has now, through the layers: a Clone copying Grizzly
/// Bears is Grizzly Bears (CR 707.2).
pub fn card_name(game: &GameState, id: ObjectId) -> String {
    compute_characteristics(game, id)
        .map(|chars| chars.name.clone())
        .unwrap_or_else(|| "<unknown>".to_string())
}

/// The name printed on the card, for a record an observer writes. A read
/// through the layers counts a walk and fills the memo, which the trace sink
/// and the dispatch audit may not do (`the_sink_changes_nothing_the_game_does`,
/// `an_audited_game_counts_and_traces_what_an_unaudited_one_does`).
pub fn printed_name(game: &GameState, id: ObjectId) -> String {
    game.objects.get(&id)
        .map(|obj| obj.card_data.name.clone())
        .unwrap_or_else(|| "<unknown>".to_string())
}

/// Format a battlefield permanent for display.
/// Example: "Grizzly Bears 2/2 [tapped]" or "Forest [tapped]"
pub fn format_permanent(game: &GameState, id: ObjectId) -> String {
    let name = card_name(game, id);
    let entry = match game.battlefield.get(&id) {
        Some(e) => e,
        None => return name,
    };

    let mut parts = vec![name];

    // P/T for creatures
    if is_creature(game, id) {
        let p = get_effective_power(game, id).unwrap_or(0);
        let t = get_effective_toughness(game, id).unwrap_or(0);
        let dmg = entry.damage_marked;
        if dmg > 0 {
            parts.push(format!("{}/{} ({}dmg)", p, t, dmg));
        } else {
            parts.push(format!("{}/{}", p, t));
        }
    }

    // Abilities: keywords shown compact, non-keyword abilities listed individually
    let keywords = collect_keywords(game, id);
    if !keywords.is_empty() {
        parts.push(format!("[{}]", keywords.join(", ")));
    }
    let ability_lines = format_abilities(game, id);
    if !ability_lines.is_empty() {
        parts.push(format!("{{{}}}" , ability_lines.join("; ")));
    }

    // Status flags
    let mut flags = Vec::new();
    if entry.tapped {
        flags.push("tapped");
    }
    if crate::oracle::characteristics::has_summoning_sickness(game, id) {
        flags.push("sick");
    }
    if entry.attacking.is_some() {
        flags.push("attacking");
    }
    if entry.blocking.is_some() {
        flags.push("blocking");
    }
    if !flags.is_empty() {
        parts.push(format!("({})", flags.join(", ")));
    }

    parts.join(" ")
}

/// A permanent's keywords in `KeywordFlag`'s order, since the effective set is
/// a hash set.
fn collect_keywords(game: &GameState, id: ObjectId) -> Vec<&'static str> {
    let Some(chars) = compute_characteristics(game, id) else { return Vec::new() };
    let mut flags: Vec<KeywordFlag> = chars.keyword_flags.iter().copied().collect();
    flags.sort();
    flags.into_iter().map(keyword_name).collect()
}

/// A keyword as it prints, one arm per flag and no wildcard: a new flag does
/// not compile until this says what it prints as.
fn keyword_name(flag: KeywordFlag) -> &'static str {
    match flag {
        KeywordFlag::Deathtouch => "deathtouch",
        KeywordFlag::Defender => "defender",
        KeywordFlag::DoubleStrike => "double strike",
        KeywordFlag::FirstStrike => "first strike",
        KeywordFlag::Flash => "flash",
        KeywordFlag::Flying => "flying",
        KeywordFlag::Haste => "haste",
        KeywordFlag::Hexproof => "hexproof",
        KeywordFlag::Indestructible => "indestructible",
        KeywordFlag::Intimidate => "intimidate",
        KeywordFlag::Lifelink => "lifelink",
        KeywordFlag::Menace => "menace",
        KeywordFlag::Reach => "reach",
        KeywordFlag::Shroud => "shroud",
        KeywordFlag::Trample => "trample",
        KeywordFlag::Vigilance => "vigilance",
    }
}

/// Format non-keyword abilities on a permanent for inline display.
///
/// Keywords are already shown via `collect_keywords` in a compact `[keyword, ...]`
/// block. This function handles the remaining ability types: activated, triggered,
/// static (non-keyword), and mana abilities. Each is shown as a short description.
///
/// Never the printed rules text: the layers can empty the list (a copy of a
/// vanilla creature, a creature under Humility), and the text would then
/// describe abilities the object does not have.
fn format_abilities(game: &GameState, id: ObjectId) -> Vec<String> {
    // Effective abilities, so a Blood-Mooned land isn't displayed with the
    // abilities CR 305.7 took away.
    let abilities = crate::oracle::characteristics::get_effective_abilities(game, id);

    let mut lines = Vec::new();
    for (i, ability) in abilities.iter().enumerate() {
        match ability.ability_type {
            // Mana abilities: show what they produce
            AbilityType::Mana => {
                if let crate::types::effects::Effect::Atom(
                    crate::types::effects::Primitive::ProduceMana(ref output),
                    _,
                ) = ability.effect
                {
                    let mana_str: Vec<String> = output.mana.iter()
                        .filter_map(|(mt, expr)| {
                            let letter = match mt {
                                crate::types::mana::ManaType::White => "W",
                                crate::types::mana::ManaType::Blue => "U",
                                crate::types::mana::ManaType::Black => "B",
                                crate::types::mana::ManaType::Red => "R",
                                crate::types::mana::ManaType::Green => "G",
                                crate::types::mana::ManaType::Colorless => "C",
                            };
                            match expr {
                                crate::types::effects::AmountExpr::Fixed(0) => None,
                                crate::types::effects::AmountExpr::Fixed(1) => {
                                    Some(format!("{{{}}}", letter))
                                }
                                crate::types::effects::AmountExpr::Fixed(amt) => {
                                    Some(format!("{}{}", amt, letter))
                                }
                                _ => Some(format!("{{?{}}}", letter)),
                            }
                        })
                        .collect();
                    lines.push(format!("mana: Add {}", mana_str.join("")));
                }
            }
            // Activated abilities: show cost -> effect summary
            AbilityType::Activated => {
                lines.push(format!("activated({})", i));
            }
            // Triggered abilities: show rules text if available
            AbilityType::Triggered => {
                lines.push(format!("triggered({})", i));
            }
            // Static abilities (non-keyword): show rules text
            AbilityType::Static => {
                lines.push(format!("static({})", i));
            }
            // Spell abilities live on instants/sorceries, not permanents
            AbilityType::Spell => {}
        }
    }
    lines
}

/// Format the current phase/step for display.
pub fn format_phase(game: &GameState) -> String {
    let phase = phase_name(game.phase.phase_type);
    match game.phase.step {
        Some(step) => format!("{} — {}", phase, step_name(step)),
        None => phase.to_string(),
    }
}

/// A phase as a person reads it, which a scenario's `step` word reuses.
pub fn phase_name(phase: PhaseType) -> &'static str {
    match phase {
        PhaseType::Beginning => "Beginning",
        PhaseType::Precombat => "Precombat Main",
        PhaseType::Combat => "Combat",
        PhaseType::Postcombat => "Postcombat Main",
        PhaseType::Ending => "Ending",
    }
}

/// A step as a person reads it.
pub fn step_name(step: StepType) -> &'static str {
    match step {
        StepType::Untap => "Untap",
        StepType::Upkeep => "Upkeep",
        StepType::Draw => "Draw",
        StepType::BeginCombat => "Begin Combat",
        StepType::DeclareAttackers => "Declare Attackers",
        StepType::DeclareBlockers => "Declare Blockers",
        StepType::FirstStrikeDamage => "First Strike Damage",
        StepType::CombatDamage => "Combat Damage",
        StepType::EndCombat => "End Combat",
        StepType::End => "End",
        StepType::Cleanup => "Cleanup",
    }
}

// ---------------------------------------------------------------------------
// Event log formatting
// ---------------------------------------------------------------------------

/// "Grizzly Bears (#12)", the shape every line naming an object uses, or
/// "Grizzly Bears (Clone, #12)" when `name` is not its card's, so a copy reads
/// as what it is.
pub fn object_label(game: &GameState, id: ObjectId, name: &str) -> String {
    match game.objects.get(&id).map(|obj| obj.card_data.name.as_str()) {
        Some(card) if card != name => format!("{name} ({card}, {id})"),
        _ => format!("{name} ({id})"),
    }
}

/// The object under the name the record kept for it, where a copy made it
/// other than its card's, else its card's; the bare id for an object the
/// store no longer holds.
fn name_with_id(game: &GameState, id: ObjectId, announced: &NamesAsAnnounced) -> String {
    let kept = announced.as_deref().and_then(|names| names.iter().find(|(named, _)| *named == id));
    match (kept, game.objects.get(&id)) {
        (Some((_, name)), _) => object_label(game, id, name),
        (None, Some(obj)) => format!("{} ({})", obj.card_data.name, id),
        (None, None) => format!("{}", id),
    }
}

/// An object as it was in the zone it left: its look-back frame from the
/// battlefield (CR 603.10a), its card anywhere else.
fn as_it_left(game: &GameState, id: ObjectId, lki: &Option<Arc<EffectiveCharacteristics>>) -> String {
    match lki {
        Some(frame) => object_label(game, id, &frame.name),
        None => name_with_id(game, id, &None),
    }
}

/// Format one event, each object named as its record kept it.
pub fn format_event(game: &GameState, event: &GameEvent, announced: &NamesAsAnnounced) -> String {
    use crate::events::event::CounterSubject;
    use crate::events::event::GameEvent::*;
    let obj_name = |game: &GameState, id: ObjectId| name_with_id(game, id, announced);
    match event {
        ZoneChange { object_id, owner, from, to, cause, lki } => {
            // The cause says which rule moved it and the CR 603.10a frame says every
            // type it had, which is what a Gideon or an artifact creature needs.
            let was = lki.as_ref().map(|f| {
                // Sorted: `types` is a `HashSet`, and an unsorted log line
                // differs run to run.
                let mut names: Vec<String> = f.types.iter().map(|t| format!("{:?}", t)).collect();
                names.sort();
                format!(" ({})", names.join(" "))
            }).unwrap_or_default();
            format!("ZoneChange: {}{} [P{}] {:?} -> {:?} [{:?}]",
                    as_it_left(game, *object_id, lki), was, owner, from, to, cause)
        }
        AbilityActivated { identity, controller } => format!(
            "AbilityActivated: {} [P{}]", obj_name(game, identity.source.id), controller),
        AbilityTriggered { seq, origin, controller, .. } => format!(
            "AbilityTriggered: {} [P{}] #{}", obj_name(game, origin.source()), controller, seq.0),
        AbilityResolved { identity, controller } => format!(
            "AbilityResolved: {} [P{}]", obj_name(game, identity.source.id), controller),
        Tapped { object_id } => format!("Tapped: {}", obj_name(game, *object_id)),
        Untapped { object_id } => format!("Untapped: {}", obj_name(game, *object_id)),
        CardDrawn { player_id, card_id } => {
            format!("CardDrawn: P{} drew {}", player_id, obj_name(game, *card_id))
        }
        ManaAdded { player_id, source_id, mana, tapped_for_mana } => {
            let mana_str: Vec<String> = mana.iter()
                .map(|(t, v)| format!("{:?}:{}", t, v))
                .collect();
            format!(
                "ManaAdded: P{} from {} [{}]{}",
                player_id,
                obj_name(game, *source_id),
                mana_str.join(", "),
                if *tapped_for_mana { " tapped for mana" } else { "" },
            )
        }
        DamageDealt { source_id, target, amount, is_combat } => {
            let target_str = match target {
                crate::events::event::DamageTarget::Player(pid) => format!("P{}", pid),
                crate::events::event::DamageTarget::Object(oid) => obj_name(game, *oid),
            };
            format!(
                "DamageDealt: {} -> {} for {}{}",
                obj_name(game, *source_id),
                target_str,
                amount,
                if *is_combat { " (combat)" } else { "" },
            )
        }
        PhaseBegin { phase, player } => format!("PhaseBegin: {:?} [P{}]", phase, player),
        StepBegin { step, player } => format!("StepBegin: {:?} [P{}]", step, player),
        TurnBegin { player, turn_number } => format!("TurnBegin: P{} turn {}", player, turn_number),
        PermanentEnteredBattlefield { object_id, controller } => {
            format!("ETB: {} [P{}]", obj_name(game, *object_id), controller)
        }
        LifeChanged { player_id, old, new, source, .. } => {
            let src = match source {
                Some(id) => format!(" (source: {})", obj_name(game, *id)),
                None => String::new(),
            };
            format!("LifeChanged: P{} {} -> {}{}", player_id, old, new, src)
        }
        AttackersDeclared { attackers } => {
            let names: Vec<String> = attackers.iter().map(|id| obj_name(game, *id)).collect();
            format!("AttackersDeclared: [{}]", names.join(", "))
        }
        BlockersDeclared { blockers } => {
            let pairs: Vec<String> = blockers.iter()
                .map(|(b, a)| format!("{} blocks {}", obj_name(game, *b), obj_name(game, *a)))
                .collect();
            format!("BlockersDeclared: [{}]", pairs.join(", "))
        }
        SpellCast { spell_id, caster } => {
            format!("SpellCast: P{} casts {}", caster, obj_name(game, *spell_id))
        }
        SpellCountered { spell_id, countered_by } => {
            format!("SpellCountered: {} countered by {}", obj_name(game, *spell_id), obj_name(game, *countered_by))
        }
        AbilityCountered { ability_id, countered_by } => {
            format!("AbilityCountered: {} countered by {}", obj_name(game, *ability_id), obj_name(game, *countered_by))
        }
        SpellFizzled { spell_id } => {
            format!("SpellFizzled: {}", obj_name(game, *spell_id))
        }
        PlayerLost { player_id, reason } => {
            format!("PlayerLost: P{} ({:?})", player_id, reason)
        }
        PlayerWon { player_id } => format!("PlayerWon: P{}", player_id),
        CountersChanged { subject, counter, added } => {
            let verb = if *added >= 0 { "put on" } else { "removed from" };
            let whom = match subject {
                CounterSubject::Object(id) => obj_name(game, *id),
                CounterSubject::Player(pid) => format!("P{}", pid),
            };
            format!("CountersChanged: {} {:?} counter(s) {} {}", added.abs(), counter, verb, whom)
        }
        Scried { player_id, n, looked_at } => {
            // Both numbers, and they differ only on a short library — which is
            // the case Elrond, Master of Healing's ruling is about, so a log
            // that showed one would hide it.
            format!("Scried: P{} scry {} (looked at {})", player_id, n, looked_at)
        }
        LibraryShuffled { player_id } => format!("LibraryShuffled: P{}", player_id),
        CountersAnnihilated { object_id, pairs_removed } => {
            format!("CountersAnnihilated: {} ({} pairs)", obj_name(game, *object_id), pairs_removed)
        }
        Attached { attachment, host, former_host } => match former_host {
            Some(former) => format!(
                "Attached: {} to {} (from {})",
                obj_name(game, *attachment), obj_name(game, *host), obj_name(game, *former)
            ),
            None => format!("Attached: {} to {}", obj_name(game, *attachment), obj_name(game, *host)),
        },
        EquipmentDetached { equipment_id, former_host } => {
            format!("EquipmentDetached: {} from {}", obj_name(game, *equipment_id), obj_name(game, *former_host))
        }
        LeftTheGame { object_id, owner, from, lki } => {
            format!("LeftTheGame: {} (P{}, from {:?})", as_it_left(game, *object_id, lki), owner, from)
        }
        TokenCreated { object_id, owner, zone } => {
            format!("TokenCreated: {} (P{}, in {:?})", obj_name(game, *object_id), owner, zone)
        }
        TokenCeasedToExist { object_id } => {
            format!("TokenCeasedToExist: {}", obj_name(game, *object_id))
        }
        StateBasedActionPerformed => "StateBasedActionPerformed".to_string(),
    }
}

/// Format every recorded event with resolved card names. The game must be
/// recording (`GameState::record_events`).
pub fn format_event_log(game: &GameState) -> Vec<String> {
    game.recorded_events()
        .records()
        .iter()
        .map(|record| format_event(game, &record.event, &record.names))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::PermanentState;
    use crate::state::game_state::{GameState, Phase};
    use crate::types::card_types::CardType;
    use crate::types::zones::Zone;

    #[test]
    fn test_card_name() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Forest").card_type(CardType::Land).build();
        let obj = GameObject::new(data, 0, Zone::Hand);
        let id = game.add_object(obj);

        assert_eq!(card_name(&game, id), "Forest");
    }

    #[test]
    fn test_format_permanent_creature() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = game.add_object(obj);
        let entry = PermanentState::new(id, 0, 0);
        game.insert_battlefield_entity(id, entry);

        let display = format_permanent(&game, id);
        assert!(display.contains("Grizzly Bears"));
        assert!(display.contains("2/2"));
    }

    #[test]
    fn test_format_permanent_tapped() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = game.add_object(obj);
        let mut entry = PermanentState::new(id, 0, 0);
        entry.tapped = true;
        game.insert_battlefield_entity(id, entry);

        let display = format_permanent(&game, id);
        assert!(display.contains("tapped"));
    }

    #[test]
    fn test_format_phase() {
        let mut game = GameState::new(2, 20);
        game.set_turn_position(Phase::new(PhaseType::Precombat));
        assert_eq!(format_phase(&game), "Precombat Main");

        game.set_turn_position(Phase::new(PhaseType::Beginning));
        assert!(format_phase(&game).contains("Untap"));
    }

    #[test]
    fn test_format_permanent_with_mana_ability() {
        use crate::types::card_types::*;
        use crate::types::mana::ManaType;

        let mut game = GameState::new(2, 20);
        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .mana_ability_single(ManaType::Green)
            .build();
        let obj = GameObject::new(forest, 0, Zone::Battlefield);
        let id = game.add_object(obj);
        let entry = PermanentState::new(id, 0, 0);
        game.insert_battlefield_entity(id, entry);

        let display = format_permanent(&game, id);
        assert!(display.contains("mana: Add"), "Should show mana ability");
        assert!(display.contains("{G}"), "Should show green mana");
    }

    #[test]
    fn a_clone_copying_grizzly_bears_shows_as_grizzly_bears() {
        use crate::engine::actions::ActionContext;
        use crate::test_support::{put_in_graveyard, put_on_battlefield, setup_two_player_game, RecordingDecisionProvider};
        use crate::types::zones::ZoneChangeCause;

        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, crate::cards::creatures::grizzly_bears(), 0);
        let clone = put_in_graveyard(&mut game, crate::cards::phase_cv_cards::clone(), 0);
        let dp = RecordingDecisionProvider::picking(0);
        game.change_zone(clone, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp)).unwrap();
        assert!(dp.kinds()[0].starts_with("ChooseCopySource"), "{:?}", dp.kinds());

        assert_eq!(card_name(&game, clone), "Grizzly Bears");
        assert_eq!(format_permanent(&game, clone), "Grizzly Bears 2/2 (sick)");
    }
    #[test]
    fn the_log_names_a_copy_as_it_was_at_each_event() {
        use crate::engine::actions::{ActionContext, GameAction};
        use crate::test_support::{
            put_in_graveyard, put_on_battlefield, setup_two_player_game, test_ctx, RecordingDecisionProvider,
        };
        use crate::types::zones::ZoneChangeCause;

        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, crate::cards::creatures::grizzly_bears(), 0);
        let clone = put_in_graveyard(&mut game, crate::cards::phase_cv_cards::clone(), 0);
        let dp = RecordingDecisionProvider::picking(0);
        game.change_zone(clone, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp)).unwrap();
        game.execute_action(GameAction::Tap { object: clone }, &test_ctx()).unwrap();
        game.change_zone(clone, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx()).unwrap();
        // And one made a copy by a row, as Cytoshape does (CR 707.2), until
        // the row ends.
        let shaped = put_on_battlefield(&mut game, crate::test_support::vanilla_creature(1, 1, &[]), 0);
        let values = crate::engine::layers::copy::copiable_values(&game, bears).unwrap();
        let timestamp = game.allocate_timestamp();
        let row = crate::test_support::registered(
            shaped,
            crate::engine::layers::types::Layer::Layer1Copy,
            timestamp,
            crate::engine::layers::types::EffectModification::CopyFrom(Arc::new(values)),
        );
        let row = game.continuous_effects.add(row);
        game.execute_action(GameAction::Tap { object: shaped }, &test_ctx()).unwrap();
        game.continuous_effects.remove(row);
        game.execute_action(GameAction::Untap { object: shaped }, &test_ctx()).unwrap();

        // Formatted after it died: each line names it as it was then. A zone
        // change names the object as it was in the zone it left.
        let log = format_event_log(&game);
        let has = |line: String| assert!(log.contains(&line), "{line}\n{log:#?}");
        has(format!("ZoneChange: Clone ({clone}) [P0] Graveyard -> Battlefield [Returned]"));
        has(format!("ETB: Grizzly Bears (Clone, {clone}) [P0]"));
        has(format!("Tapped: Grizzly Bears (Clone, {clone})"));
        has(format!("ZoneChange: Grizzly Bears (Clone, {clone}) (Creature) [P0] Battlefield -> Graveyard [Destroyed]"));
        has(format!("Tapped: Grizzly Bears (Test Creature, {shaped})"));
        has(format!("Untapped: Test Creature ({shaped})"));
    }

    #[test]
    fn every_keyword_flag_prints_in_the_enums_order() {
        use crate::test_support::{put_on_battlefield, setup_two_player_game, vanilla_creature};

        let mut game = setup_two_player_game();
        let flags = [KeywordFlag::Shroud, KeywordFlag::Flying, KeywordFlag::Intimidate, KeywordFlag::Flash];
        let id = put_on_battlefield(&mut game, vanilla_creature(1, 1, &flags), 0);
        assert_eq!(format_permanent(&game, id), "Test Creature 1/1 [flash, flying, intimidate, shroud]");
    }

}
