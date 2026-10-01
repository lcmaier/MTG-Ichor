//! A game's state written as a [`Scenario`] (`setup-architecture.md` §4.3).
//!
//! Every field of `GameState`, `PlayerState`, `PermanentState` and
//! `GameObject` is named below with no `..`, so a field the engine adds fails
//! to compile here until it is answered (§5.2's growth contract): written as
//! a word, reported in [`Written::unwritten`], or bound to `_` as the
//! engine's own, with the reason beside it.

use super::board::{Arrival, CardLine, CardRef, Head, HistoryRow, Located, PlayerFact, Scenario, Target, Word};
use super::text::{every_fact, in_combat_from};
use crate::engine::layers::intrinsic::intrinsic_entry_counter;
use crate::engine::layers::types::EffectOrigin;
use crate::objects::object::GameObject;
use crate::state::battlefield::{AttackTarget, CostChoices, PermanentState};
use crate::state::game::starting_player_skips_first_draw;
use crate::state::game_state::{GameState, StepType, TurnPlan};
use crate::state::player::PlayerState;
use crate::types::history::TurnFact;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::zones::Zone;

/// A board written from a game, and what it could not write.
#[derive(Debug, Clone)]
pub struct Written {
    pub scenario: Scenario,
    /// What §2 plays rather than writes, and the fields whose word waits
    /// (§5.2), a sentence each. Empty when the board is exact.
    pub unwritten: Vec<String>,
}

impl std::fmt::Display for Written {
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
    pub fn write(state: &GameState) -> Written {
        let mut writer = Writer { state, scenario: Scenario::default(), unwritten: Vec::new(), tags: Vec::new() };
        writer.tags();
        writer.game();
        writer.players();
        writer.cards();
        Written { scenario: writer.scenario, unwritten: writer.unwritten }
    }
}

struct Writer<'g> {
    state: &'g GameState,
    scenario: Scenario,
    unwritten: Vec<String>,
    /// The objects that need a tag, and theirs.
    tags: Vec<(ObjectId, String)>,
}

fn written<T>(item: T) -> Located<T> {
    Located { line: 0, item }
}

impl<'g> Writer<'g> {
    fn report(&mut self, what: impl Into<String>) {
        self.unwritten.push(what.into());
    }

