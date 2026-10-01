//! A [`Scenario`] built into a `Game` (`setup-architecture.md` §3): every
//! write through the engine's own doors, in file order, before the first
//! turn, and nothing emitted. §4.1's refusals are here.

use std::sync::Arc;

use super::board::{Arrival, CardLine, CardRef, Head, HistoryRow, Located, PlayerFact, Scenario, Target, Word};
use super::refusal::{Refusal, RefusalKind};
use super::text::{position_word, positions};
use crate::cards::registry::CardRegistry;
use crate::objects::card_data::{AbilityType, CardData};
use crate::objects::object::GameObject;
use crate::oracle::characteristics::{get_effective_controller, get_effective_types};
use crate::state::battlefield::{AttackTarget, AttackingInfo, BlockingInfo, PermanentState};
use crate::state::game::{starting_player_skips_first_draw, Game, Streams};
use crate::state::game_config::GameConfig;
use crate::state::game_state::{AbilityIdentity, GameState, Phase, PhaseType, StepType};
use crate::state::history::PlayerHistory;
use crate::types::card_types::CardType;
use crate::types::history::TurnFact;
use crate::types::ids::{ObjectId, ObjectRef, PlayerId};
use crate::types::zones::Zone;

impl Scenario {
    /// Build the board this describes, at the start of `step`'s priority
    /// round with the active player to act; `Game::resume` plays it. Names
    /// are looked up in `registry`, then among its cards in development.
    pub fn build(&self, registry: &CardRegistry) -> Result<Game, Refusal> {
        let mut loader = Loader::new(self)?;
        for line in &self.cards {
            loader.card_line(line, registry)?;
        }
        loader.combat()?;
        loader.player_facts()?;
        loader.turn()?;
        loader.this_turn()?;
        let state = loader.state;
        Ok(Game { state, config: GameConfig { starting_life: self.starting_life, ..GameConfig::unrestricted() } })
    }
}

fn refusal(kind: RefusalKind, line: usize, message: impl Into<String>) -> Refusal {
    Refusal::at(kind, line, message)
}

fn show(card: &CardRef) -> String {
    match &card.tag {
        Some(tag) => format!("{} [{tag}]", card.name),
        None => card.name.clone(),
    }
}

/// An object the file created, and what a reference to it can match.
struct Created {
    line: usize,
    id: ObjectId,
    card: CardRef,
    zone: Zone,
    commander: bool,
}

