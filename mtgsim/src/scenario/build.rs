//! A [`Scenario`] built into a `Game` (`setup-architecture.md` §3): every
//! write through the engine's own doors, in file order, before the first
//! turn, and nothing emitted. §4.1's refusals are here.

use std::sync::Arc;

use super::board::{Arrival, Attacked, CardLine, CardWord, LineKind, LineNumbered, NamedCard, PlayerWord, Scenario};
use super::error::{ScenarioError, ScenarioErrorKind};
use super::text::{in_combat_from, position_word};
use crate::cards::registry::CardRegistry;
use crate::objects::card_data::{AbilityType, CardData};
use crate::objects::object::GameObject;
use crate::oracle::characteristics::{get_effective_controller, get_effective_types};
use crate::state::battlefield::{AttackTarget, AttackingInfo, BlockingInfo, PermanentState};
use crate::state::game::{starting_player_skips_first_draw, Game, RandomStreams};
use crate::state::game_config::GameConfig;
use crate::state::game_state::{AbilityIdentity, GameState, StepType};
use crate::state::history::PlayerHistory;
use crate::types::card_types::CardType;
use crate::types::history::{HistorySpan, TurnFact};
use crate::types::ids::{ObjectId, ObjectRef, PlayerId};
use crate::types::zones::Zone;

impl Scenario {
    /// Build the board this describes, at the start of `step`'s priority
    /// round with the active player to act; `Game::resume` plays it. Names
    /// are looked up in `registry`, then among its cards in development.
    pub fn build(&self, registry: &CardRegistry) -> Result<Game, ScenarioError> {
        let mut loader = Loader::new(self)?;
        for line in &self.cards {
            loader.load_card_line(line, registry)?;
        }
        loader.load_player_words()?;
        loader.load_combat()?;
        loader.begin_turns()?;
        loader.load_this_turn_counts()?;
        let state = loader.state;
        Ok(Game { state, config: GameConfig { starting_life: self.starting_life, ..GameConfig::unrestricted() } })
    }
}

/// An object the file created, and what a reference to it can match.
struct CreatedObject {
    line: usize,
    id: ObjectId,
    card: NamedCard,
    zone: Zone,
    commander: bool,
}