    fn game(&mut self) {
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
            turn_plan,
            turn_rotation,
            attacks_declared,
            blockers_declared,
            blocker_damage_divisions,
            dealt_first_strike_damage: _, // on each permanent's line
            next_timestamp: _,      // the counters ids and stamps come from
            next_object_id: _,
            player_lost,
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
            entry_selection: _,     // a batch's, empty at any prompt outside one
            prevention_allocations: _,
            nesting: _,
            rider_lineage: _,
            next_zone_change_epoch: _, // CR 704.6d's ticks, settled at rest
            last_sba_check_epoch: _,
            pending_triggers,
            next_trigger_seq: _,
            action_taken_this_turn: _, // `this turn:` lines, in `cards`
            triggered_this_turn: _,
            resolutions_this_turn: _,
            trigger_sources: _,     // the dispatcher's gates
            zone_trigger_sources: _,
            look_back_snapshots: _, // a window's, empty outside one
            departure_frames: _,
            dispatch_audit: _,
            events: _,
            trace: _,
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
        if !turn_queue.is_empty() || *turn_rotation != active {
            self.report("an extra turn (CR 500.7)");
        }
        if turn_plan.phases != TurnPlan::natural().phases {
            self.report("an extra phase (CR 500.8)");
        }
        // The loader begins the natural rotation's turns over the players in
        // the game, ending with the active player's (`build.rs`).
        let in_game: Vec<PlayerId> = (0..players.len()).filter(|&p| !player_lost[p]).collect();
        let mut natural = vec![0; players.len()];
        if let Some(place) = in_game.iter().position(|&p| p == active) {
            for back in 0..(turn as usize).min(in_game.len()) {
                natural[in_game[(place + in_game.len() - back) % in_game.len()]] = turn - back as u32;
            }
        }
        if *last_turn_began != natural {
            self.report("turns that left the natural rotation (an extra or a skipped turn), which CR 302.6 reads");
        }
        let attacked = self.state.battlefield.values().any(|e| e.attacking.is_some())
            || players[active].history.this_turn(turn).count(TurnFact::AttackersDeclared) > 0;
        if *attacks_declared != (attacked && in_combat_from(*phase, StepType::DeclareAttackers))
            || *blockers_declared != (attacked && in_combat_from(*phase, StepType::DeclareBlockers))
        {
            self.report("whether attackers or blockers were declared, which the board does not show (CR 508.8)");
        }
        if !blocker_damage_divisions.is_empty() {
            self.report("a blocker's division of combat damage (CR 510.1d)");
        }
        let first_draw = turn == 1 && phase.step == Some(StepType::Upkeep) && starting_player_skips_first_draw(self.state);
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

    fn players(&mut self) {
        let state = self.state;
        let turn = state.turn_number;
        for (p, player) in state.players.iter().enumerate() {
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
                history,
            } = player;
            let mut facts = Vec::new();
            if *life_total != state.starting_life {
                facts.push(PlayerFact::Life { player: p, life: *life_total });
            }
            facts.extend(counters.iter().map(|(kind, count)| PlayerFact::Counter { player: p, kind: *kind, count: *count }));
            if *lands_played_this_turn > 0 {
                facts.push(PlayerFact::LandsPlayed { player: p, count: *lands_played_this_turn });
            }
            if state.player_lost[p] {
                facts.push(PlayerFact::LeftTheGame { player: p });
            }
            let mut damage: Vec<(&ObjectId, &u32)> = commander_damage_taken.iter().collect();
            damage.sort_by_key(|(id, _)| state.object_timestamp(**id));
            for (commander, damage) in damage {
                match state.objects.get(commander) {
                    Some(obj) => facts.push(PlayerFact::CommanderDamage { player: p, damage: *damage, from: self.reference(obj) }),
                    None => self.report(format!("player {p}'s commander damage from a commander no longer in the game")),
                }
            }
            for fact in every_fact() {
                let (this, last, game) =
                    (history.this_turn(turn).count(fact), history.last_turn(turn).count(fact), history.this_game().count(fact));
                for (row, count) in [(HistoryRow::ThisTurn, this), (HistoryRow::LastTurn, last)] {
                    if count > 0 {
                        facts.push(PlayerFact::History { player: p, row, fact, count });
                    }
                }
                if game != this + last {
                    facts.push(PlayerFact::History { player: p, row: HistoryRow::ThisGame, fact, count: game });
                }
            }
            self.scenario.player_facts.extend(facts.into_iter().map(written));
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

    /// Does `player`'s "since your last turn" differ from what the loader
    /// derives? It lands the rows on turns T and T-1 and the rest of "this
    /// game" on T-2, so a player whose last turn ended at T-1 or T-2 reads
    /// exactly, and one whose ended earlier, or who has had none, reads it all.
    fn since_your_last_turn_differs(&self, player: PlayerId) -> bool {
        let state = self.state;
        let turn = state.turn_number;
        let in_game = state.players.len() - state.player_lost.iter().filter(|&&l| l).count();
        let ended = match state.last_turn_began[player] {
            began if player == state.active_player => (began as usize).saturating_sub(in_game) as u32,
            began => began,
        };
        let mine = &state.players[player].history;
        state.players.iter().enumerate().any(|(q, theirs)| {
            every_fact().any(|fact| {
                let (this, last) = (theirs.history.this_turn(turn).count(fact), theirs.history.last_turn(turn).count(fact));
                let derived = match turn.checked_sub(ended) {
                    Some(1) if ended > 0 => this,
                    Some(2) if ended > 0 => this + last,
                    _ => theirs.history.this_game().count(fact),
                };
                mine.since_your_last_turn(q, &theirs.history).count(fact) != derived
            })
        })
    }

    /// `obj`'s name, with its tag if two objects it could be confused with
    /// share it.
    fn reference(&self, obj: &GameObject) -> CardRef {
        let tag = self.tags.iter().find(|(id, _)| *id == obj.id).map(|(_, tag)| tag.clone());
        CardRef { name: obj.card_data.name.clone(), tag }
    }

    /// A tag for each name two permanents share, or two commanders, as the
    /// loader resolves a reference among the one or the other.
    fn tags(&mut self) {
        let state = self.state;
        let permanents = state.battlefield_ids_ordered();
        let mut commanders: Vec<ObjectId> = state.objects.values().filter(|o| o.is_commander).map(|o| o.id).collect();
        commanders.sort_by_key(|&id| state.object_timestamp(id));
        for group in [&permanents, &commanders] {
            for &id in group.iter() {
                let name = &state.objects[&id].card_data.name;
                let shared = group.iter().filter(|other| state.objects[other].card_data.name == *name).count() > 1;
                if shared && !self.tags.iter().any(|(tagged, _)| *tagged == id) {
                    let used = self.tags.iter().filter(|(other, _)| state.objects[other].card_data.name == *name).count();
                    self.tags.push((id, ((b'a' + (used % 26) as u8) as char).to_string()));
                }
            }
        }
    }

    fn cards(&mut self) {
        let state = self.state;
        let permanents = state.battlefield_ids_ordered();
        // Each library top first, then everything else in CR 613.7d's order,
        // a permanent's counters folded into its line unless something was
        // stamped between them (CR 613.7c).
        for (p, player) in state.players.iter().enumerate() {
            for id in player.library.iter().rev() {
                let line = CardLine { head: Head::Library { player: p, shuffled: false }, ..self.card_line(*id) };
                self.push(line);
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
        let mut references: Vec<(usize, Word)> = Vec::new();
        let mut last: Option<ObjectId> = None;
        for (_, id, counter) in stamps {
            let Some(kind) = counter else {
                let (line, reference) = self.permanent_line(id);
                if let Some(word) = reference {
                    references.push((self.scenario.cards.len(), word));
                }
                self.push(line);
                last = Some(id);
                continue;
            };
            let count = Word::Counter(kind, state.battlefield[&id].counter_count(kind));
            if last == Some(id) {
                if let Some(line) = self.scenario.cards.last_mut() {
                    line.item.words.push(count);
                }
            } else {
                let card = self.reference(&state.objects[&id]);
                if !self.scenario.cards.iter().any(|line| line.item.card == card) {
                    self.report(format!("{card}'s {} counters, older than its timestamp", kind.name()));
                }
                self.push(CardLine { head: Head::Counters, card, copies: 1, words: vec![count] });
                last = Some(id);
            }
        }
        for (index, word) in references {
            self.scenario.cards[index].item.words.push(word);
        }
        self.this_turn();
    }

    /// `line`, folded into the line before it when the two say the same of
    /// two cards in one zone.
    fn push(&mut self, line: CardLine) {
        if let Some(previous) = self.scenario.cards.last_mut()
            && line.head != Head::Battlefield
            && line.head != Head::Counters
            && previous.item.head == line.head
            && previous.item.card == line.card
            && previous.item.words == line.words
        {
            previous.item.copies += 1;
            return;
        }
        self.scenario.cards.push(written(line));
    }

    /// The object's line in its zone, with the `GameObject` words.
    fn card_line(&mut self, id: ObjectId) -> CardLine {
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
        let card = self.reference(obj);
        if *is_token || *is_copy {
            self.report(format!("{card}, a token or a copy, whose word waits (§5.2)"));
        }
        let mut words = Vec::new();
        let head = match zone {
            Zone::Hand => Head::Hand(*owner),
            Zone::Library => Head::Library { player: *owner, shuffled: false },
            Zone::Graveyard => Head::Graveyard(*owner),
            Zone::Exile | Zone::Command => {
                words.push(Word::Owner(*owner));
                if *zone == Zone::Exile { Head::Exile } else { Head::Command }
            }
            Zone::Battlefield | Zone::Stack => Head::Battlefield,
        };
        if *is_commander {
            words.push(Word::Commander);
        }
        CardLine { head, card, copies: 1, words }
    }

    /// A permanent's line, or any other object's, and the one word naming
    /// another card that the line ends with, added once its counters are.
    fn permanent_line(&mut self, id: ObjectId) -> (CardLine, Option<Word>) {
        let mut line = self.card_line(id);
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
        } = entry;
        let obj = &state.objects[&id];
        let card = line.card.to_string();
        let mut words = vec![Word::Controller(*controller)];
        if obj.owner != *controller {
            words.push(Word::Owner(obj.owner));
        }
        words.append(&mut line.words);
        if *tapped {
            words.push(Word::Tapped);
        }
        // CR 302.6 reads the arrival only against the controller's most
        // recent turn; an arrival before it is the default's answer.
        let arrived = *entered_battlefield_turn;
        let since = state.most_recent_turn_began(*controller).unwrap_or(1);
        if arrived == state.turn_number {
            words.push(Word::Arrived(Arrival::ThisTurn));
        } else if arrived > 0 && arrived >= since {
            words.push(Word::Arrived(Arrival::Turn(arrived)));
        }
        if let Some(kind) = intrinsic_entry_counter(&obj.card_data).filter(|kind| entry.counter_count(*kind) == 0) {
            words.push(Word::Counter(kind, 0));
        }
        if *damage_marked > 0 {
            words.push(Word::Damage(*damage_marked));
        }
        if state.dealt_first_strike_damage.contains(&id) {
            words.push(Word::DealtFirstStrikeDamage);
        }
        let mut references = Vec::new();
        if let Some(attack) = attacking {
            match attack.target {
                AttackTarget::Player(p) => words.push(Word::Attacking(Target::Player(p))),
                AttackTarget::Planeswalker(target) | AttackTarget::Battle(target) => {
                    references.push(Word::Attacking(Target::Permanent(self.reference(&state.objects[&target]))));
                }
            }
            if attack.is_blocked && attack.blocked_by.is_empty() {
                words.push(Word::Blocked);
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
            references.extend(block.blocking.first().map(|a| Word::Blocking(self.reference(&state.objects[a]))));
        }
        if let Some(host) = attached_to {
            if state.object_timestamp(*host) > obj.timestamp {
                self.report(format!("{card}, attached to a permanent stamped after it"));
            }
            references.push(Word::AttachedTo(self.reference(&state.objects[host])));
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
    fn this_turn(&mut self) {
        let state = self.state;
        let mut facts: Vec<(ObjectId, usize, Word)> = Vec::new();
        // A count of an object that has since moved names nothing: the object
        // it was is gone (CR 400.7), and nothing reads the entry again.
        let ability_of = |writer: &mut Self, source: crate::types::ids::ObjectRef, ability: crate::types::ids::AbilityId| {
            let obj = state.objects.get(&source.id).filter(|o| o.zone_change_epoch == source.zone_change_epoch)?;
            let place = obj.card_data.abilities.iter().position(|a| a.id.definition() == ability.definition());
            if place.is_none() || obj.zone != Zone::Battlefield {
                writer.report("a \"this turn\" count of an ability not printed on a permanent on the battlefield");
            }
            place.filter(|_| obj.zone == Zone::Battlefield).map(|i| (source.id, i + 1))
        };
        for identity in &state.triggered_this_turn {
            if let Some((id, n)) = ability_of(self, identity.source, identity.ability) {
                facts.push((id, n, Word::Triggered { ability: Some(n) }));
            }
        }
        for (&(source, ability), &times) in &state.resolutions_this_turn {
            if let Some((id, n)) = ability_of(self, source, ability) {
                facts.push((id, n, Word::Resolved { ability: Some(n), times }));
            }
        }
        for (identity, player) in &state.action_taken_this_turn {
            if let Some((id, n)) = ability_of(self, identity.source, identity.ability) {
                if crate::oracle::characteristics::get_effective_controller(state, id) != Some(*player) {
                    self.report("a once-each-turn action taken under another controller");
                }
                facts.push((id, n, Word::TookOnceEachTurnAction { ability: Some(n) }));
            }
        }
        // The sets are hashed, so the order comes from the board.
        facts.sort_by_key(|(id, n, word)| (state.object_timestamp(*id), *n, format!("{word:?}")));
        for (id, _, word) in facts {
            let card = self.reference(&state.objects[&id]);
            match self.scenario.cards.last_mut() {
                Some(line) if line.item.head == Head::ThisTurn && line.item.card == card => line.item.words.push(word),
                _ => self.scenario.cards.push(written(CardLine { head: Head::ThisTurn, card, copies: 1, words: vec![word] })),
            }
        }
    }
}