struct Loader<'s> {
    scenario: &'s Scenario,
    state: GameState,
    created: Vec<Created>,
    /// Combat words, applied once every permanent exists.
    attackers: Vec<(usize, ObjectId, &'s Target)>,
    blocked: Vec<(usize, ObjectId)>,
    blockers: Vec<(usize, ObjectId, &'s CardRef)>,
    /// Whether each player's library lines are `shuffled`, once one is seen.
    shuffled: Vec<Option<bool>>,
}

impl<'s> Loader<'s> {
    fn new(scenario: &'s Scenario) -> Result<Loader<'s>, Refusal> {
        let whole = |message: String| Refusal { kind: RefusalKind::Unreachable, line: None, message };
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
        state.reseed(Streams::from_seed(scenario.seed).game);
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

    fn seat(&self, player: PlayerId, line: usize) -> Result<PlayerId, Refusal> {
        if player < self.scenario.players {
            Ok(player)
        } else {
            Err(refusal(RefusalKind::Reference, line, format!("player {player} is not in a {}-player game", self.scenario.players)))
        }
    }

    /// The object `card` names among those created so far that `among`
    /// keeps, refusing a name that matches none, or two.
    fn resolve(&self, card: &CardRef, line: usize, what: &str, among: impl Fn(&Created) -> bool) -> Result<ObjectId, Refusal> {
        let matches: Vec<&Created> =
            self.created.iter().filter(|c| among(c) && c.card.name == card.name && (card.tag.is_none() || c.card.tag == card.tag)).collect();
        match matches.as_slice() {
            [one] => Ok(one.id),
            [] => Err(refusal(RefusalKind::Reference, line, format!("{} names no {what} listed above this line", show(card)))),
            [first, second, ..] => Err(refusal(
                RefusalKind::Reference,
                line,
                format!("{} names two {what}s, lines {} and {}; give each a tag, as `{} [a]`", show(card), first.line, second.line, card.name),
            )),
        }
    }

    fn permanent(&self, card: &CardRef, line: usize) -> Result<ObjectId, Refusal> {
        self.resolve(card, line, "permanent", |c| c.zone == Zone::Battlefield)
    }

    fn card_line(&mut self, located: &'s Located<CardLine>, registry: &CardRegistry) -> Result<(), Refusal> {
        let (line, card_line) = (located.line, &located.item);
        match card_line.head {
            Head::Counters => {
                let id = self.permanent(&card_line.card, line)?;
                for word in &card_line.words {
                    if let Word::Counter(kind, count) = word {
                        self.set_counters(id, *kind, *count);
                    }
                }
                return Ok(());
            }
            // Read once the turn has begun, which clears this turn's sets.
            Head::ThisTurn => return Ok(()),
            _ => {}
        }
        let name = &card_line.card.name;
        let card = registry.create(name).ok().or_else(|| registry.create_in_development(name)).ok_or_else(|| {
            refusal(
                RefusalKind::NotACard,
                line,
                format!("{name} is not registered: names are exact, and a card the engine does not play yet goes in the list of cards in development"),
            )
        })?;
        let owner_word = card_line.words.iter().find_map(|w| if let Word::Owner(p) = w { Some(*p) } else { None });
        let controller_word = card_line.words.iter().find_map(|w| if let Word::Controller(p) = w { Some(*p) } else { None });
        let (zone, owner) = match card_line.head {
            Head::Hand(p) => (Zone::Hand, p),
            Head::Library { player, shuffled } => {
                let seat = self.seat(player, line)?;
                if *self.shuffled[seat].get_or_insert(shuffled) != shuffled {
                    let message = format!("library {player} has both shuffled and listed lines; a shuffle orders the whole library");
                    return Err(refusal(RefusalKind::Syntax, line, message));
                }
                (Zone::Library, player)
            }
            Head::Graveyard(p) => (Zone::Graveyard, p),
            Head::Exile | Head::Command => {
                let owner = owner_word.ok_or_else(|| refusal(RefusalKind::Syntax, line, format!("{name} needs `owner p`: whose card is it?")))?;
                (if card_line.head == Head::Exile { Zone::Exile } else { Zone::Command }, owner)
            }
            _ => {
                let owner = owner_word.or(controller_word).ok_or_else(|| {
                    refusal(RefusalKind::Syntax, line, format!("{name} needs `controller p` or `owner p`; each defaults to the other"))
                })?;
                (Zone::Battlefield, owner)
            }
        };
        self.seat(owner, line)?;
        for _ in 0..card_line.copies {
            let id = self.create(&card, zone, owner, controller_word, card_line, line)?;
            let commander = card_line.words.contains(&Word::Commander);
            self.created.push(Created { line, id, card: card_line.card.clone(), zone, commander });
        }
        Ok(())
    }

    fn create(
        &mut self,
        card: &Arc<CardData>,
        zone: Zone,
        owner: PlayerId,
        controller: Option<PlayerId>,
        card_line: &'s CardLine,
        line: usize,
    ) -> Result<ObjectId, Refusal> {
        let mut obj = GameObject::new(Arc::clone(card), owner, zone);
        obj.is_commander = card_line.words.contains(&Word::Commander);
        let unreachable = |message: String| refusal(RefusalKind::Unreachable, line, message);
        if zone != Zone::Battlefield {
            let id = self.state.create_in_zone(obj).map_err(unreachable)?;
            if let Head::Library { player, .. } = card_line.head {
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
        let arrived = match card_line.words.iter().find_map(|w| if let Word::Arrived(a) = w { Some(*a) } else { None }) {
            None => 0,
            Some(Arrival::ThisTurn) => turn,
            Some(Arrival::Turn(n)) if (1..=turn).contains(&n) => n,
            Some(Arrival::Turn(n)) => return Err(unreachable(format!("`arrived turn {n}` is not a turn before turn {turn}"))),
        };
        let controller = self.seat(controller.unwrap_or(owner), line)?;
        let id = self.state.create_on_battlefield(obj, controller, arrived).map_err(unreachable)?;
        for word in &card_line.words {
            match word {
                Word::Tapped => self.write(id, |entry| entry.tapped = true),
                Word::Counter(kind, count) => self.set_counters(id, *kind, *count),
                Word::Damage(damage) => self.write(id, |entry| entry.damage_marked = *damage),
                Word::DealtFirstStrikeDamage => {
                    self.state.dealt_first_strike_damage.insert(id);
                }
                Word::AttachedTo(host) => {
                    let host = self.permanent(host, line)?;
                    self.state.attach(id, host);
                }
                Word::Attacking(target) => self.attackers.push((line, id, target)),
                Word::Blocked => self.blocked.push((line, id)),
                Word::Blocking(attacker) => self.blockers.push((line, id, attacker)),
                // Read as the object was created.
                Word::Owner(_) | Word::Controller(_) | Word::Commander | Word::Arrived(_) => {}
                Word::Triggered { .. } | Word::Resolved { .. } | Word::TookOnceEachTurnAction { .. } => {}
            }
        }
        Ok(id)
    }

    /// Write to `id`'s battlefield entry, one the loader created.
    fn write(&mut self, id: ObjectId, change: impl FnOnce(&mut PermanentState)) {
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
        let index = |phase: Phase| positions().position(|p| p == phase);
        self.scenario.step.phase_type == PhaseType::Combat
            && index(self.scenario.step) >= index(Phase { phase_type: PhaseType::Combat, step: Some(from) })
    }

    fn combat(&mut self) -> Result<(), Refusal> {
        let (active, step) = (self.scenario.active, position_word(self.scenario.step));
        let unreachable = |line: usize, message: String| refusal(RefusalKind::Unreachable, line, message);
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
                Target::Player(player) => AttackTarget::Player(self.seat(*player, line)?),
                Target::Permanent(card) => {
                    let attacked = self.permanent(card, line)?;
                    let types = get_effective_types(&self.state, attacked);
                    if types.contains(&CardType::Planeswalker) {
                        AttackTarget::Planeswalker(attacked)
                    } else if types.contains(&CardType::Battle) {
                        AttackTarget::Battle(attacked)
                    } else {
                        return Err(unreachable(line, format!("{name} attacks {}, which is neither a planeswalker nor a battle (CR 508.1b)", show(card))));
                    }
                }
            };
            if defender(&self.state, &target) == Some(active) {
                return Err(unreachable(line, format!("{name} attacks its own controller's side; an attacker attacks an opponent (CR 508.1b)")));
            }
            let is_blocked = self.blocked.iter().any(|&(_, blocked)| blocked == id);
            self.write(id, |entry| entry.attacking = Some(AttackingInfo { target, is_blocked, blocked_by: Vec::new() }));
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
            let attacker_id = self.permanent(attacker, line)?;
            let Some(attacked) = self.state.battlefield[&attacker_id].attacking.as_ref().map(|a| defender(&self.state, &a.target)) else {
                return Err(unreachable(line, format!("{name} blocks {}, which is not attacking (CR 509.1a)", show(attacker))));
            };
            if get_effective_controller(&self.state, blocker) != attacked {
                return Err(unreachable(line, format!("{name} blocks {}, and only the player it attacks blocks it (CR 509.1a)", show(attacker))));
            }
            self.write(blocker, |entry| entry.blocking = Some(BlockingInfo { blocking: vec![attacker_id] }));
            self.write(attacker_id, |entry| {
                if let Some(attacking) = entry.attacking.as_mut() {
                    attacking.is_blocked = true;
                    attacking.blocked_by.push(blocker);
                }
            });
        }
        if !self.state.dealt_first_strike_damage.is_empty() && !self.in_combat_from(StepType::FirstStrikeDamage) {
            let message = format!("first-strike damage is dealt in the first-strike damage step (CR 510.4), not before it, in the {step}");
            return Err(Refusal { kind: RefusalKind::Unreachable, line: None, message });
        }
        Ok(())
    }

    fn player_facts(&mut self) -> Result<(), Refusal> {
        for located in &self.scenario.player_facts {
            let line = located.line;
            let (PlayerFact::Life { player, .. }
            | PlayerFact::Counter { player, .. }
            | PlayerFact::LandsPlayed { player, .. }
            | PlayerFact::LeftTheGame { player }
            | PlayerFact::CommanderDamage { player, .. }
            | PlayerFact::History { player, .. }) = located.item;
            let player = self.seat(player, line)?;
            match &located.item {
                PlayerFact::Life { life, .. } => self.state.players[player].life_total = *life,
                PlayerFact::Counter { kind, count, .. } => self.state.players[player].add_counters(*kind, *count),
                PlayerFact::LandsPlayed { count, .. } => self.state.players[player].lands_played_this_turn = *count,
                PlayerFact::LeftTheGame { .. } => self.left_the_game(player, line)?,
                PlayerFact::CommanderDamage { damage, from, .. } => {
                    let commander = self.resolve(from, line, "commander", |c| c.commander)?;
                    self.state.players[player].commander_damage_taken.insert(commander, *damage);
                }
                PlayerFact::History { .. } => {}
            }
        }
        Ok(())
    }

    /// CR 800.4a: a player who has left owns and controls nothing, and the
    /// game has two players left (CR 104.2a) and an active one.
    fn left_the_game(&mut self, player: PlayerId, line: usize) -> Result<(), Refusal> {
        let theirs = self.created.iter().find(|c| {
            self.state.objects[&c.id].owner == player || get_effective_controller(&self.state, c.id) == Some(player)
        });
        let message = if let Some(card) = theirs {
            format!("player {player} has left, and owns or controls {} on line {} (CR 800.4a)", show(&card.card), card.line)
        } else if player == self.scenario.active {
            format!("player {player} has left, and is the active player: a turn begins only for a player in the game (CR 800.4k)")
        } else if self.state.player_lost.iter().filter(|&&lost| !lost).count() <= 2 {
            "a game with fewer than two players left in it is over (CR 104.2a)".to_string()
        } else {
            self.state.player_lost[player] = true;
            return Ok(());
        };
        Err(refusal(RefusalKind::Unreachable, line, message))
    }

    /// The turns the natural rotation over the players in the game gives,
    /// ending with the active player's turn T, each begun through the turn's
    /// writers; then the position, and each history row on its turn.
    fn turn(&mut self) -> Result<(), Refusal> {
        let (turn, active) = (self.scenario.turn, self.scenario.active);
        let in_game: Vec<PlayerId> = (0..self.scenario.players).filter(|&p| self.state.in_game(p)).collect();
        let place = in_game.iter().position(|&p| p == active).unwrap_or_default();
        let rows = self.history_rows()?;
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
        if self.in_combat_from(StepType::DeclareBlockers) && !attacked {
            return Err(Refusal {
                kind: RefusalKind::Unreachable,
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
    fn history_rows(&self) -> Result<Vec<(PlayerId, u32, TurnFact, u64)>, Refusal> {
        let turn = self.scenario.turn;
        let mut totals: Vec<(PlayerId, TurnFact, [u64; 2], Option<(usize, u64)>)> = Vec::new();
        for located in &self.scenario.player_facts {
            let PlayerFact::History { player, row, fact, count } = located.item else { continue };
            let index = match totals.iter().position(|t| t.0 == player && t.1 == fact) {
                Some(index) => index,
                None => {
                    totals.push((player, fact, [0, 0], None));
                    totals.len() - 1
                }
            };
            match row {
                HistoryRow::ThisTurn => totals[index].2[0] += count,
                HistoryRow::LastTurn if turn > 1 => totals[index].2[1] += count,
                HistoryRow::LastTurn => {
                    return Err(refusal(RefusalKind::Unreachable, located.line, "turn 1 has no last turn: the game began with it"));
                }
                HistoryRow::ThisGame => totals[index].3 = Some((located.line, count)),
            }
        }
        let mut rows = Vec::new();
        for (player, fact, [this, last], game) in totals {
            rows.push((player, turn, fact, this));
            rows.push((player, turn.saturating_sub(1), fact, last));
            let Some((line, game)) = game else { continue };
            let earlier = game.checked_sub(this + last).ok_or_else(|| {
                refusal(RefusalKind::Unreachable, line, "this game counts fewer than this turn and last turn together")
            })?;
            if earlier > 0 && turn < 3 {
                return Err(refusal(RefusalKind::Unreachable, line, format!("turn {turn} has no turn before last to count the rest on")));
            }
            rows.push((player, turn.saturating_sub(2), fact, earlier));
        }
        Ok(rows)
    }

    /// CR 603.2h and 603.7h's counts, once turn T has begun and cleared them.
    fn this_turn(&mut self) -> Result<(), Refusal> {
        for located in &self.scenario.cards {
            let (line, card_line) = (located.line, &located.item);
            if card_line.head != Head::ThisTurn {
                continue;
            }
            let source = self.permanent(&card_line.card, line)?;
            let card = Arc::clone(&self.state.objects[&source].card_data);
            let reference = ObjectRef { id: source, zone_change_epoch: self.state.objects[&source].zone_change_epoch };
            for word in &card_line.words {
                // CR 603.2h's limit is a triggered ability's; CR 603.7h counts
                // either kind resolving.
                let (ability, can) = match word {
                    Word::Triggered { ability } | Word::TookOnceEachTurnAction { ability } => (ability, AbilityType::Triggered),
                    Word::Resolved { ability, .. } => (ability, AbilityType::Activated),
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
                    refusal(RefusalKind::Reference, line, format!("name the ability: `ability N`, N its place among {}'s printed abilities", card.name))
                })?;
                let identity = AbilityIdentity { source: reference, ability: ability.id };
                match word {
                    Word::Triggered { .. } => {
                        self.state.triggered_this_turn.insert(identity);
                    }
                    Word::Resolved { times, .. } => {
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
fn defender(state: &GameState, target: &AttackTarget) -> Option<PlayerId> {
    match target {
        AttackTarget::Player(player) => Some(*player),
        AttackTarget::Planeswalker(id) | AttackTarget::Battle(id) => get_effective_controller(state, *id),
    }
}
