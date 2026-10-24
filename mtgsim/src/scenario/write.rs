//! A game's state written as a [`Scenario`] (`setup-architecture.md` §4.3).
//!
//! Every field of `GameState`, `PlayerState`, `PermanentState` and
//! `GameObject` is named below with no `..`, so a field the engine adds fails
//! to compile here until it is answered (§5.2's growth contract): written as
//! a word, reported in [`WrittenBoard::unwritten`], or bound to `_` as the
//! engine's own, with the reason beside it.

use super::board::{Arrival, Attacked, CardLine, CardWord, LineKind, LineNumbered, NamedCard, PlayerWord, Scenario};
use super::text::{draw_step_skipped, every_turn_fact, in_combat_from};
use crate::engine::layers::compute::seed_frame;
use crate::state::history_log::{RowFact, FIRST_TURN};
use crate::engine::layers::intrinsic::intrinsic_entry_counter_kind;
use crate::engine::layers::types::EffectOrigin;
use crate::objects::object::GameObject;
use crate::state::battlefield::{AttackTarget, CostChoices, PermanentState};
use crate::state::game_state::{GameState, StepType, TurnPlan};
use crate::state::player::PlayerState;
use crate::types::history::{HistorySpan, TurnFact};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::zones::Zone;

/// A board written from a game, and what it could not write.
#[derive(Debug, Clone)]
pub struct WrittenBoard {
    pub scenario: Scenario,
    /// What §2 plays rather than writes, and the fields whose word waits
    /// (§5.2), a sentence each. Empty when the board is exact.
    pub unwritten: Vec<String>,
}

impl std::fmt::Display for WrittenBoard {
    /// The report as comments at the top, then the board.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in &self.unwritten {
            writeln!(f, "# not written: {line}")?;
        }
        write!(f, "{}", self.scenario)
    }
}

impl Scenario {
    /// The board `state` shows, resuming at the start of its step's priority
    /// round with the active player to act. Its randomness is fresh: a
    /// `StdRng` keeps its position private, so the save, not this, continues
    /// a game exactly.
    pub fn write(state: &GameState) -> WrittenBoard {
        let mut writer = Writer { state, scenario: Scenario::default(), unwritten: Vec::new(), tags: Vec::new() };
        writer.assign_tags();
        writer.write_game();
        writer.write_players();
        writer.write_cards();
        WrittenBoard { scenario: writer.scenario, unwritten: writer.unwritten }
    }
}

struct Writer<'g> {
    state: &'g GameState,
    scenario: Scenario,
    unwritten: Vec<String>,
    /// The objects that need a tag, and theirs.
    tags: Vec<(ObjectId, String)>,
}

fn unnumbered<T>(value: T) -> LineNumbered<T> {
    LineNumbered { line: 0, value }
}

impl<'g> Writer<'g> {
    fn report(&mut self, what: impl Into<String>) {
        self.unwritten.push(what.into());
    }

