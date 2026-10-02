//! The board as owned data, built on the engine thread at each prompt.
//!
//! Characteristics come off the layer system's output, `compute_characteristics`
//! — the frame `oracle/characteristics.rs` wraps a field at a time — and never
//! off `card_data` (`CLAUDE.md`'s layer invariant). One call per object gives
//! every field, including the two with no wrapper: mana cost and the keyword set.

use std::collections::HashMap;
use std::sync::Arc;

use mtgsim::engine::layers::compute::compute_characteristics;
use mtgsim::engine::layers::types::EffectiveCharacteristics;
use mtgsim::engine::resolve::ResolvedTarget;
use mtgsim::oracle::characteristics::has_summoning_sickness;
use mtgsim::scenario::Scenario;
use mtgsim::state::game_state::GameState;
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::CounterType;
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::{ManaSymbol, ManaType};
use mtgsim::ui::display::{
    TypeLine, TypeWord, TypeWordStatus, attack_target_name, format_event, format_permanent, format_phase, keyword_name,
    named, player_name, printed_faces, type_line_now,
};

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub turn: u32,
    pub active_player: PlayerId,
    pub phase: String,
    /// In seat order, so four seats are a longer list rather than a new shape.
    pub players: Vec<PlayerView>,
    /// Top first.
    pub stack: Vec<StackItem>,
    pub pending_triggers: Vec<PendingTriggerView>,
    pub exile: Vec<CardView>,
    pub command: Vec<CardView>,
    /// `format_event` lines for what happened since the previous snapshot.
    pub log: Vec<String>,
    /// How many recorded events the window's log holds, where the next
    /// snapshot's starts.
    pub events_logged: usize,
    /// The board as a scenario file, with what it could not write at the top
    /// (`Scenario::write`), for "Save board as scenario".
    pub board_text: String,
}

#[derive(Clone, Debug)]
pub struct PlayerView {
    pub id: PlayerId,
    pub life: i64,
    /// `({R}, 2)`, in WUBRGC order.
    pub mana_pool: Vec<(String, u64)>,
    pub counters: Vec<(String, u32)>,
    pub lost: bool,
    pub hand: Vec<CardView>,
    /// Top first.
    pub library: Vec<CardView>,
    /// Top first.
    pub graveyard: Vec<CardView>,
    /// What this player controls, in timestamp order.
    pub battlefield: Vec<PermanentView>,
}

#[derive(Clone, Debug)]
pub struct CardView {
    pub id: ObjectId,
    pub owner: PlayerId,
    pub name: String,
    pub mana_cost: Option<String>,
    pub type_line: Arc<TypeLineView>,
    /// `ui::display::printed_faces`: the card before any effect, a face an
    /// entry. Shared, since the views are built again at every repaint.
    pub printed: Arc<[String]>,
}

/// `ui::display::type_line_now` as the window draws it: the words in order,
/// a dash before the subtypes, and a word an effect took away faded where
/// it stood. `text` is the line the object has now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeLineView {
    pub front: Vec<TypeWordView>,
    pub subtypes: Vec<TypeWordView>,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeWordView {
    pub text: String,
    pub faded: bool,
}

impl TypeLineView {
    fn of(line: &TypeLine) -> TypeLineView {
        let words = |section: &[TypeWord]| {
            section
                .iter()
                .map(|word| TypeWordView { text: word.text.clone(), faded: word.status == TypeWordStatus::Lost })
                .collect()
        };
        TypeLineView { front: words(&line.front), subtypes: words(&line.subtypes), text: line.to_string() }
    }
}

#[derive(Clone, Debug)]
pub struct PermanentView {
    pub card: CardView,
    pub controller: PlayerId,
    pub is_creature: bool,
    pub is_land: bool,
    /// Creatures only: the layer output still carries a noncreature
    /// permanent's printed numbers, which CR 208.3 says it does not have.
    pub power_toughness: Option<(i32, i32)>,
    pub damage: u32,
    pub keywords: Vec<String>,
    pub counters: Vec<(String, u32)>,
    pub tapped: bool,
    pub summoning_sick: bool,
    /// What it attacks, by name.
    pub attacking: Option<String>,
    /// The attackers it blocks, by name.
    pub blocking: Vec<String>,
    /// What it is attached to, by name.
    pub attached_to: Option<String>,
    pub phased_out: bool,
    pub face_down: bool,
    /// `ui::display::format_permanent`'s lines: the status, then each
    /// ability's own text in the effective list's order.
    pub engine_text: String,
}

