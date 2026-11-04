//! `options-named-by-what-they-are`: every option the engine offers is named
//! by what it is, so a game's log replays to the same game wherever a build
//! lists the options (`setup-architecture.md#7e-pausing-at-a-decision`).
//!
//! The board is Bridge from Below's ruling: a nontoken creature of yours and
//! one of an opponent's die in one event, both of its abilities trigger, and
//! you choose their order. The Zombie ability checks "if this card is in your
//! graveyard" again as it resolves, so the order decides whether a Zombie is
//! made.

use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use mtgsim::cards::printings::{grizzly_bears, lightning_bolt, mending_hands, words_of_worship};
use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::GeneratedBy;
use mtgsim::objects::card_data::CardData;
use mtgsim::objects::object::GameObject;
use mtgsim::state::decision_log::{self, AnswerLine, BuiltStart, Chosen, GameStart};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{fill_library, put_on_battlefield, setup_game, test_dp};
use mtgsim::types::effects::{AmountExpr, Effect, EffectRecipient, Primitive};
use mtgsim::types::ids::{ObjectId, ObjectOrPlayerId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::triggers::TriggerTier;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::{AbilityAsRead, ChoiceContext, ChoiceKind, ChoiceOption, ReplacementOption, TriggerOption, TriggeredAbility};
use mtgsim::ui::decision::{DecisionProvider, DispatchDecisionProvider, PriorityAction, ScriptedDecisionProvider, SeatMode};
use mtgsim::ui::display::{format_event_log, option_label};
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::replay::Replay;

const BRIDGE_AND_PYROCLASM: &str = "\
hand 0: Pyroclasm
graveyard 0: Bridge from Below
battlefield: Mountain | controller 0
battlefield: Mountain [second] | controller 0
battlefield: Grizzly Bears | controller 0
battlefield: Grizzly Bears [theirs] | controller 1
library 0: Mountain | x5
library 1: Mountain | x5
";

const CLONE_OF_THE_ELDER: &str = "\
hand 0: Clone
battlefield: Island | controller 0
battlefield: Island [second] | controller 0
battlefield: Island [third] | controller 0
battlefield: Island [fourth] | controller 0
battlefield: Thunder-Thrash Elder | controller 0, arrived this turn
battlefield: Grumgully, the Generous | controller 0, arrived this turn
library 0: Island | x5
library 1: Island | x5
";

/// The board built, recording its events and writing its log into the
/// returned text, as a client attaches the engine's writer.
fn recorded() -> (BuiltStart, Arc<Mutex<String>>) {
    let start = GameStart::Scenario { path: "bridge".to_string(), seed: 0, text: BRIDGE_AND_PYROCLASM.to_string() };
    let mut built = start.build(&CardRegistry::default_registry()).expect("the board builds");
    let text = Arc::new(Mutex::new(decision_log::opening(&start).iter().map(|line| format!("{line}\n")).collect::<String>()));
    let (writer, answers) = (Arc::clone(&text), Arc::new(Mutex::new(0)));
    let game = built.game_mut();
    game.state.record_events();
    game.state.log_decisions(move |game, decision| {
        let mut number = answers.lock().unwrap();
        *number += 1;
        writer.lock().unwrap().push_str(&format!("{}\n", AnswerLine::of(game, *number, decision)));
    });
    (built, text)
}

/// The one card named `name` that `controller` controls, or that is in a
/// graveyard or a hand when `controller` is `None`.
fn named(game: &GameState, name: &str, controller: Option<usize>) -> ObjectId {
    // ORDER-FREE: one card of each name and controller on this board.
    let found: Vec<ObjectId> = game
        .objects
        .values()
        .filter(|object| object.card_data.name == name)
        .filter(|object| controller.is_none_or(|seat| game.battlefield.get(&object.id).is_some_and(|entry| entry.controller == seat)))
        .map(|object| object.id)
        .collect();
    let [id] = found[..] else { panic!("{} cards named {name}", found.len()) };
    id
}

/// Bridge from Below's trigger of its ability at `place`, for `died`.
fn bridge_trigger(game: &GameState, bridge: ObjectId, place: usize, died: ObjectId) -> ChoiceOption {
    let ability = game.objects[&bridge].card_data.abilities[place - 1].id;
    ChoiceOption::Trigger(TriggerOption {
        source: game.object_ref(bridge).expect("in the store"),
        ability: TriggeredAbility::OfItsSource { ability: ability.definition(), granted_by: None, read_as: AbilityAsRead(None) },
        triggered_by: vec![died],
    })
}

/// Casts Pyroclasm, which kills both Bears at once, and orders Bridge from
/// Below's two triggers: the Zombie's on top when `token_first`, so it
/// resolves first. Returns the game and its log, read back.
fn played(token_first: bool) -> (BuiltStart, decision_log::Log) {
    let (mut built, record) = recorded();
    let state = &built.game().state;
    let pyroclasm = named(state, "Pyroclasm", None);
    let bridge = named(state, "Bridge from Below", None);
    let (yours, theirs) = (named(state, "Grizzly Bears", Some(0)), named(state, "Grizzly Bears", Some(1)));
    // ORDER-FREE: the two Mountains are alike; either pays.
    let mountains: Vec<ObjectId> = state.objects.values().filter(|o| o.card_data.name == "Mountain" && state.battlefield.contains_key(&o.id)).map(|o| o.id).collect();
    let tap = |id: ObjectId| ChoiceOption::Action(PriorityAction::ActivateAbility(id, state.objects[&id].card_data.abilities[0].id));
    let makes_a_zombie = bridge_trigger(state, bridge, 1, yours);
    let exiles_itself = bridge_trigger(state, bridge, 2, theirs);

    let caster = ScriptedDecisionProvider::new();
    caster.expect_choice(ChoiceKind::PriorityAction, vec![ChoiceOption::Action(PriorityAction::CastSpell(pyroclasm))]);
    let window = ChoiceKind::ManaAbilityWindow { spell_or_ability_id: pyroclasm, remaining_cost: ManaCost::build(&[], 0) };
    caster.expect_choice(window.clone(), vec![tap(mountains[0])]);
    caster.expect_choice(window, vec![tap(mountains[1])]);
    // The first item goes on the stack lowest, so the last resolves first.
    let order = if token_first { vec![exiles_itself, makes_a_zombie] } else { vec![makes_a_zombie, exiles_itself] };
    caster.expect_order(ChoiceKind::OrderTriggers { player: 0, tier: TriggerTier::First }, order);
    let seats = DispatchDecisionProvider::new(vec![Box::new(ManaWindowStop::new(caster)), Box::new(ScriptedDecisionProvider::new())]);
    built.game_mut().until_stopped(|game| game.resume_turn_at_priority(&seats)).expect("the turn plays");

    let log = decision_log::read(&record.lock().unwrap()).expect("the log reads");
    (built, log)
}

/// A provider that hands `0` every list reversed and maps its answer back, as
/// a build listing the options in another order would.
struct Reversed<'a>(&'a dyn DecisionProvider);