    fn write_game(&mut self) {
        let GameState {
            objects: _,             // by zone, in `cards`
            players,
            stack,
            stack_entries: _,       // with the stack
            resolving,
            battlefield: _,         // in `cards`
            diagnostics: _,         // the engine's count of its own work
            layer_epoch: _,         // the memo's key: engine's own
            layer_memo: _,
            exile: _,               // in `cards`
            command: _,
            turn_number,
            last_turn_began,
            active_player,
            priority_player,
            phase,
            turn_queue,
            extra_turn,
            next_extra_turn_id: _, // a counter
            turn_plan,
            turn_rotation,
            combat_had_attackers,
            blockers_declared,
            blocker_damage_divisions,
            dealt_first_strike_damage: _, // on each permanent's line
            next_timestamp: _,      // the counters ids and stamps come from
            next_object_id: _,
            player_lost: _, // each player's line, and the rotation below
            result,
            starting_life,
            skip_first_draw,
            continuous_effects,
            replacement_effects,
            replacement_ability_sources: _, // the gates, rebuilt by the doors
            zone_replacement_ability_sources: _,
            restrictions,
            restriction_ability_sources: _,
            cost_modification_ability_sources: _,
            trigger_multiplier_sources: _,
            entry_selection: _,     // a batch's, empty at any prompt outside one
            prevention_allocations: _,
            tokens_being_created: _, // a creation's, empty at any prompt outside one
            lethal_damage_left: _,
            nesting: _,
            rider_lineage: _,
            next_zone_change_epoch: _, // CR 704.6d's ticks, settled at rest
            arrivals_since_last_sba_check: _, // empty at rest
            #[cfg(debug_assertions)]
            last_sba_check_epoch: _,
            pending_triggers,
            next_trigger_seq: _,
            delayed_triggers,
            until_returns,
            next_delayed_trigger_id: _, // a counter
            next_until_return_id: _,
            history,                // the players' rows in `write_players`, the abilities' `this turn:` lines in `cards`; the combat's is a report
            trigger_sources: _,     // the dispatcher's gates
            zone_trigger_sources: _,
            newcomer_trigger_sources: _,
            look_back_snapshots: _, // a window's, empty outside one
            control_effects_at_last_sweep: _, // the control sweep's, settled at load
            settled_layer_epoch: _,           // the settlement's gate
            lki_captures: _,
            lki_store: _,           // what left before the board was written, which a scenario states none of, as its rows keep no frames
            dispatch_audit: _,
            events: _,
            trace: _,
            decision_log: _,        // an observer's, as the trace is
            stopped: _,             // the run's: a stopped game writes as its prompt showed it
            rng: _,                 // private to `StdRng`: a written board's randomness is fresh
        } = self.state;
        let (turn, active) = (*turn_number, *active_player);
        self.scenario.players = players.len();
        self.scenario.starting_life = *starting_life;
        self.scenario.turn = turn;
        self.scenario.active = active;
        self.scenario.step = *phase;
        if matches!(phase.step, Some(StepType::Untap | StepType::Cleanup)) {
            self.report("the untap or cleanup step, where no player receives priority (CR 502.4, 514.3)");
        }
        if *priority_player != active {
            self.report(format!("a prompt partway through a priority round: the board resumes at its start, player {active} to act"));
        }
        if !stack.is_empty() || resolving.is_some() {
            self.report(format!("the stack ({} objects): play it from the board", stack.len()));
        }
        if !pending_triggers.is_empty() {
            self.report(format!("{} triggered abilities waiting to be put on the stack", pending_triggers.len()));
        }
        if !delayed_triggers.is_empty() {
            self.report(format!("{} delayed triggered abilities waiting for their events (CR 603.7)", delayed_triggers.len()));
        }
        if !until_returns.is_empty() {
            self.report(format!("{} \"until\" returns waiting for their events (CR 610.3)", until_returns.len()));
        }
        if !turn_queue.is_empty() || extra_turn.is_some() || *turn_rotation != active {
            self.report("an extra turn (CR 500.7)");
        }
        if turn_plan.phases != TurnPlan::natural().phases {
            self.report("an extra phase (CR 500.8)");
        }
        // The loader begins the natural rotation's turns over the players in
        // the game, ending with the active player's (`build.rs`).
        let in_game: Vec<PlayerId> = self.state.players_in_game().collect();
        let mut natural = vec![0; players.len()];
        if let Some(place) = in_game.iter().position(|&p| p == active) {
            for back in 0..(turn as usize).min(in_game.len()) {
                natural[in_game[(place + in_game.len() - back) % in_game.len()]] = turn - back as u32;
            }
        }
        if *last_turn_began != natural {
            self.report("turns that left the natural rotation (an extra or a skipped turn), which CR 302.6 reads");
        }
        // ORDER-FREE: whether any is attacking.
        let attacked = self.state.battlefield.values().any(|e| e.attacking.is_some())
            || history.count(active, TurnFact::AttackersDeclared, turn..=turn) > 0;
        if *combat_had_attackers != (attacked && in_combat_from(*phase, StepType::DeclareAttackers))
            || *blockers_declared != (attacked && in_combat_from(*phase, StepType::DeclareBlockers))
        {
            self.report("whether attackers or blockers were declared, which the board does not show (CR 508.8)");
        }
        if !blocker_damage_divisions.is_empty() {
            self.report("a blocker's division of combat damage (CR 510.1d)");
        }
        // A board is played through its turn rather than saved with each
        // creature's part in the turn's combats.
        let this_turn = || history.log.rows_of_turn(turn).map(|row| row.fact);
        if this_turn().any(|fact| matches!(fact, RowFact::Attacked(_) | RowFact::Blocked(_) | RowFact::RemovedFromCombat { .. })) {
            self.report("what creatures did in combat this turn (CR 508.4, 508.5)");
        }
        if this_turn().any(|fact| matches!(fact, RowFact::BecameTarget { .. })) {
            self.report("what became the target of a spell or ability this turn (CR 115.1)");
        }
        let first_draw = phase.step == Some(StepType::Upkeep) && draw_step_skipped(self.state, turn);
        if *skip_first_draw != first_draw {
            self.report("a first draw step skipped by override, not by CR 103.8");
        }
        if result.is_some() {
            self.report("the game is over");
        }
        let resolved = continuous_effects.iter().filter(|row| row.origin == EffectOrigin::Resolution).count();
        if resolved > 0 {
            self.report(format!("{resolved} continuous effects a resolution created (a pump, a control change): play them from the board"));
        }
        if !replacement_effects.is_empty() || !restrictions.is_empty() {
            self.report("a resolution's shield, prevention or \"can't\" effect: play it from the board");
        }
    }

