//! Every prompt a game raises says which object it is about.
//!
//! `backlog.md` §2.21: a `DecisionContext` is supposed to carry enough that a
//! client can highlight *what* the question is about, and until this test
//! nothing enforced it. The discipline lived in prose on each variant; the one
//! consumer in the tree, `ui/cli.rs`, is omniscient; and
//! `RandomDecisionProvider` picks by index without asking what a prompt means,
//! so the fuzz harness cannot notice a variant that dropped its source.
//! `DecisionKind::subject` is the contract (`codebase-state.md` item 141's
//! payload rule): its exhaustive match makes a new variant decide at compile
//! time, and this file checks the runtime half — whole games at two seats and
//! four, each deck a different sixty-card window of the registry, plus one of
//! each variant built by hand for the prompts no registered card raises.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use mtgsim::cards::registry::CardRegistry;
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::{GameState, PhaseType, StepType};
use mtgsim::types::effects::{EffectRecipient, SelectionFilter, TargetCount};
use mtgsim::types::ids::{ObjectId, ObjectOrPlayerId, PlayerId};
use mtgsim::types::mana::ManaCost;
use mtgsim::types::triggers::TriggerTier;
use mtgsim::types::zones::Zone;
use mtgsim::types::costs::Cost;
use mtgsim::decision::decision_types::{AmountOf, DecisionContext, DecisionKind, DecisionOption, EntryOffer, EntryRole};
use mtgsim::oracle::x_values::LegalXValues;
use mtgsim::decision::provider::DecisionProvider;
use mtgsim::ui::random::RandomDecisionProvider;

/// The variants that may answer `subject()` with `None`, and the rule that
/// makes each one so. Beside the assertion rather than in the engine so that
/// adding a name here is a visible decision in review; `DecisionKind::subject`'s
/// doc is the same list with the reasons. `Discard` is not here: its `None` is
/// allowed only in the cleanup step, which the recorder checks.
const SUBJECT_LESS: &[(&str, &str)] = &[
    ("PriorityAction", "CR 117.1"),
    ("DeclareAttackers", "CR 508.1a"),
    ("DeclareBlockers", "CR 509.1a"),
    ("LegendRule", "CR 704.5j"),
    // `affected_object: None` — the event is about the choosing player.
    ("ChooseReplacementEffect", "CR 616.1"),
    // Several triggers' sources at once, as `DeclareAttackers` has attackers.
    ("OrderTriggers", "CR 603.3b"),
    // Several objects at once, the same way.
    ("OrderSimultaneousEntries", "CR 613.7m"),
];

/// One prompt as the engine raised it: what `subject()` said, and whether the
/// game was in its cleanup step — the one place a `Discard` may have no
/// source (CR 514.1).
struct Raised {
    variant: String,
    subject: Option<ObjectId>,
    at_cleanup: bool,
}

impl Raised {
    fn of(kind: &DecisionKind, at_cleanup: bool) -> Self {
        Raised { variant: kind.as_str().to_string(), subject: kind.subject(), at_cleanup }
    }

    /// What every prompt must satisfy, whichever way it was built.
    fn check(&self) {
        if self.subject.is_some() {
            return;
        }
        let allowed = SUBJECT_LESS.iter().any(|(name, _)| *name == self.variant)
            || (self.variant == "Discard" && self.at_cleanup);
        assert!(allowed, "{} was asked without a subject", self.variant);
    }
}

/// `RandomDecisionProvider` with every prompt recorded on the way through.
struct Recorder {
    inner: RandomDecisionProvider,
    raised: RefCell<Vec<Raised>>,
}

impl Recorder {
    fn seeded(seed: u64) -> Self {
        Recorder {
            inner: RandomDecisionProvider::seeded(seed),
            raised: RefCell::new(Vec::new()),
        }
    }

    fn record(&self, game: &GameState, ctx: &DecisionContext) {
        let at_cleanup = matches!(
            (game.phase.phase_type, game.phase.step),
            (PhaseType::Ending, Some(StepType::Cleanup))
        );
        self.raised.borrow_mut().push(Raised::of(&ctx.kind, at_cleanup));
    }
}

impl DecisionProvider for Recorder {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &DecisionContext,
        options: &[DecisionOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        self.record(game, context);
        self.inner.pick_n(game, player, context, options, bounds)
    }

    fn pick_number(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &DecisionContext,
        min: u64,
        max: u64,
    ) -> u64 {
        self.record(game, context);
        self.inner.pick_number(game, player, context, min, max)
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &DecisionContext,
        total: u64,
        buckets: &[DecisionOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        self.record(game, context);
        self.inner.allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs)
    }

    fn choose_ordering(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &DecisionContext,
        items: &[DecisionOption],
    ) -> Vec<usize> {
        self.record(game, context);
        self.inner.choose_ordering(game, player, context, items)
    }
}