impl Reversed<'_> {
    fn back(listed: usize, answer: Vec<usize>) -> Vec<usize> {
        answer.into_iter().map(|at| listed - 1 - at).collect()
    }
}

impl DecisionProvider for Reversed<'_> {
    fn pick_n(&self, game: &GameState, player: PlayerId, ctx: &ChoiceContext, options: &[ChoiceOption], bounds: (usize, usize)) -> Vec<usize> {
        let reversed: Vec<ChoiceOption> = options.iter().rev().cloned().collect();
        Self::back(options.len(), self.0.pick_n(game, player, ctx, &reversed, bounds))
    }

    fn pick_number(&self, game: &GameState, player: PlayerId, ctx: &ChoiceContext, min: u64, max: u64) -> u64 {
        self.0.pick_number(game, player, ctx, min, max)
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        ctx: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        let reversed: Vec<ChoiceOption> = buckets.iter().rev().cloned().collect();
        let mins: Vec<u64> = per_bucket_mins.iter().rev().copied().collect();
        let maxs: Option<Vec<u64>> = per_bucket_maxs.map(|maxs| maxs.iter().rev().copied().collect());
        let mut given = self.0.allocate(game, player, ctx, total, &reversed, &mins, maxs.as_deref());
        given.reverse();
        given
    }

    fn choose_ordering(&self, game: &GameState, player: PlayerId, ctx: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        let reversed: Vec<ChoiceOption> = items.iter().rev().cloned().collect();
        Self::back(items.len(), self.0.choose_ordering(game, player, ctx, &reversed))
    }

    fn seat_mode(&self, player: PlayerId) -> SeatMode {
        self.0.seat_mode(player)
    }
}

fn zombies(built: &BuiltStart) -> usize {
    // ORDER-FREE: a count.
    built.game().state.battlefield.keys().filter(|id| built.game().state.objects[id].card_data.name.starts_with("Zombie")).count()
}