    fn write_players(&mut self) {
        let state = self.state;
        let turn = state.turn_number;
        for (p, (player, history)) in state.players.iter().zip(&state.history.players).enumerate() {
            let PlayerState {
                id: _,              // the seat
                life_total,
                mana_pool,
                library: _,         // in `cards`
                hand: _,
                graveyard: _,
                max_hand_size,
                lands_per_turn,
                lands_played_this_turn,
                counters,
                commander_damage_taken,
                has_drawn_from_empty_library,
            } = player;
            let mut facts = Vec::new();
            if *life_total != state.starting_life {
                facts.push(PlayerWord::Life { player: p, life: *life_total });
            }
            facts.extend(counters.iter().map(|(kind, count)| PlayerWord::Counter { player: p, kind: *kind, count: *count }));
            if *lands_played_this_turn > 0 {
                facts.push(PlayerWord::LandsPlayed { player: p, count: *lands_played_this_turn });
            }
            if state.in_own_draw_step(p) && history.this_draw_step() > 0 {
                facts.push(PlayerWord::DrawnThisDrawStep { player: p, count: history.this_draw_step() });
            }
            if !state.in_game(p) {
                facts.push(PlayerWord::LeftTheGame { player: p });
            }
            let mut damage: Vec<(&ObjectId, &u32)> = commander_damage_taken.iter().collect();
            damage.sort_by_key(|(id, _)| state.object_timestamp(**id));
            for (commander, damage) in damage {
                match state.objects.get(commander) {
                    Some(obj) => facts.push(PlayerWord::CommanderDamage { player: p, damage: *damage, from: self.named_card(obj) }),
                    None => self.report(format!("player {p}'s commander damage from a commander no longer in the game")),
                }
            }
            let casts = self.named_casts(p);
            let named = |span| casts.iter().filter(|word| matches!(word, PlayerWord::Cast { span: named, .. } if *named == span)).count() as u64;
            let (named_this, named_last, named_earlier) =
                (named(HistorySpan::ThisTurn), named(HistorySpan::LastTurn), named(HistorySpan::ThisGame));
            for fact in every_turn_fact() {
                let count = |span| state.history.count(p, fact, state.history.turns_of(span, p, turn));
                let (mut this, mut last, mut game) = (count(HistorySpan::ThisTurn), count(HistorySpan::LastTurn), count(HistorySpan::ThisGame));
                // The named casts are said by name; the count says the rest.
                if fact == TurnFact::SpellsCast {
                    this -= named_this;
                    last -= named_last;
                    game -= named_this + named_last + named_earlier;
                }
                for (span, count) in [(HistorySpan::ThisTurn, this), (HistorySpan::LastTurn, last)] {
                    if count > 0 {
                        facts.push(PlayerWord::History { player: p, span, fact, count });
                    }
                }
                if game != this.saturating_add(last) {
                    facts.push(PlayerWord::History { player: p, span: HistorySpan::ThisGame, fact, count: game });
                }
            }
            facts.extend(casts);
            self.scenario.player_words.extend(facts.into_iter().map(unnumbered));
            if mana_pool.total() > 0 || !mana_pool.special_atoms().is_empty() {
                self.report(format!("player {p}'s mana pool (the word waits for item 33's provenance)"));
            }
            if *max_hand_size != 7 || *lands_per_turn != 1 {
                self.report(format!("player {p}'s maximum hand size or land plays per turn"));
            }
            if *has_drawn_from_empty_library {
                self.report(format!("player {p}'s draw from an empty library, not yet checked (CR 704.5b)"));
            }
            if self.since_your_last_turn_differs(p) {
                self.report(format!("player {p}'s counts since their last turn, which the loader derives from the rows"));
            }
        }
    }