/// Sixty registered cards, every deck alike, the way `determinism_test`
/// builds a game, at `players` seats for up to `turns` turns. `window` says
/// which sixty: `card_names` is sorted, so the deck is the same in every
/// process, and successive windows walk the registry rather than its head.
fn walk(seed: u64, players: usize, turns: usize, window: usize) -> Vec<Raised> {
    let registry = CardRegistry::default_registry();
    let deck: Vec<_> = registry
        .card_names()
        .iter()
        .cycle()
        .skip(window * 60)
        .take(60)
        .filter_map(|name| registry.create(name).ok())
        .collect();
    let mut game = Game::new(GameConfig::test(), vec![deck; players]).expect("game creation");
    game.reseed(seed);
    let dp = Recorder::seeded(seed);
    game.setup(&dp).expect("setup");
    let mut played = 0;
    while !game.is_over() && played < turns {
        game.run_turn(&dp).expect("turn");
        played += 1;
    }
    dp.raised.into_inner()
}

/// Whole games, two seats and four: nothing the engine raises is without a
/// subject it did not declare.
#[test]
fn every_prompt_a_game_raises_says_what_it_is_about() {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut total = 0;
    let games = [(0xA4_0001u64, 2), (0xA4_0039, 2), (0xA4_0004, 4), (0xA4_0005, 4)];
    // The decks are windows of the sorted registry, so a card registered
    // ahead of a window changes it; fifty turns raise every kind below on
    // the registry as TR-3b left it, where forty raised no blockers. A card
    // registered ahead of a window can leave its game one that blocks nothing
    // or asks little, and its seed is re-picked then; the commit that moves
    // one says what it read.
    for (window, (seed, players)) in games.into_iter().enumerate() {
        for raised in walk(seed, players, 50, window) {
            raised.check();
            *seen.entry(raised.variant).or_default() += 1;
            total += 1;
        }
    }
    // What the walk reached, for `--nocapture` and the record.
    eprintln!("{total} prompts: {seen:?}");
    // The guard: a walk that raised little would pass the checks for free.
    // About 530 since the engine took the priority prompts that offer `Pass`
    // alone (`engine::priority`), which were most of the 1,000 this once read.
    assert!(total >= 400, "only {total} prompts across four games: {seen:?}");
    for must in [
        "PriorityAction",
        "DeclareAttackers",
        "DeclareBlockers",
        "SelectRecipients",
        "ManaAbilityWindow",
        "Discard",
    ] {
        assert!(seen.contains_key(must), "no game raised {must}: {seen:?}");
    }
}

