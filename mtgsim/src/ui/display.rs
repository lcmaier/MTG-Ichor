// Text formatting helpers for CLI output and logging.
//
// All functions are pure formatters over &GameState — no mutations.
// Lives in ui/ because these are presentation helpers, not game-state queries.

use crate::objects::card_data::AbilityType;
use crate::oracle::characteristics::{get_effective_power, get_effective_toughness, has_keyword, is_creature};
use crate::state::game_state::{GameState, PhaseType, StepType};
use crate::types::card_types::CardType;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::keywords::KeywordAbility;

/// Format a card name with its ObjectId (short UUID suffix for disambiguation).
pub fn card_label(game: &GameState, id: ObjectId) -> String {
    match game.objects.get(&id) {
        Some(obj) => {
            let short_id = &format!("{}", id)[..8];
            format!("{} ({})", obj.card_data.name, short_id)
        }
        None => format!("<unknown {}>", &format!("{}", id)[..8]),
    }
}

/// Format a card name only (no ID).
pub fn card_name(game: &GameState, id: ObjectId) -> String {
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
    if entry.summoning_sick && is_creature(game, id) {
        let has_haste = has_keyword(game, id, KeywordAbility::Haste);
        if !has_haste {
            flags.push("sick");
        }
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

/// Collect displayable keyword names for a permanent.
fn collect_keywords(game: &GameState, id: ObjectId) -> Vec<&'static str> {
    let check = |kw: KeywordAbility, name: &'static str| -> Option<&'static str> {
        if has_keyword(game, id, kw) { Some(name) } else { None }
    };
    [
        check(KeywordAbility::Flying, "flying"),
        check(KeywordAbility::Reach, "reach"),
        check(KeywordAbility::Deathtouch, "deathtouch"),
        check(KeywordAbility::Lifelink, "lifelink"),
        check(KeywordAbility::FirstStrike, "first strike"),
        check(KeywordAbility::DoubleStrike, "double strike"),
        check(KeywordAbility::Trample, "trample"),
        check(KeywordAbility::Vigilance, "vigilance"),
        check(KeywordAbility::Haste, "haste"),
        check(KeywordAbility::Defender, "defender"),
        check(KeywordAbility::Hexproof, "hexproof"),
        check(KeywordAbility::Indestructible, "indestructible"),
        check(KeywordAbility::Menace, "menace"),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Format non-keyword abilities on a permanent for inline display.
///
/// Keywords are already shown via `collect_keywords` in a compact `[keyword, ...]`
/// block. This function handles the remaining ability types: activated, triggered,
/// static (non-keyword), and mana abilities. Each is shown as a short description.
///
/// For cards with `rules_text`, we use that as a fallback for abilities that don't
/// have a simple name. Long-term, a proper text template system will replace this.
fn format_abilities(game: &GameState, id: ObjectId) -> Vec<String> {
    let obj = match game.objects.get(&id) {
        Some(o) => o,
        None => return Vec::new(),
    };

    let mut lines = Vec::new();
    for (i, ability) in obj.card_data.abilities.iter().enumerate() {
        match ability.ability_type {
            // Mana abilities: show what they produce
            AbilityType::Mana => {
                if let crate::types::effects::Effect::Atom(
                    crate::types::effects::Primitive::ProduceMana(ref output),
                    _,
                ) = ability.effect
                {
                    let mana_str: Vec<String> = output.mana.iter()
                        .filter(|(_, amt)| **amt > 0)
                        .map(|(mt, amt)| {
                            let letter = match mt {
                                crate::types::mana::ManaType::White => "W",
                                crate::types::mana::ManaType::Blue => "U",
                                crate::types::mana::ManaType::Black => "B",
                                crate::types::mana::ManaType::Red => "R",
                                crate::types::mana::ManaType::Green => "G",
                                crate::types::mana::ManaType::Colorless => "C",
                            };
                            if *amt == 1 {
                                format!("{{{}}}", letter)
                            } else {
                                format!("{}{}", amt, letter)
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

    // If we have rules_text and no structured ability descriptions, show it
    // as a fallback. Even simple text like "{T}: Add {G}." is fine to display —
    // users reading CLI output can handle the redundancy.
    if lines.is_empty() && !obj.card_data.rules_text.is_empty() {
        lines.push(obj.card_data.rules_text.clone());
    }

    lines
}

/// Format a player's hand for display.
pub fn format_hand(game: &GameState, player_id: PlayerId) -> String {
    let player = match game.players.get(player_id) {
        Some(p) => p,
        None => return "Invalid player".to_string(),
    };

    if player.hand.is_empty() {
        return "  (empty)".to_string();
    }

    player.hand.iter()
        .enumerate()
        .map(|(i, &id)| {
            let obj = match game.objects.get(&id) {
                Some(o) => o,
                None => return format!("  {}: <unknown>", i),
            };
            let cost_str = obj.card_data.mana_cost.as_ref()
                .map(|c| format!(" {}", c))
                .unwrap_or_default();
            let pt_str = match (obj.card_data.power, obj.card_data.toughness) {
                (Some(p), Some(t)) => format!(" {}/{}", p, t),
                _ => String::new(),
            };
            format!("  {}: {}{}{}", i, obj.card_data.name, cost_str, pt_str)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format a player's battlefield for display, grouped by permanent type.
///
/// Groups: Creatures, Lands, Other (artifacts, enchantments, planeswalkers, etc.).
/// Each group is shown with a sub-header. Permanents are numbered sequentially
/// across groups so CLI index references remain unambiguous.
pub fn format_battlefield(game: &GameState, player_id: PlayerId) -> String {
    let perms: Vec<ObjectId> = game.battlefield.iter()
        .filter(|(_, e)| e.controller == player_id)
        .map(|(id, _)| *id)
        .collect();

    if perms.is_empty() {
        return "  (empty)".to_string();
    }

    let mut creatures: Vec<ObjectId> = Vec::new();
    let mut lands: Vec<ObjectId> = Vec::new();
    let mut other: Vec<ObjectId> = Vec::new();

    for &id in &perms {
        let obj = game.objects.get(&id);
        let is_land = obj.map(|o| o.card_data.types.contains(&CardType::Land)).unwrap_or(false);
        let is_creat = is_creature(game, id);
        if is_creat {
            creatures.push(id);
        } else if is_land {
            lands.push(id);
        } else {
            other.push(id);
        }
    }

    let mut lines = Vec::new();
    let mut idx = 0usize;

    if !creatures.is_empty() {
        lines.push("  Creatures:".to_string());
        for &id in &creatures {
            lines.push(format!("    {}: {}", idx, format_permanent(game, id)));
            idx += 1;
        }
    }
    if !lands.is_empty() {
        lines.push("  Lands:".to_string());
        for &id in &lands {
            lines.push(format!("    {}: {}", idx, format_permanent(game, id)));
            idx += 1;
        }
    }
    if !other.is_empty() {
        lines.push("  Other:".to_string());
        for &id in &other {
            lines.push(format!("    {}: {}", idx, format_permanent(game, id)));
            idx += 1;
        }
    }

    lines.join("\n")
}

/// Format the stack for display, with top/bottom markers.
pub fn format_stack(game: &GameState) -> String {
    if game.stack.is_empty() {
        return "  (empty)".to_string();
    }

    let count = game.stack.len();
    game.stack.iter().rev()
        .enumerate()
        .map(|(i, &id)| {
            let name = card_name(game, id);
            let controller = game.stack_entries.get(&id)
                .map(|e| format!(" (P{})", e.controller))
                .unwrap_or_default();
            let marker = if count == 1 {
                " <- top/bottom"
            } else if i == 0 {
                " <- top (resolves next)"
            } else if i == count - 1 {
                " <- bottom"
            } else {
                ""
            };
            format!("  {}: {}{}{}", i, name, controller, marker)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format the current phase/step for display.
pub fn format_phase(game: &GameState) -> String {
    let phase = match game.phase.phase_type {
        PhaseType::Beginning => "Beginning",
        PhaseType::Precombat => "Precombat Main",
        PhaseType::Combat => "Combat",
        PhaseType::Postcombat => "Postcombat Main",
        PhaseType::Ending => "Ending",
    };
    let step = game.phase.step.map(|s| match s {
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
    });

    match step {
        Some(s) => format!("{} — {}", phase, s),
        None => phase.to_string(),
    }
}

/// Format a summary line for a player (life, hand size, library size, graveyard size).
pub fn format_player_summary(game: &GameState, player_id: PlayerId) -> String {
    match game.players.get(player_id) {
        Some(p) => format!(
            "Player {} — Life: {} | Hand: {} | Library: {} | Graveyard: {}",
            player_id,
            p.life_total,
            p.hand.len(),
            p.library.len(),
            p.graveyard.len(),
        ),
        None => format!("Player {} — invalid", player_id),
    }
}

/// Format the mana pool for display.
pub fn format_mana_pool(game: &GameState, player_id: PlayerId) -> String {
    match game.players.get(player_id) {
        Some(p) => {
            let pool = p.mana_pool.available();
            if pool.is_empty() || pool.values().all(|&v| v == 0) {
                return "(empty)".to_string();
            }
            let mut parts = Vec::new();
            for (mt, &amount) in pool {
                if amount > 0 {
                    let letter = match mt {
                        crate::types::mana::ManaType::White => "W",
                        crate::types::mana::ManaType::Blue => "U",
                        crate::types::mana::ManaType::Black => "B",
                        crate::types::mana::ManaType::Red => "R",
                        crate::types::mana::ManaType::Green => "G",
                        crate::types::mana::ManaType::Colorless => "C",
                    };
                    parts.push(format!("{}{}", amount, letter));
                }
            }
            parts.join(" ")
        }
        None => "Invalid player".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Event log formatting
// ---------------------------------------------------------------------------

/// Resolve an ObjectId to "CardName (short-id)" for readable logs.
fn obj_name(game: &GameState, id: ObjectId) -> String {
    match game.objects.get(&id) {
        Some(obj) => {
            let short = &format!("{}", id)[..8];
            format!("{} ({})", obj.card_data.name, short)
        }
        None => format!("{}", id),
    }
}

/// Format a single GameEvent with resolved card names.
pub fn format_event(game: &GameState, event: &crate::events::event::GameEvent) -> String {
    use crate::events::event::GameEvent::*;
    match event {
        ZoneChange { object_id, owner, from, to } => {
            format!("ZoneChange: {} [P{}] {:?} -> {:?}", obj_name(game, *object_id), owner, from, to)
        }
        ManaAdded { player_id, source_id, mana } => {
            let mana_str: Vec<String> = mana.iter()
                .filter(|(_, v)| **v > 0)
                .map(|(t, v)| format!("{:?}:{}", t, v))
                .collect();
            format!("ManaAdded: P{} from {} [{}]", player_id, obj_name(game, *source_id), mana_str.join(", "))
        }
        DamageDealt { source_id, target, amount } => {
            let target_str = match target {
                crate::events::event::DamageTarget::Player(pid) => format!("P{}", pid),
                crate::events::event::DamageTarget::Object(oid) => obj_name(game, *oid),
            };
            format!("DamageDealt: {} -> {} for {}", obj_name(game, *source_id), target_str, amount)
        }
        PhaseBegin { phase } => format!("PhaseBegin: {:?}", phase),
        PhaseEnd { phase } => format!("PhaseEnd: {:?}", phase),
        StepBegin { step } => format!("StepBegin: {:?}", step),
        StepEnd { step } => format!("StepEnd: {:?}", step),
        TurnBegin { player, turn_number } => format!("TurnBegin: P{} turn {}", player, turn_number),
        TurnEnd { player, turn_number } => format!("TurnEnd: P{} turn {}", player, turn_number),
        PermanentEnteredBattlefield { object_id, controller } => {
            format!("ETB: {} [P{}]", obj_name(game, *object_id), controller)
        }
        PermanentLeftBattlefield { object_id } => {
            format!("LTB: {}", obj_name(game, *object_id))
        }
        LifeChanged { player_id, old, new } => {
            format!("LifeChanged: P{} {} -> {}", player_id, old, new)
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
        SpellResolved { spell_id } => {
            format!("SpellResolved: {}", obj_name(game, *spell_id))
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
        CreatureDied { creature_id, owner } => {
            format!("CreatureDied: {} [P{}]", obj_name(game, *creature_id), owner)
        }
        PlayerLost { player_id, reason } => {
            format!("PlayerLost: P{} ({:?})", player_id, reason)
        }
        StateBasedActionPerformed => "StateBasedActionPerformed".to_string(),
    }
}

/// Format the entire event log with resolved card names.
pub fn format_event_log(game: &GameState) -> Vec<String> {
    game.events.events().iter()
        .map(|e| format_event(game, e))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::BattlefieldEntity;
    use crate::state::game_state::{GameState, Phase};
    use crate::types::card_types::CardType;
    use crate::types::zones::Zone;

    #[test]
    fn test_card_name() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Forest").card_type(CardType::Land).build();
        let obj = GameObject::new(data, 0, Zone::Hand);
        let id = obj.id;
        game.add_object(obj);

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
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let mut entry = BattlefieldEntity::new(id, 0, ts);
        entry.summoning_sick = false;
        game.battlefield.insert(id, entry);

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
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let mut entry = BattlefieldEntity::new(id, 0, ts);
        entry.tapped = true;
        game.battlefield.insert(id, entry);

        let display = format_permanent(&game, id);
        assert!(display.contains("tapped"));
    }

    #[test]
    fn test_format_phase() {
        let mut game = GameState::new(2, 20);
        game.phase = Phase::new(PhaseType::Precombat);
        assert_eq!(format_phase(&game), "Precombat Main");

        game.phase = Phase::new(PhaseType::Beginning);
        assert!(format_phase(&game).contains("Untap"));
    }

    #[test]
    fn test_format_player_summary() {
        let game = GameState::new(2, 20);
        let summary = format_player_summary(&game, 0);
        assert!(summary.contains("Life: 20"));
        assert!(summary.contains("Hand: 0"));
    }

    #[test]
    fn test_format_stack_empty() {
        let game = GameState::new(2, 20);
        assert_eq!(format_stack(&game), "  (empty)");
    }

    #[test]
    fn test_format_battlefield_grouped() {
        use crate::types::card_types::*;
        use crate::types::mana::ManaType;

        let mut game = GameState::new(2, 20);

        // Add a creature
        let bears = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(bears, 0, Zone::Battlefield);
        let bears_id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let mut entry = BattlefieldEntity::new(bears_id, 0, ts);
        entry.summoning_sick = false;
        game.battlefield.insert(bears_id, entry);

        // Add a land
        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .mana_ability_single(ManaType::Green)
            .build();
        let obj = GameObject::new(forest, 0, Zone::Battlefield);
        let forest_id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let mut entry = BattlefieldEntity::new(forest_id, 0, ts);
        entry.summoning_sick = false;
        game.battlefield.insert(forest_id, entry);

        let output = format_battlefield(&game, 0);
        assert!(output.contains("Creatures:"), "Should have Creatures header");
        assert!(output.contains("Lands:"), "Should have Lands header");
        assert!(output.contains("Grizzly Bears"));
        assert!(output.contains("Forest"));
    }

    #[test]
    fn test_format_stack_single_item_marker() {
        use crate::state::game_state::StackEntry;
        use crate::types::effects::{Effect, Primitive, AmountExpr, TargetSpec, TargetCount};

        let mut game = GameState::new(2, 20);
        let bolt = CardDataBuilder::new("Lightning Bolt")
            .card_type(CardType::Instant)
            .build();
        let obj = GameObject::new(bolt, 0, Zone::Stack);
        let bolt_id = obj.id;
        game.add_object(obj);
        game.stack.push(bolt_id);
        game.stack_entries.insert(bolt_id, StackEntry {
            object_id: bolt_id,
            controller: 0,
            chosen_targets: Vec::new(),
            chosen_modes: Vec::new(),
            x_value: None,
            effect: Effect::Atom(Primitive::DealDamage(AmountExpr::Fixed(3)), TargetSpec::Any(TargetCount::Exactly(1))),
            is_spell: true,
        });

        let output = format_stack(&game);
        assert!(output.contains("top/bottom"), "Single item should show top/bottom marker");
    }

    #[test]
    fn test_format_stack_two_items_markers() {
        use crate::state::game_state::StackEntry;
        use crate::types::effects::{Effect, Primitive, AmountExpr, TargetSpec, TargetCount};

        let mut game = GameState::new(2, 20);

        let bolt = CardDataBuilder::new("Lightning Bolt")
            .card_type(CardType::Instant)
            .build();
        let obj = GameObject::new(bolt, 0, Zone::Stack);
        let bolt_id = obj.id;
        game.add_object(obj);
        game.stack.push(bolt_id);
        game.stack_entries.insert(bolt_id, StackEntry {
            object_id: bolt_id,
            controller: 0,
            chosen_targets: Vec::new(),
            chosen_modes: Vec::new(),
            x_value: None,
            effect: Effect::Atom(Primitive::DealDamage(AmountExpr::Fixed(3)), TargetSpec::Any(TargetCount::Exactly(1))),
            is_spell: true,
        });

        let recall = CardDataBuilder::new("Ancestral Recall")
            .card_type(CardType::Instant)
            .build();
        let obj2 = GameObject::new(recall, 0, Zone::Stack);
        let recall_id = obj2.id;
        game.add_object(obj2);
        game.stack.push(recall_id);
        game.stack_entries.insert(recall_id, StackEntry {
            object_id: recall_id,
            controller: 0,
            chosen_targets: Vec::new(),
            chosen_modes: Vec::new(),
            x_value: None,
            effect: Effect::Atom(Primitive::DealDamage(AmountExpr::Fixed(3)), TargetSpec::Any(TargetCount::Exactly(1))),
            is_spell: true,
        });

        let output = format_stack(&game);
        assert!(output.contains("top (resolves next)"), "Top item should have resolves-next marker");
        assert!(output.contains("bottom"), "Bottom item should have bottom marker");
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
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let mut entry = BattlefieldEntity::new(id, 0, ts);
        entry.summoning_sick = false;
        game.battlefield.insert(id, entry);

        let display = format_permanent(&game, id);
        assert!(display.contains("mana: Add"), "Should show mana ability");
        assert!(display.contains("{G}"), "Should show green mana");
    }
}