    /// `player`'s casts the log keeps a frame for, oldest first, each in the
    /// span a save states it in: this turn, last turn, or the rest as "this
    /// game". A cast the name would not reload as it was is reported.
    fn named_casts(&mut self, player: PlayerId) -> Vec<PlayerWord> {
        let state = self.state;
        let turn = state.turn_number;
        let mut casts = Vec::new();
        for row in state.history.log.rows_of_turns(FIRST_TURN..=turn) {
            let (RowFact::Cast { from }, Some(frame)) = (row.fact, row.frame.characteristics()) else { continue };
            if row.player != Some(player) {
                continue;
            }
            let span = match turn - row.turn {
                0 => HistorySpan::ThisTurn,
                1 => HistorySpan::LastTurn,
                _ => HistorySpan::ThisGame,
            };
            if from != Zone::Hand {
                self.report(format!("player {player}'s spell cast from a {from:?} zone, which a named cast does not say"));
            }
            // AS PRINTED: a named cast reloads as its card's printed frame. A
            // row with no object is a loaded named cast, which is that frame.
            let reloads_as_cast = row.object.map(|object| {
                state.objects.get(&object.id).is_some_and(|obj| *frame == seed_frame(&obj.card_data, frame.controller, frame.control_since_turn))
            });
            if reloads_as_cast == Some(false) {
                self.report(format!("player {player}'s {} as it was cast, which its printed card does not say", frame.name));
            }
            casts.push(PlayerWord::Cast { player, span, card: frame.name.clone() });
        }
        casts.reverse();
        casts
    }

    /// Does `player`'s "since your last turn" differ from what the loader
    /// derives? It lands the rows on turns T and T-1 and the rest of "this
    /// game" on T-2, so a player whose last turn ended at T-1 or T-2 reads
    /// exactly, and one whose ended earlier, or who has had none, reads it all.
    fn since_your_last_turn_differs(&self, player: PlayerId) -> bool {
        let state = self.state;
        let turn = state.turn_number;
        let in_game = state.players_in_game().count();
        let ended = match state.last_turn_began[player] {
            began if player == state.active_player => (began as usize).saturating_sub(in_game) as u32,
            began => began,
        };
        let histories = &state.history;
        let since = histories.since_your_last_turn(player, turn);
        (0..histories.players.len()).any(|q| {
            every_turn_fact().any(|fact| {
                let count = |turns| histories.count(q, fact, turns);
                let spanned = |span| count(histories.turns_of(span, q, turn));
                let (this, last) = (spanned(HistorySpan::ThisTurn), spanned(HistorySpan::LastTurn));
                let derived = match turn.checked_sub(ended) {
                    Some(1) if ended > 0 => this,
                    Some(2) if ended > 0 => this.saturating_add(last),
                    _ => spanned(HistorySpan::ThisGame),
                };
                count(since.clone()) != derived
            })
        })
    }