struct Loader<'s> {
    scenario: &'s Scenario,
    state: GameState,
    created: Vec<CreatedObject>,
    /// Combat words, applied once every permanent exists.
    attackers: Vec<(usize, ObjectId, &'s Attacked)>,
    blocked: Vec<(usize, ObjectId)>,
    blockers: Vec<(usize, ObjectId, &'s NamedCard)>,
    /// Whether each player's library lines are `shuffled`, once one is seen.
    shuffled: Vec<Option<bool>>,
}

impl<'s> Loader<'s> {
    fn new(scenario: &'s Scenario) -> Result<Loader<'s>, ScenarioError> {
        let whole = |message: String| ScenarioError { kind: ScenarioErrorKind::Unreachable, line: None, message };
        if scenario.players < 2 {
            return Err(whole("a game has two players or more (CR 102.1)".to_string()));
        }
        if scenario.active >= scenario.players || scenario.turn == 0 {
            return Err(whole(format!("`turn {}, active {}` names no turn of this game", scenario.turn, scenario.active)));
        }
        if matches!(scenario.step.step, Some(StepType::Untap | StepType::Cleanup)) {
            return Err(whole(format!(
                "no player receives priority in the {} step (CR 502.4, 514.3); a scenario starts at a priority round",
                position_word(scenario.step)
            )));
        }
        let mut state = GameState::new(scenario.players, scenario.starting_life);
        if scenario.turn == 1 && scenario.step.step == Some(StepType::Draw) && starting_player_skips_first_draw(&state) {
            return Err(whole(
                "the starting player of a two-player game skips their first draw step (CR 103.8a), so turn 1 has none \
                 (CR 500.11); set `step upkeep` or a later step"
                    .to_string(),
            ));
        }
        // Built in turn 0, as `Game::setup` deals the opening hands; `turn`
        // begins the rotation's turns last. `GameState::new` began turn 1
        // for player 0, which this game's rotation may not.
        state.turn_number = 0;
        state.last_turn_began = vec![0; scenario.players];
        for player in &mut state.players {
            player.history = PlayerHistory::before_any_turn();
        }
        state.reseed(RandomStreams::from_seed(scenario.seed).game);
        Ok(Loader {
            scenario,
            state,
            created: Vec::new(),
            attackers: Vec::new(),
            blocked: Vec::new(),
            blockers: Vec::new(),
            shuffled: vec![None; scenario.players],
        })
    }

    fn checked_player(&self, player: PlayerId, line: usize) -> Result<PlayerId, ScenarioError> {
        if player < self.scenario.players {
            Ok(player)
        } else {
            Err(ScenarioError::at(ScenarioErrorKind::Reference, line, format!("player {player} is not in a {}-player game", self.scenario.players)))
        }
    }

    /// The object `card` names among those created so far that `among`
    /// keeps, refusing a name that matches none, or two.
    fn object_named(&self, card: &NamedCard, line: usize, what: &str, among: impl Fn(&CreatedObject) -> bool) -> Result<ObjectId, ScenarioError> {
        let matches: Vec<&CreatedObject> =
            self.created.iter().filter(|c| among(c) && c.card.name == card.name && (card.tag.is_none() || c.card.tag == card.tag)).collect();
        match matches.as_slice() {
            [one] => Ok(one.id),
            [] => Err(ScenarioError::at(
                ScenarioErrorKind::Reference,
                line,
                format!("{card} names no {what} listed above this line (a word that names a card takes the rest of its line, so it comes last)"),
            )),
            [first, second, ..] => Err(ScenarioError::at(
                ScenarioErrorKind::Reference,
                line,
                format!("{} names two {what}s, lines {} and {}; give each a tag, as `{} [a]`", card, first.line, second.line, card.name),
            )),
        }
    }

    fn permanent_named(&self, card: &NamedCard, line: usize) -> Result<ObjectId, ScenarioError> {
        self.object_named(card, line, "permanent", |c| c.zone == Zone::Battlefield)
    }

    fn load_card_line(&mut self, located: &'s LineNumbered<CardLine>, registry: &CardRegistry) -> Result<(), ScenarioError> {
        let (line, card_line) = (located.line, &located.value);
        match card_line.kind {
            LineKind::Counters => {
                let id = self.permanent_named(&card_line.card, line)?;
                for word in &card_line.words {
                    if let CardWord::Counter(kind, count) = word {
                        self.set_counters(id, *kind, *count);
                    }
                }
                return Ok(());
            }
            // Read once the turn has begun, which clears this turn's sets.
            LineKind::ThisTurn => return Ok(()),
            _ => {}
        }
        let name = &card_line.card.name;
        let card = registry.create(name).ok().or_else(|| registry.create_in_development(name)).ok_or_else(|| {
            ScenarioError::at(
                ScenarioErrorKind::NotACard,
                line,
                format!("{name} is not registered: names are exact, and a card the engine does not play yet goes in the list of cards in development"),
            )
        })?;
        let owner_word = card_line.words.iter().find_map(|w| if let CardWord::Owner(p) = w { Some(*p) } else { None });
        let controller_word = card_line.words.iter().find_map(|w| if let CardWord::Controller(p) = w { Some(*p) } else { None });
        let (zone, owner) = match card_line.kind {
            LineKind::Hand(p) => (Zone::Hand, p),
            LineKind::Library { player, shuffled } => {
                let seat = self.checked_player(player, line)?;
                if *self.shuffled[seat].get_or_insert(shuffled) != shuffled {
                    let message = format!("library {player} has both shuffled and listed lines; a shuffle orders the whole library");
                    return Err(ScenarioError::at(ScenarioErrorKind::Syntax, line, message));
                }
                (Zone::Library, player)
            }
            LineKind::Graveyard(p) => (Zone::Graveyard, p),
            LineKind::Exile | LineKind::Command => {
                let owner = owner_word.ok_or_else(|| ScenarioError::at(ScenarioErrorKind::Syntax, line, format!("{name} needs `owner p`: whose card is it?")))?;
                (if card_line.kind == LineKind::Exile { Zone::Exile } else { Zone::Command }, owner)
            }
            _ => {
                let owner = owner_word.or(controller_word).ok_or_else(|| {
                    ScenarioError::at(ScenarioErrorKind::Syntax, line, format!("{name} needs `controller p` or `owner p`; each defaults to the other"))
                })?;
                (Zone::Battlefield, owner)
            }
        };
        self.checked_player(owner, line)?;
        for _ in 0..card_line.copies {
            let id = self.create_object(&card, zone, owner, controller_word, card_line, line)?;
            let commander = card_line.words.contains(&CardWord::Commander);
            self.created.push(CreatedObject { line, id, card: card_line.card.clone(), zone, commander });
        }
        Ok(())
    }

    fn create_object(
        &mut self,
        card: &Arc<CardData>,
        zone: Zone,
        owner: PlayerId,
        controller: Option<PlayerId>,
        card_line: &'s CardLine,
        line: usize,
    ) -> Result<ObjectId, ScenarioError> {
        let mut obj = GameObject::new(Arc::clone(card), owner, zone);
        obj.is_commander = card_line.words.contains(&CardWord::Commander);
        let unreachable = |message: String| ScenarioError::at(ScenarioErrorKind::Unreachable, line, message);
        if zone != Zone::Battlefield {
            let id = self.state.create_in_zone(obj).map_err(unreachable)?;
            if let LineKind::Library { player, .. } = card_line.kind {
                // Top first: each card goes under the ones above it.
                let library = &mut self.state.players[player].library;
                library.pop();
                library.insert(0, id);
            }
            return Ok(id);
        }
        // PRE-LAYER ZONE: CR 304.4 and 307.4 are asked of a card before it is
        // a permanent, and no effect can make a permanent one (CR 205.1b).
        if card.types.contains(&CardType::Instant) || card.types.contains(&CardType::Sorcery) {
            return Err(unreachable(format!("{} can't be on the battlefield: instants and sorceries can't enter it (CR 304.4, 307.4)", card.name)));
        }
        let turn = self.scenario.turn;
        let arrived = match card_line.words.iter().find_map(|w| if let CardWord::Arrived(a) = w { Some(*a) } else { None }) {
            None => 0,
            Some(Arrival::ThisTurn) => turn,
            Some(Arrival::Turn(n)) if (1..=turn).contains(&n) => n,
            Some(Arrival::Turn(n)) => return Err(unreachable(format!("`arrived turn {n}` is not a turn before turn {turn}"))),
        };
        let controller = self.checked_player(controller.unwrap_or(owner), line)?;
        let id = self.state.create_on_battlefield(obj, controller, arrived).map_err(unreachable)?;
        for word in &card_line.words {
            match word {
                CardWord::Tapped => self.update_permanent(id, |entry| entry.tapped = true),
                CardWord::Counter(kind, count) => self.set_counters(id, *kind, *count),
                CardWord::Damage(damage) => self.update_permanent(id, |entry| entry.damage_marked = *damage),
                CardWord::DealtFirstStrikeDamage => {
                    self.state.dealt_first_strike_damage.insert(id);
                }
                CardWord::AttachedTo(host) => {
                    let host = self.permanent_named(host, line)?;
                    self.state.attach(id, host);
                }
                CardWord::Attacking(target) => self.attackers.push((line, id, target)),
                CardWord::Blocked => self.blocked.push((line, id)),
                CardWord::Blocking(attacker) => self.blockers.push((line, id, attacker)),
                // Read as the object was created.
                CardWord::Owner(_) | CardWord::Controller(_) | CardWord::Commander | CardWord::Arrived(_) => {}
                CardWord::Triggered { .. } | CardWord::Resolved { .. } | CardWord::TookOnceEachTurnAction { .. } => {}
            }
        }
        Ok(id)
    }

    /// Write to `id`'s battlefield entry, one the loader created.
    fn update_permanent(&mut self, id: ObjectId, change: impl FnOnce(&mut PermanentState)) {
        if let Some(entry) = self.state.battlefield.get_mut(&id) {
            change(entry);
        }
    }

    /// A stated kind replaces the count the permanent has, stamped at this
    /// line (CR 613.7c); 0 leaves none, which is how a stated kind replaces
    /// CR 306.5b's intrinsic loyalty.
    fn set_counters(&mut self, id: ObjectId, kind: crate::types::effects::CounterType, count: u32) {
        let held = self.state.battlefield[&id].counter_count(kind);
        self.state.remove_counters(id, kind, held);
        if count > 0 {
            self.state.add_counters(id, kind, count);
        }
    }

    /// Is the board at `from` or later in this turn's combat?
    fn in_combat_from(&self, from: StepType) -> bool {
        in_combat_from(self.scenario.step, from)
    }

    fn load_combat(&mut self) -> Result<(), ScenarioError> {
        let active = self.scenario.active;
        // "the precombat main phase", "the declare blockers step".
        let step = format!("{} {}", position_word(self.scenario.step), if self.scenario.step.step.is_some() { "step" } else { "phase" });
        let unreachable = |line: usize, message: String| ScenarioError::at(ScenarioErrorKind::Unreachable, line, message);
        for (line, id, target) in std::mem::take(&mut self.attackers) {
            let name = self.state.objects[&id].card_data.name.clone();
            if !self.in_combat_from(StepType::DeclareAttackers) {
                return Err(unreachable(line, format!(
                    "{name} is attacking in the {step}; attackers exist from the declare attackers step to the end of combat \
                     (CR 506.4, 511.3); set `step declare attackers` or later"
                )));
            }
            if get_effective_controller(&self.state, id) != Some(active) {
                return Err(unreachable(line, format!("{name} is attacking, and only the active player's creatures attack (CR 508.1a)")));
            }
            let target = match target {
                Attacked::Player(player) if !self.state.in_game(self.checked_player(*player, line)?) => {
                    return Err(unreachable(line, format!("{name} attacks player {player}, who has left the game (CR 506.2, 800.4a)")));
                }
                Attacked::Player(player) => AttackTarget::Player(*player),
                Attacked::Permanent(card) => {
                    let attacked = self.permanent_named(card, line)?;
                    let types = get_effective_types(&self.state, attacked);
                    if types.contains(&CardType::Planeswalker) {
                        AttackTarget::Planeswalker(attacked)
                    } else if types.contains(&CardType::Battle) {
                        AttackTarget::Battle(attacked)
                    } else {
                        return Err(unreachable(line, format!("{name} attacks {}, which is neither a planeswalker nor a battle (CR 508.1b)", card)));
                    }
                }
            };
            if attacked_player(&self.state, &target) == Some(active) {
                return Err(unreachable(line, format!("{name} attacks its own controller's side; an attacker attacks an opponent (CR 508.1b)")));
            }
            let is_blocked = self.blocked.iter().any(|&(_, blocked)| blocked == id);
            self.update_permanent(id, |entry| entry.attacking = Some(AttackingInfo { target, is_blocked, blocked_by: Vec::new() }));
        }
        if let Some(&(line, id)) = self.blocked.iter().find(|&&(_, id)| self.state.battlefield[&id].attacking.is_none()) {
            return Err(unreachable(line, format!("{} is blocked, and only an attacker is (CR 509.1h)", self.state.objects[&id].card_data.name)));
        }
        for (line, blocker, attacker) in std::mem::take(&mut self.blockers) {
            let name = self.state.objects[&blocker].card_data.name.clone();
            if !self.in_combat_from(StepType::DeclareBlockers) {
                return Err(unreachable(line, format!(
                    "{name} is blocking in the {step}; blockers exist from the declare blockers step to the end of combat \
                     (CR 509.1, 511.3); set `step declare blockers` or later"
                )));
            }
            let attacker_id = self.permanent_named(attacker, line)?;
            let Some(attacked) = self.state.battlefield[&attacker_id].attacking.as_ref().map(|a| attacked_player(&self.state, &a.target)) else {
                return Err(unreachable(line, format!("{name} blocks {}, which is not attacking (CR 509.1a)", attacker)));
            };
            if get_effective_controller(&self.state, blocker) != attacked {
                return Err(unreachable(line, format!("{name} blocks {}, and only the player it attacks blocks it (CR 509.1a)", attacker)));
            }
            self.update_permanent(blocker, |entry| entry.blocking = Some(BlockingInfo { blocking: vec![attacker_id] }));
            self.update_permanent(attacker_id, |entry| {
                if let Some(attacking) = entry.attacking.as_mut() {
                    attacking.is_blocked = true;
                    attacking.blocked_by.push(blocker);
                }
            });
        }
        if !self.state.dealt_first_strike_damage.is_empty() && !self.in_combat_from(StepType::FirstStrikeDamage) {
            let message = format!("first-strike damage is dealt in the first-strike damage step (CR 510.4), not before it, in the {step}");
            return Err(ScenarioError { kind: ScenarioErrorKind::Unreachable, line: None, message });
        }
        Ok(())
    }

    fn load_player_words(&mut self) -> Result<(), ScenarioError> {
        for located in &self.scenario.player_words {
            let line = located.line;
            let (PlayerWord::Life { player, .. }
            | PlayerWord::Counter { player, .. }
            | PlayerWord::LandsPlayed { player, .. }
            | PlayerWord::LeftTheGame { player }
            | PlayerWord::CommanderDamage { player, .. }
            | PlayerWord::History { player, .. }) = located.value;
            let player = self.checked_player(player, line)?;
            match &located.value {
                PlayerWord::Life { life, .. } => self.state.players[player].life_total = *life,
                PlayerWord::Counter { kind, count, .. } => self.state.players[player].add_counters(*kind, *count),
                PlayerWord::LandsPlayed { count, .. } => self.state.players[player].lands_played_this_turn = *count,
                PlayerWord::LeftTheGame { .. } => self.player_leaves(player, line)?,
                PlayerWord::CommanderDamage { damage, from, .. } => {
                    let commander = self.object_named(from, line, "commander", |c| c.commander)?;
                    self.state.players[player].commander_damage_taken.insert(commander, *damage);
                }
                PlayerWord::History { .. } => {}
            }
        }
        Ok(())
    }

    /// CR 800.4a: a player who has left owns and controls nothing, and the
    /// game has two players left (CR 104.2a) and an active one.
    fn player_leaves(&mut self, player: PlayerId, line: usize) -> Result<(), ScenarioError> {
        let theirs = self.created.iter().find(|c| {
            self.state.objects[&c.id].owner == player || get_effective_controller(&self.state, c.id) == Some(player)
        });
        let message = if let Some(card) = theirs {
            format!("player {player} has left, and owns or controls {} on line {} (CR 800.4a)", card.card, card.line)
        } else if player == self.scenario.active {
            format!("player {player} has left, and is the active player: a turn begins only for a player in the game (CR 800.4k)")
        } else if self.state.player_lost.iter().filter(|&&lost| !lost).count() <= 2 {
            "a game with fewer than two players left in it is over (CR 104.2a)".to_string()
        } else {
            self.state.player_lost[player] = true;
            return Ok(());
        };
        Err(ScenarioError::at(ScenarioErrorKind::Unreachable, line, message))
    }

    /// The turns the natural rotation over the players in the game gives,
    /// ending with the active player's turn T, each begun through the turn's
    /// writers; then the position, and each history row on its turn.
    fn begin_turns(&mut self) -> Result<(), ScenarioError> {
        let (turn, active) = (self.scenario.turn, self.scenario.active);
        let in_game: Vec<PlayerId> = (0..self.scenario.players).filter(|&p| self.state.in_game(p)).collect();
        let place = in_game.iter().position(|&p| p == active).unwrap_or_default();
        let rows = self.history_counts_by_turn()?;
        for t in 1..=turn {
            let back = (turn - t) as usize % in_game.len();
            let player = in_game[(place + in_game.len() - back) % in_game.len()];
            self.state.begin_turn(t, player);
            self.state.begin_turn_history(t, player);
            for &(player, on_turn, fact, count) in &rows {
                if on_turn == t && count > 0 {
                    self.state.players[player].history.add(t, fact, count);
                }
            }
        }
        self.state.turn_rotation = active;
        self.state.priority_player = active;
        self.state.set_turn_position(self.scenario.step);
        // CR 103.8: only a turn-1 board before its draw step has the skip to
        // come, and `begin_step` refuses every draw step while it is set.
        self.state.skip_first_draw = turn == 1 && self.scenario.step.step == Some(StepType::Upkeep) && starting_player_skips_first_draw(&self.state);
        let attacked = self.state.battlefield.values().any(|e| e.attacking.is_some())
            || self.state.players[active].history.this_turn(turn).count(TurnFact::AttackersDeclared) > 0;
        self.state.attacks_declared = self.in_combat_from(StepType::DeclareAttackers) && attacked;
        self.state.blockers_declared = self.in_combat_from(StepType::DeclareBlockers) && attacked;
        let skipped_without_attackers = matches!(
            self.scenario.step.step,
            Some(StepType::DeclareBlockers | StepType::FirstStrikeDamage | StepType::CombatDamage)
        );
        if skipped_without_attackers && !attacked {
            return Err(ScenarioError {
                kind: ScenarioErrorKind::Unreachable,
                line: None,
                message: format!(
                    "the {} step happens only after attackers were declared (CR 508.8); add an attacker, or \
                     `player {active} this turn: attackers declared N`",
                    position_word(self.scenario.step)
                ),
            });
        }
        for (player, shuffled) in self.shuffled.clone().into_iter().enumerate() {
            if shuffled == Some(true) {
                self.state.shuffle_library(player);
            }
        }
        Ok(())
    }

    /// Each history count as (player, turn, fact, count): this turn's on T,
    /// last turn's on T-1, and what "this game" counts beyond them on T-2,
    /// the turn "since your last turn" reads back to for the player before.
    fn history_counts_by_turn(&self) -> Result<Vec<(PlayerId, u32, TurnFact, u64)>, ScenarioError> {
        let turn = self.scenario.turn;
        let mut totals: Vec<(PlayerId, TurnFact, [u64; 2], Option<(usize, u64)>)> = Vec::new();
        for located in &self.scenario.player_words {
            let PlayerWord::History { player, span, fact, count } = located.value else { continue };
            let index = match totals.iter().position(|t| t.0 == player && t.1 == fact) {
                Some(index) => index,
                None => {
                    totals.push((player, fact, [0, 0], None));
                    totals.len() - 1
                }
            };
            match span {
                HistorySpan::ThisTurn => totals[index].2[0] += count,
                HistorySpan::LastTurn if turn > 1 => totals[index].2[1] += count,
                HistorySpan::LastTurn => {
                    return Err(ScenarioError::at(ScenarioErrorKind::Unreachable, located.line, "turn 1 has no last turn: the game began with it"));
                }
                HistorySpan::ThisGame => totals[index].3 = Some((located.line, count)),
                HistorySpan::SinceYourLastTurn => {
                    let message = "\"since your last turn\" is derived from this turn, last turn and this game; state those";
                    return Err(ScenarioError::at(ScenarioErrorKind::Unreachable, located.line, message));
                }
            }
        }
        let mut rows = Vec::new();
        for (player, fact, [this, last], game) in totals {
            rows.push((player, turn, fact, this));
            rows.push((player, turn.saturating_sub(1), fact, last));
            let Some((line, game)) = game else { continue };
            let earlier = game.checked_sub(this + last).ok_or_else(|| {
                ScenarioError::at(ScenarioErrorKind::Unreachable, line, "this game counts fewer than this turn and last turn together")
            })?;
            if earlier > 0 && turn < 3 {
                return Err(ScenarioError::at(ScenarioErrorKind::Unreachable, line, format!("turn {turn} has no turn before last to count the rest on")));
            }
            rows.push((player, turn.saturating_sub(2), fact, earlier));
        }
        Ok(rows)
    }

    /// CR 603.2h and 603.7h's counts, once turn T has begun and cleared them.
    fn load_this_turn_counts(&mut self) -> Result<(), ScenarioError> {
        for located in &self.scenario.cards {
            let (line, card_line) = (located.line, &located.value);
            if card_line.kind != LineKind::ThisTurn {
                continue;
            }
            let source = self.permanent_named(&card_line.card, line)?;
            let card = Arc::clone(&self.state.objects[&source].card_data);
            let reference = ObjectRef { id: source, zone_change_epoch: self.state.objects[&source].zone_change_epoch };
            for word in &card_line.words {
                // CR 603.2h's limit is a triggered ability's; CR 603.7h counts
                // either kind resolving.
                let (ability, can) = match word {
                    CardWord::Triggered { ability } | CardWord::TookOnceEachTurnAction { ability } => (ability, AbilityType::Triggered),
                    CardWord::Resolved { ability, .. } => (ability, AbilityType::Activated),
                    _ => continue,
                };
                let found = match ability {
                    Some(n) => n.checked_sub(1).and_then(|i| card.abilities.get(i)),
                    None => {
                        let mut able = card.abilities.iter().filter(|a| a.ability_type == AbilityType::Triggered || a.ability_type == can);
                        let one = able.next();
                        one.filter(|_| able.next().is_none())
                    }
                };
                let ability = found.ok_or_else(|| {
                    ScenarioError::at(ScenarioErrorKind::Reference, line, format!("name the ability: `ability N`, N its place among {}'s printed abilities", card.name))
                })?;
                let identity = AbilityIdentity { source: reference, ability: ability.id };
                match word {
                    CardWord::Triggered { .. } => {
                        self.state.triggered_this_turn.insert(identity);
                    }
                    CardWord::Resolved { times, .. } => {
                        self.state.resolutions_this_turn.insert((reference, ability.id.definition()), *times);
                    }
                    _ => {
                        let controller = get_effective_controller(&self.state, source).unwrap_or(self.state.objects[&source].owner);
                        self.state.action_taken_this_turn.insert((identity, controller));
                    }
                }
            }
        }
        Ok(())
    }
}

/// The player an attack is against: the player, or the attacked permanent's
/// controller.
fn attacked_player(state: &GameState, target: &AttackTarget) -> Option<PlayerId> {
    match target {
        AttackTarget::Player(player) => Some(*player),
        AttackTarget::Planeswalker(id) | AttackTarget::Battle(id) => get_effective_controller(state, *id),
    }
}