/// One of each variant, built by hand: the walk reaches what random games
/// over the registry happen to ask, and this reaches the rest — the
/// alternative and additional cost prompts no registered card raises, the
/// commander SBA, the replacement and copy prompts. The count at the end is
/// what tells the author of a new variant to add its fixture here.
#[test]
fn every_variant_decides_its_subject() {
    let id = ObjectId::UNASSIGNED;
    let kinds = vec![
        DecisionKind::PriorityAction,
        DecisionKind::DeclareAttackers,
        DecisionKind::DeclareBlockers,
        DecisionKind::AssignCombatDamage { attacker_id: id },
        DecisionKind::AssignTrampleDamage { attacker_id: id, defending_target: ObjectOrPlayerId::Player(1) },
        DecisionKind::ChooseXValue { spell_id: id, values: Box::new(LegalXValues::zero_through(2)) },
        DecisionKind::ChooseAlternativeCost { spell_id: id },
        DecisionKind::ChooseAdditionalCosts { spell_id: id },
        DecisionKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Creature, TargetCount::UpTo(2)),
            spell_id: id,
        },
        DecisionKind::GenericManaAllocation { spell_or_ability_id: id, mana_cost: ManaCost::build(&[], 2) },
        DecisionKind::OrderCostReductions { spell_id: id },
        DecisionKind::OrderCostPayments { spell_or_ability_id: id },
        DecisionKind::OrderSimultaneousEntries { player: 0 },
        DecisionKind::ManaAbilityWindow { spell_or_ability_id: id, remaining_cost: ManaCost::build(&[], 1) },
        DecisionKind::ChooseSacrificeForCost { spell_or_ability_id: id, count: 1 },
        DecisionKind::ChooseReplacementEffect { affected_object: Some(id) },
        DecisionKind::ChooseReplacementEffect { affected_object: None },
        DecisionKind::ApplyOptionalReplacement { affected_object: None, source: id },
        DecisionKind::ApplyOptionalEffect { source: id },
        DecisionKind::AllocateNextDamage { source: id, remaining: 3 },
        DecisionKind::ChooseDamageSource { source: id },
        DecisionKind::ChoosePlayer { source: id },
        DecisionKind::ChoosePlayerAsObjectEnters { object: id, source: id, role: EntryRole::CounterPutter },
        DecisionKind::ChooseAuxiliaryZoneChange { entering: id, source: id, to: Zone::Graveyard },
        DecisionKind::ChooseCopySource { source: id },
        DecisionKind::CommanderToCommandZoneSba { commander: id },
        DecisionKind::Discard { source: Some(id) },
        DecisionKind::Scry { source: Some(id), n: 2 },
        DecisionKind::OrderLibraryCards { source: Some(id), bottom: true },
        DecisionKind::LegendRule { legend_name: "Isamaru, Hound of Konda".to_string() },
        DecisionKind::OrderTriggers { player: 0, tier: TriggerTier::Second },
        DecisionKind::AcceptAsObjectEnters { object: id, source: id, offer: EntryOffer::Pay(Cost::PayLife(2)) },
        DecisionKind::ChooseDelayedTriggerEvent { source: id },
        DecisionKind::ChooseEnteringDefender { object: id },
        DecisionKind::DivideAmong { source: id },
        DecisionKind::ChooseModes { source: id, words: Vec::new() },
        DecisionKind::PayAsItResolves {
            source: id,
            offer: vec![Cost::Sacrifice(mtgsim::types::effects::ObjectFilter::All, 1)],
            any_amount_of: Some(ManaCost::build(&[], 1)),
        },
        DecisionKind::ChooseAmountToPay {
            source: id,
            of: AmountOf::AnyAmountOf(ManaCost::build(&[mtgsim::types::mana::ManaType::Red], 0)),
            values: Box::new(mtgsim::oracle::x_values::XOffer::Set(vec![0, 2])),
        },
        DecisionKind::ChooseColor { source: id },
        DecisionKind::ChooseCreatureType { source: id },
        DecisionKind::ChooseCardType { source: id },
        DecisionKind::ChooseNumber { source: id, values: Box::new(mtgsim::oracle::x_values::XOffer::Interval { at_least: 0, at_most: 10 }) },
    ];
    for kind in &kinds {
        Raised::of(kind, false).check();
        // `as_str` is a match the compiler holds to the variants, and its
        // strings are checked here against each variant's own name.
        let debug = format!("{kind:?}");
        assert_eq!(debug.split([' ', '{', '(']).next(), Some(kind.as_str()), "{debug}");
    }
    // Every variant has a place, by a match the compiler holds to the enum:
    // a new variant does not compile here until it has one, and the places
    // the fixtures above reach must be all of them.
    let place = |kind: &DecisionKind| match kind {
        DecisionKind::PriorityAction => 0,
        DecisionKind::DeclareAttackers => 1,
        DecisionKind::DeclareBlockers => 2,
        DecisionKind::AssignCombatDamage { .. } => 3,
        DecisionKind::AssignTrampleDamage { .. } => 4,
        DecisionKind::ChooseXValue { .. } => 5,
        DecisionKind::ChooseAlternativeCost { .. } => 6,
        DecisionKind::ChooseAdditionalCosts { .. } => 7,
        DecisionKind::SelectRecipients { .. } => 8,
        DecisionKind::GenericManaAllocation { .. } => 9,
        DecisionKind::OrderCostReductions { .. } => 10,
        DecisionKind::ManaAbilityWindow { .. } => 11,
        DecisionKind::ChooseSacrificeForCost { .. } => 12,
        DecisionKind::ChooseReplacementEffect { .. } => 13,
        DecisionKind::OrderTriggers { .. } => 14,
        DecisionKind::ApplyOptionalReplacement { .. } => 15,
        DecisionKind::ApplyOptionalEffect { .. } => 16,
        DecisionKind::AllocateNextDamage { .. } => 17,
        DecisionKind::ChooseDamageSource { .. } => 18,
        DecisionKind::ChooseDelayedTriggerEvent { .. } => 19,
        DecisionKind::ChoosePlayer { .. } => 20,
        DecisionKind::ChoosePlayerAsObjectEnters { .. } => 21,
        DecisionKind::ChooseEnteringDefender { .. } => 22,
        DecisionKind::ChooseAuxiliaryZoneChange { .. } => 23,
        DecisionKind::ChooseCopySource { .. } => 24,
        DecisionKind::CommanderToCommandZoneSba { .. } => 25,
        DecisionKind::Discard { .. } => 26,
        DecisionKind::Scry { .. } => 27,
        DecisionKind::OrderLibraryCards { .. } => 28,
        DecisionKind::LegendRule { .. } => 29,
        DecisionKind::AcceptAsObjectEnters { .. } => 30,
        DecisionKind::DivideAmong { .. } => 31,
        DecisionKind::OrderCostPayments { .. } => 32,
        DecisionKind::OrderSimultaneousEntries { .. } => 33,
        DecisionKind::ChooseModes { .. } => 34,
        DecisionKind::PayAsItResolves { .. } => 35,
        DecisionKind::ChooseAmountToPay { .. } => 36,
        DecisionKind::ChooseColor { .. } => 37,
        DecisionKind::ChooseCreatureType { .. } => 38,
        DecisionKind::ChooseCardType { .. } => 39,
        DecisionKind::ChooseNumber { .. } => 40,
    };
    let reached: BTreeSet<usize> = kinds.iter().map(place).collect();
    assert_eq!(reached, (0..41).collect::<BTreeSet<_>>(), "one fixture per variant, the match's every place");
}