    /// `obj`'s name, with its tag if two objects it could be confused with
    /// share it.
    fn named_card(&self, obj: &GameObject) -> NamedCard {
        let tag = self.tags.iter().find(|(id, _)| *id == obj.id).map(|(_, tag)| tag.clone());
        NamedCard { name: obj.card_data.name.clone(), tag }
    }

    /// A tag for each name two permanents share, or two commanders, as the
    /// loader resolves a reference among the one or the other.
    fn assign_tags(&mut self) {
        let state = self.state;
        let permanents = state.battlefield_ids_ordered();
        // ORDER-FREE: sorted by timestamp next.
        let mut commanders: Vec<ObjectId> = state.objects.values().filter(|o| o.is_commander).map(|o| o.id).collect();
        commanders.sort_by_key(|&id| state.object_timestamp(id));
        for group in [&permanents, &commanders] {
            for &id in group.iter() {
                let name = &state.objects[&id].card_data.name;
                let shared = group.iter().filter(|other| state.objects[other].card_data.name == *name).count() > 1;
                if shared && !self.tags.iter().any(|(tagged, _)| *tagged == id) {
                    let used = self.tags.iter().filter(|(other, _)| state.objects[other].card_data.name == *name).count();
                    self.tags.push((id, tag_letters(used)));
                }
            }
        }
    }

    fn write_cards(&mut self) {
        let state = self.state;
        let permanents = state.battlefield_ids_ordered();
        // Each library top first, then everything else in CR 613.7d's order,
        // a permanent's counters folded into its line unless something was
        // stamped between them (CR 613.7c).
        for (p, player) in state.players.iter().enumerate() {
            for id in player.library.iter().rev() {
                let line = CardLine { kind: LineKind::Library { player: p, shuffled: false }, ..self.object_line(*id) };
                self.push_line(line);
            }
        }
        let mut stamps: Vec<(u64, ObjectId, Option<crate::types::effects::CounterType>)> = Vec::new();
        for player in &state.players {
            stamps.extend(player.hand.iter().chain(&player.graveyard).map(|&id| (state.object_timestamp(id), id, None)));
        }
        stamps.extend(state.exile.iter().chain(&state.command).chain(&permanents).map(|&id| (state.object_timestamp(id), id, None)));
        for &id in &permanents {
            stamps.extend(state.battlefield[&id].counters.iter().map(|(kind, stack)| (stack.timestamp, id, Some(*kind))));
        }
        stamps.sort_by_key(|&(stamp, _, _)| stamp);
        let mut references: Vec<(usize, CardWord)> = Vec::new();
        let mut last: Option<ObjectId> = None;
        for (_, id, counter) in stamps {
            let Some(kind) = counter else {
                let (line, reference) = self.permanent_line(id);
                if let Some(word) = reference {
                    references.push((self.scenario.cards.len(), word));
                }
                self.push_line(line);
                last = Some(id);
                continue;
            };
            let count = CardWord::Counter(kind, state.battlefield[&id].counter_count(kind));
            if last == Some(id) {
                if let Some(line) = self.scenario.cards.last_mut() {
                    line.value.words.push(count);
                }
            } else {
                let card = self.named_card(&state.objects[&id]);
                if !self.scenario.cards.iter().any(|line| line.value.card == card) {
                    self.report(format!("{card}'s {} counters, older than its timestamp", kind.name()));
                }
                self.push_line(CardLine { kind: LineKind::Counters, card, copies: 1, words: vec![count] });
                last = Some(id);
            }
        }
        for (index, word) in references {
            self.scenario.cards[index].value.words.push(word);
        }
        self.write_this_turn_counts();
    }

    /// `line`, folded into the line before it when the two say the same of
    /// two cards in one zone.
    fn push_line(&mut self, line: CardLine) {
        if let Some(previous) = self.scenario.cards.last_mut()
            && line.kind != LineKind::Battlefield
            && line.kind != LineKind::Counters
            && previous.value.kind == line.kind
            && previous.value.card == line.card
            && previous.value.words == line.words
        {
            previous.value.copies += 1;
            return;
        }
        self.scenario.cards.push(unnumbered(line));
    }