#[derive(Clone, Debug)]
pub struct StackItem {
    pub id: ObjectId,
    pub name: String,
    /// `None` until the stack entry is written: a spell or ability still being
    /// cast or activated (CR 601.2, 602.2), which is when most prompts come.
    pub is_spell: Option<bool>,
    pub controller: PlayerId,
    pub targets: Vec<String>,
    pub x: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct PendingTriggerView {
    pub controller: PlayerId,
    pub source: String,
}

impl Snapshot {
    /// The board now, with the log from event `events_logged` on.
    pub fn build(game: &GameState, events_logged: usize) -> Snapshot {
        let recorded = game.recorded_events();
        let log = recorded
            .records_from(events_logged)
            .iter()
            .map(|record| format_event(game, &record.event, &record.names))
            .collect();
        let permanents: Vec<PermanentView> = game
            .battlefield_ids_ordered()
            .into_iter()
            .filter_map(|id| permanent(game, id))
            .collect();
        let players = game
            .players
            .iter()
            .map(|player| PlayerView {
                id: player.id,
                life: player.life_total,
                mana_pool: mana_pool(player.mana_pool.available()),
                counters: player.counters.iter().map(|(kind, n)| (kind.name().to_string(), *n)).collect(),
                lost: game.player_lost.get(player.id).copied().unwrap_or(false),
                hand: cards(game, &player.hand),
                library: cards(game, player.library.iter().rev()),
                graveyard: cards(game, player.graveyard.iter().rev()),
                battlefield: permanents.iter().filter(|p| p.controller == player.id).cloned().collect(),
            })
            .collect();
        Snapshot {
            turn: game.turn_number,
            active_player: game.active_player,
            phase: format_phase(game),
            players,
            stack: game.stack.iter().rev().map(|id| stack_item(game, *id)).collect(),
            pending_triggers: game
                .pending_triggers
                .iter()
                .map(|trigger| PendingTriggerView {
                    controller: trigger.controller,
                    source: named(game, trigger.origin.source()),
                })
                .collect(),
            exile: cards(game, &game.exile),
            command: cards(game, &game.command),
            log,
            events_logged: recorded.len(),
            board_text: Scenario::write(game).to_string(),
        }
    }
}

fn cards<'a>(game: &GameState, ids: impl IntoIterator<Item = &'a ObjectId>) -> Vec<CardView> {
    ids.into_iter().filter_map(|id| card(game, *id)).collect()
}

fn card(game: &GameState, id: ObjectId) -> Option<CardView> {
    let chars = compute_characteristics(game, id)?;
    card_with(game, id, &chars)
}

fn card_with(game: &GameState, id: ObjectId, chars: &EffectiveCharacteristics) -> Option<CardView> {
    Some(CardView {
        id,
        owner: game.objects.get(&id)?.owner,
        name: chars.name.clone(),
        mana_cost: chars.mana_cost.as_ref().map(ToString::to_string),
        type_line: Arc::new(TypeLineView::of(&type_line_now(game, id))),
        printed: printed_faces(game, id).into(),
    })
}

fn permanent(game: &GameState, id: ObjectId) -> Option<PermanentView> {
    let state = game.battlefield.get(&id)?;
    let chars = compute_characteristics(game, id)?;
    let is_creature = chars.types.contains(&CardType::Creature);
    let mut keywords: Vec<_> = chars.keyword_flags.iter().copied().collect();
    keywords.sort();
    let mut counters: Vec<(CounterType, u32)> =
        state.counters.iter().map(|(kind, stack)| (*kind, stack.count)).collect();
    counters.sort();
    Some(PermanentView {
        card: card_with(game, id, &chars)?,
        controller: chars.controller,
        is_creature,
        is_land: chars.types.contains(&CardType::Land),
        power_toughness: if is_creature { chars.power.zip(chars.toughness) } else { None },
        damage: state.damage_marked,
        keywords: keywords.into_iter().map(|k| keyword_name(k).to_string()).collect(),
        counters: counters.into_iter().map(|(kind, n)| (kind.name().to_string(), n)).collect(),
        tapped: state.tapped,
        summoning_sick: has_summoning_sickness(game, id),
        attacking: state.attacking.as_ref().map(|a| attack_target_name(game, &a.target)),
        blocking: state.blocking.iter().flat_map(|b| &b.blocking).map(|attacker| named(game, *attacker)).collect(),
        attached_to: state.attached_to.map(|host| named(game, host)),
        phased_out: state.phased_out,
        face_down: state.face_down,
        engine_text: format_permanent(game, id),
    })
}

fn stack_item(game: &GameState, id: ObjectId) -> StackItem {
    let chars = compute_characteristics(game, id);
    let entry = game.stack_entries.get(&id);
    StackItem {
        id,
        name: chars.as_ref().map_or_else(|| id.to_string(), |c| c.name.clone()),
        is_spell: entry.map(|e| e.is_spell),
        controller: chars.as_ref().map(|c| c.controller).or(entry.map(|e| e.controller)).unwrap_or_default(),
        targets: entry
            .map(|e| e.chosen_targets.iter().flat_map(|t| &t.chosen).map(|t| target_name(game, t)).collect())
            .unwrap_or_default(),
        x: entry.and_then(|e| e.x_value),
    }
}

fn mana_pool(pool: &HashMap<ManaType, u64>) -> Vec<(String, u64)> {
    let mut entries: Vec<(ManaType, u64)> = pool.iter().filter(|(_, n)| **n > 0).map(|(t, n)| (*t, *n)).collect();
    entries.sort_by_key(|(mana, _)| wubrgc_rank(*mana));
    entries.into_iter().map(|(mana, n)| (ManaSymbol::Colored(mana).to_string(), n)).collect()
}

fn wubrgc_rank(mana: ManaType) -> u8 {
    match mana {
        ManaType::White => 0,
        ManaType::Blue => 1,
        ManaType::Black => 2,
        ManaType::Red => 3,
        ManaType::Green => 4,
        ManaType::Colorless => 5,
    }
}

fn target_name(game: &GameState, target: &ResolvedTarget) -> String {
    match target {
        ResolvedTarget::Object(id) => named(game, *id),
        ResolvedTarget::Player(player) => player_name(*player),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mana_pool_reads_in_wubrgc_order_whatever_the_hash_order() {
        let pool = HashMap::from([
            (ManaType::Colorless, 1),
            (ManaType::Green, 2),
            (ManaType::White, 1),
            (ManaType::Red, 0),
        ]);
        let symbols: Vec<String> = mana_pool(&pool).into_iter().map(|(s, _)| s).collect();
        assert_eq!(symbols, ["{W}", "{G}", "{C}"]);
    }
}