/// The log names each of the Bridge's two triggers by its ability and the
/// creature it is for, and a replay of each order plays it again, offered
/// every list reversed: a Zombie when its trigger resolved first, none when
/// the exile did.
#[test]
fn bridge_from_below_replays_its_trigger_order_both_ways() {
    for token_first in [true, false] {
        let (original, log) = played(token_first);
        let state = &original.game().state;
        let bridge = named(state, "Bridge from Below", None);
        assert_eq!(state.objects[&bridge].zone, Zone::Exile, "the exile resolved either way");
        assert_eq!(zombies(&original), usize::from(token_first), "token_first: {token_first}");

        let ordered = log
            .answers
            .iter()
            .find_map(|line| match &line.chosen {
                Chosen::Order(items) if line.kind == "OrderTriggers" => Some(items.clone()),
                _ => None,
            })
            .expect("the order is a line of the log");
        assert!(
            ordered.iter().all(|item| item.starts_with(&format!("Bridge from Below ({bridge}), ability "))),
            "each trigger names its ability: {ordered:?}"
        );
        assert_ne!(ordered[0], ordered[1], "and the two read apart");

        let mut replayed = log.start.build(&CardRegistry::default_registry()).expect("the log's start builds");
        replayed.game_mut().state.record_events();
        let replay = Replay::of(log);
        replayed.game_mut().until_stopped(|game| game.resume_turn_at_priority(&Reversed(&replay))).expect("the replay plays");
        assert_eq!(replay.remaining(), 0, "every answer the log holds was asked for");
        assert_eq!(format_event_log(&replayed.game().state), format_event_log(&original.game().state));
        assert_eq!(zombies(&replayed), zombies(&original));
    }
}

/// A provider that answers from `inner` and keeps what a client draws for
/// each option of CR 616.1's prompt.
struct DrawsReplacements<'a> {
    inner: &'a dyn DecisionProvider,
    drawn: RefCell<Vec<String>>,
}

impl DecisionProvider for DrawsReplacements<'_> {
    fn pick_n(&self, game: &GameState, player: PlayerId, ctx: &ChoiceContext, options: &[ChoiceOption], bounds: (usize, usize)) -> Vec<usize> {
        if matches!(ctx.kind, ChoiceKind::ChooseReplacementEffect { .. }) {
            self.drawn.borrow_mut().extend(options.iter().map(|option| option_label(game, option)));
        }
        self.inner.pick_n(game, player, ctx, options, bounds)
    }

    fn pick_number(&self, game: &GameState, player: PlayerId, ctx: &ChoiceContext, min: u64, max: u64) -> u64 {
        self.inner.pick_number(game, player, ctx, min, max)
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        ctx: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        self.inner.allocate(game, player, ctx, total, buckets, per_bucket_mins, per_bucket_maxs)
    }

    fn choose_ordering(&self, game: &GameState, player: PlayerId, ctx: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        self.inner.choose_ordering(game, player, ctx, items)
    }

    fn seat_mode(&self, player: PlayerId) -> SeatMode {
        self.inner.seat_mode(player)
    }
}

/// A Clone copying Thunder-Thrash Elder has devour only in the frame it enters
/// with (CR 614.12), which no label can read off the board, so the option
/// carries the ability's text as the engine read it. Cast from hand with exact
/// mana, beside Grumgully's counter, so CR 616.1 asks.
#[test]
fn a_copied_ability_reads_as_its_text_while_the_copy_enters() {
    let start = GameStart::Scenario { path: "clone".to_string(), seed: 0, text: CLONE_OF_THE_ELDER.to_string() };
    let mut built = start.build(&CardRegistry::default_registry()).expect("the board builds");
    let state = &built.game().state;
    let clone = named(state, "Clone", None);
    let elder = named(state, "Thunder-Thrash Elder", Some(0));
    let grumgully = named(state, "Grumgully, the Generous", Some(0));
    // ORDER-FREE: the four Islands are alike; any pays.
    let islands: Vec<ObjectId> = state.objects.values().filter(|o| o.card_data.name == "Island" && state.battlefield.contains_key(&o.id)).map(|o| o.id).collect();
    let tap = |id: ObjectId| ChoiceOption::Action(PriorityAction::ActivateAbility(id, state.objects[&id].card_data.abilities[0].id));

    let caster = ScriptedDecisionProvider::new();
    caster.expect_choice(ChoiceKind::PriorityAction, vec![ChoiceOption::Action(PriorityAction::CastSpell(clone))]);
    let window = ChoiceKind::ManaAbilityWindow { spell_or_ability_id: clone, remaining_cost: ManaCost::build(&[], 0) };
    for island in islands {
        caster.expect_choice(window.clone(), vec![tap(island)]);
    }
    caster.expect_choice(ChoiceKind::ChooseCopySource { source: clone }, vec![ChoiceOption::Object(elder)]);
    caster.expect_choice(ChoiceKind::ChooseReplacementEffect { affected_object: None }, vec![ChoiceOption::Object(grumgully)]);
    caster.expect_choice(ChoiceKind::ChooseAuxiliaryZoneChange { entering: clone, source: clone, to: Zone::Graveyard }, Vec::new());
    let seats = DispatchDecisionProvider::new(vec![Box::new(ManaWindowStop::new(caster)), Box::new(ScriptedDecisionProvider::new())]);
    let drawing = DrawsReplacements { inner: &seats, drawn: RefCell::new(Vec::new()) };
    built.game_mut().until_stopped(|game| game.resume_turn_at_priority(&drawing)).expect("the turn plays");

    let drawn = drawing.drawn.into_inner();
    assert!(drawn.contains(&format!("Clone ({clone}) · Devour 3")), "the copied devour reads as itself: {drawn:?}");
}