    /// The object's line in its zone, with the `GameObject` words.
    fn object_line(&mut self, id: ObjectId) -> CardLine {
        let GameObject {
            id: _,                  // minted in file order by the loader
            owner,
            card_data: _,           // the name, through `reference`
            zone,
            is_token,
            is_copy,
            is_commander,
            zone_change_epoch: _,   // CR 704.6d's tick: settled at rest
            timestamp: _,           // the file's order
        } = &self.state.objects[&id];
        let obj = &self.state.objects[&id];
        let card = self.named_card(obj);
        // Its line names it as a card, and no registry holds a token's name.
        if *is_token {
            self.report(format!("that {card} is a token (§5.2): the loader refuses its line, so remove it and play what makes the token"));
        }
        if *is_copy {
            self.report(format!("that {card} is a copy (§5.2)"));
        }
        let mut words = Vec::new();
        let kind = match zone {
            Zone::Hand => LineKind::Hand(*owner),
            Zone::Library => LineKind::Library { player: *owner, shuffled: false },
            Zone::Graveyard => LineKind::Graveyard(*owner),
            Zone::Exile | Zone::Command => {
                words.push(CardWord::Owner(*owner));
                if *zone == Zone::Exile { LineKind::Exile } else { LineKind::Command }
            }
            Zone::Battlefield | Zone::Stack => LineKind::Battlefield,
        };
        if *is_commander {
            words.push(CardWord::Commander);
        }
        CardLine { kind, card, copies: 1, words }
    }

    /// A permanent's line, or any other object's, and the one word naming
    /// another card that the line ends with, added once its counters are.
    fn permanent_line(&mut self, id: ObjectId) -> (CardLine, Option<CardWord>) {
        let mut line = self.object_line(id);
        let state = self.state;
        let Some(entry) = state.battlefield.get(&id) else { return (line, None) };
        let PermanentState {
            object_id: _,
            controller,
            timestamp: _,           // the file's order
            tapped,
            flipped,
            face_down,
            phased_out,
            entered_battlefield_turn,
            controller_since_turn,
            damage_marked,
            damaged_by_deathtouch,
            attacking,
            blocking,
            counters: _,            // stamped at their place, in `cards`
            x_value: _,             // read only as the entry is dispatched (CR 107.3f, 400.7d)
            attached_to,
            attached_by: _,         // each attachment's `attached to`
            cast: _,
            cost_choices,
            entered_as,
            announced_controller: _, // every change is announced at a prompt; settled at load
            frame_before_control_change: _,
        } = entry;
        let obj = &state.objects[&id];
        let card = line.card.to_string();
        let mut words = CardWord::seats(*controller, obj.owner);
        words.append(&mut line.words);
        if *tapped {
            words.push(CardWord::Tapped);
        }
        // CR 302.6 reads the arrival only against the controller's most
        // recent turn; an arrival before it is the default's answer.
        let arrived = *entered_battlefield_turn;
        let since = state.most_recent_turn_began(*controller).unwrap_or(1);
        if arrived == state.turn_number {
            words.push(CardWord::Arrived(Arrival::ThisTurn));
        } else if arrived > 0 && arrived >= since {
            words.push(CardWord::Arrived(Arrival::Turn(arrived)));
        }
        if let Some(kind) = intrinsic_entry_counter_kind(&obj.card_data).filter(|kind| entry.counter_count(*kind) == 0) {
            words.push(CardWord::Counter(kind, 0));
        }
        if *damage_marked > 0 {
            words.push(CardWord::Damage(*damage_marked));
        }
        if state.dealt_first_strike_damage.contains(&id) {
            words.push(CardWord::DealtFirstStrikeDamage);
        }
        let mut references = Vec::new();
        if let Some(attack) = attacking {
            match attack.target {
                AttackTarget::Player(p) => words.push(CardWord::Attacking(Attacked::Player(p))),
                AttackTarget::Planeswalker(target) | AttackTarget::Battle(target) => match state.objects.get(&target) {
                    Some(attacked) => references.push(CardWord::Attacking(Attacked::Permanent(self.named_card(attacked)))),
                    // CR 506.4c: it attacks nothing once that has gone.
                    None => self.report(format!("{card} attacking a permanent that no longer exists")),
                },
            }
            if attack.is_blocked && attack.blocked_by.is_empty() {
                words.push(CardWord::Blocked);
            }
            let mut ordered = attack.blocked_by.clone();
            ordered.sort_by_key(|&b| state.object_timestamp(b));
            if ordered != attack.blocked_by {
                self.report(format!("the order {card}'s blockers were declared in, unlike their timestamps"));
            }
        }
        if let Some(block) = blocking {
            if block.blocking.len() > 1 {
                self.report(format!("{card} blocking more than one attacker"));
            }
            match block.blocking.first().map(|attacker| state.objects.get(attacker)) {
                Some(Some(attacker)) => references.push(CardWord::Blocking(self.named_card(attacker))),
                // CR 509.1g: still blocking, though what it blocked has gone.
                Some(None) => self.report(format!("{card} blocking a creature that no longer exists")),
                None => {}
            }
        }
        if let Some(host) = attached_to {
            if state.object_timestamp(*host) > obj.timestamp {
                self.report(format!("{card}, attached to a permanent stamped after it"));
            }
            match state.objects.get(host) {
                Some(host) => references.push(CardWord::AttachedTo(self.named_card(host))),
                None => self.report(format!("{card}, attached to an object that no longer exists")),
            }
        }
        if references.len() > 1 {
            self.report(format!("{card}'s second reference to another card on its line"));
        }
        for (reported, what) in [
            (*flipped || *face_down || *phased_out, "flipped, face down or phased out"),
            (*damaged_by_deathtouch, "dealt deathtouch damage"),
            (*controller_since_turn != *entered_battlefield_turn, "controlled since a turn other than its arrival"),
            (*cost_choices != CostChoices::NONE, "kicked, evoked or otherwise paid for (CR 707.10)"),
            (!entered_as.is_empty(), "entered as a copy, or with its characteristics fixed (CR 614.1c)"),
        ] {
            if reported {
                self.report(format!("{card}, {what}"));
            }
        }
        (CardLine { words, ..line }, references.into_iter().next())
    }