/// Two activations of Words of Worship leave two rows that read alike: a
/// person sees the ability's text, and the stack object tells them apart.
#[test]
fn a_row_an_ability_made_reads_as_its_text_and_its_stack_object() {
    let mut game = setup_game(2);
    fill_library(&mut game, 0, 2);
    let words = put_on_battlefield(&mut game, words_of_worship(), 0);
    let mut activate = || {
        game.players[0].mana_pool.add(ManaType::Colorless, 1);
        game.activate_ability(0, words, 0, &ManaWindowStop::new(test_dp())).expect("{1} is in the pool");
        let ability = *game.stack.last().expect("the ability is on the stack");
        game.resolve_top_of_stack(&test_dp()).expect("the ability resolves");
        ability
    };
    let (first, second) = (activate(), activate());

    let answers = ScriptedDecisionProvider::new();
    let either = ChoiceOption::Replacement(ReplacementOption::FromResolution { source: words, resolution: second });
    answers.expect_choice(ChoiceKind::ChooseReplacementEffect { affected_object: None }, vec![either]);
    let drawing = DrawsReplacements { inner: &answers, drawn: RefCell::new(Vec::new()) };
    let draw = Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Controller);
    game.resolve_effect(&draw, &ResolutionContext::untargeted(words, 0, GeneratedBy::ActivatedAbility), &drawing).expect("the draw resolves");

    let drawn = drawing.drawn.into_inner();
    let text = "{1}: The next time you would draw a card this turn, you gain 5 life instead.";
    for made_by in [first, second] {
        assert!(drawn.contains(&format!("Words of Worship ({words}) · {text} ({made_by})")), "{drawn:?}");
    }
}

/// A spell is the object that made its row, so the card's id already tells two
/// apart and the label does not print it twice.
#[test]
fn a_row_a_spell_made_reads_as_its_cards_text() {
    let mut game = setup_game(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let at_the_bears = || ChosenTargets::one(vec![ObjectOrPlayerId::Object(bears)]);
    let mut resolved = |card: Arc<CardData>, dp: &dyn DecisionProvider| {
        let id = game.add_object(GameObject::new(Arc::clone(&card), 0, Zone::Stack));
        let ctx = ResolutionContext { targets: at_the_bears(), ..ResolutionContext::untargeted(id, 0, GeneratedBy::Spell) };
        game.resolve_effect(&card.abilities[0].effect, &ctx, dp).expect("the spell resolves");
        id
    };
    let (one, other) = (resolved(mending_hands(), &test_dp()), resolved(mending_hands(), &test_dp()));

    let answers = ScriptedDecisionProvider::new();
    answers.expect_choice(ChoiceKind::ChooseReplacementEffect { affected_object: None }, vec![ChoiceOption::Object(one)]);
    let drawing = DrawsReplacements { inner: &answers, drawn: RefCell::new(Vec::new()) };
    resolved(lightning_bolt(), &drawing);

    let drawn = drawing.drawn.into_inner();
    let text = "Prevent the next 4 damage that would be dealt to any target this turn.";
    for hands in [one, other] {
        assert!(drawn.contains(&format!("Mending Hands ({hands}) · {text}")), "{drawn:?}");
    }
}