    /// CR 603.2h and 603.7h's counts, a `this turn:` line per source.
    fn write_this_turn_counts(&mut self) {
        let state = self.state;
        let mut facts: Vec<(ObjectId, usize, CardWord)> = Vec::new();
        // A count of an object that has since moved names nothing: the object
        // it was is gone (CR 400.7), and nothing reads the entry again.
        let ability_of = |writer: &mut Self, source: crate::types::ids::ObjectRef, ability: crate::types::ids::AbilityId| {
            let obj = state.find(source)?;
            // AS PRINTED: a count names its ability by its place on the
            // printed card, which is where the loader finds it again.
            let place = crate::objects::card_data::printed_place(&obj.card_data.abilities, ability);
            if place.is_none() || obj.zone != Zone::Battlefield {
                writer.report("a \"this turn\" count of an ability not printed on a permanent on the battlefield");
            }
            place.filter(|_| obj.zone == Zone::Battlefield).map(|i| (source.id, i + 1))
        };
        // The sets are hashed, so each is read in the board's order: a report
        // is a line of the output, and a controller is a layer walk.
        let board_order = |source: crate::types::ids::ObjectRef, ability: crate::types::ids::AbilityId| {
            (state.object_timestamp(source.id), source.zone_change_epoch, format!("{ability:?}"))
        };
        // ORDER-FREE: sorted next.
        let mut triggered: Vec<_> = state.history.triggered_this_turn.iter().collect();
        triggered.sort_by_key(|identity| board_order(identity.source, identity.ability));
        for identity in triggered {
            if let Some((id, n)) = ability_of(self, identity.source, identity.ability) {
                facts.push((id, n, CardWord::Triggered { ability: Some(n) }));
            }
        }
        // ORDER-FREE: sorted next.
        let mut resolved: Vec<_> = state.history.resolutions_this_turn.iter().collect();
        resolved.sort_by_key(|entry| board_order(entry.0.0, entry.0.1));
        for (&(source, ability), &times) in resolved {
            if let Some((id, n)) = ability_of(self, source, ability) {
                facts.push((id, n, CardWord::Resolved { ability: Some(n), times }));
            }
        }
        // ORDER-FREE: sorted next.
        let mut taken: Vec<_> = state.history.action_taken_this_turn.iter().collect();
        taken.sort_by_key(|entry| (board_order(entry.0.source, entry.0.ability), entry.1));
        for (identity, player) in taken {
            if let Some((id, n)) = ability_of(self, identity.source, identity.ability) {
                if crate::oracle::characteristics::get_effective_controller(state, id) != Some(*player) {
                    self.report("a once-each-turn action taken under another controller");
                }
                facts.push((id, n, CardWord::TookOnceEachTurnAction { ability: Some(n) }));
            }
        }
        // By the board, and within a source by its ability's place on the card.
        facts.sort_by_key(|(id, n, word)| (state.object_timestamp(*id), *n, format!("{word:?}")));
        for (id, _, word) in facts {
            let card = self.named_card(&state.objects[&id]);
            match self.scenario.cards.last_mut() {
                Some(line) if line.value.kind == LineKind::ThisTurn && line.value.card == card => line.value.words.push(word),
                _ => self.scenario.cards.push(unnumbered(CardLine { kind: LineKind::ThisTurn, card, copies: 1, words: vec![word] })),
            }
        }
    }
}

/// The `n`th tag, from 0: `a` to `z`, then `aa`, `ab`, … — so a board with
/// more than 26 permanents of one name still tells each apart.
pub fn tag_letters(mut n: usize) -> String {
    let mut letters = Vec::new();
    loop {
        letters.push(b'a' + (n % 26) as u8);
        if n < 26 {
            break;
        }
        n = n / 26 - 1;
    }
    letters.reverse();
    String::from_utf8(letters).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::tag_letters;
    use crate::scenario::Scenario;
    use crate::test_support::{put_on_battlefield, set_attacking, set_blocking, setup_two_player_game, vanilla_creature};

    #[test]
    fn tags_run_past_z() {
        let tags: Vec<String> = [0, 25, 26, 27, 701, 702].into_iter().map(tag_letters).collect();
        assert_eq!(tags, ["a", "z", "aa", "ab", "zz", "aaa"]);
    }

    /// A token's word waits (§5.2), so its line names it as a card, which the
    /// loader refuses; the report says so, and what to do.
    #[test]
    fn a_token_is_reported_with_the_refusal_its_line_meets() {
        let mut game = setup_two_player_game();
        let token = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
        game.objects.get_mut(&token).unwrap().is_token = true;
        let name = game.objects[&token].card_data.name.clone();

        let written = Scenario::write(&game);
        let report = format!("that {name} is a token (§5.2): the loader refuses its line, so remove it and play what makes the token");
        assert_eq!(written.unwritten, [report]);
        let text = written.to_string();
        let refused = Scenario::parse(&text).and_then(|board| board.build(&crate::cards::registry::CardRegistry::default_registry()).map(|_| ()));
        assert!(refused.is_err_and(|refusal| refusal.message.contains(&name)), "{text}");
    }

    /// CR 509.1g: a creature stays blocking until combat ends, after the
    /// attacker it blocked has gone. A token that died ceased to exist (CR
    /// 704.5d), so there is no card to name, and the writer says so. The dev
    /// GUI writes the board at every prompt, and under full control asks at
    /// combat damage's priority points.
    #[test]
    fn a_blocker_whose_attacker_ceased_to_exist_is_reported() {
        let mut game = setup_two_player_game();
        let token = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
        let blocker = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0);
        set_attacking(&mut game, token, 0);
        set_blocking(&mut game, blocker, vec![token]);
        game.battlefield.remove(&token);
        game.remove_object(token);

        let written = Scenario::write(&game);
        assert!(written.unwritten.iter().any(|line| line.contains("blocking a creature that no longer exists")), "{:?}", written.unwritten);
    }
}
