//! Every prompt a game raises says which object it is about.
//!
//! `backlog.md` §2.21: a `ChoiceContext` is supposed to carry enough that a
//! client can highlight *what* the question is about, and until this test
//! nothing enforced it. The discipline lived in prose on each variant; the one
//! consumer in the tree, `ui/cli.rs`, is omniscient; and
//! `RandomDecisionProvider` picks by index without asking what a prompt means,
//! so the fuzz harness cannot notice a variant that dropped its source.
//! `ChoiceKind::subject` is the contract (`codebase-state.md` item 141's
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
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption, EntryOffer, EntryRole};
use mtgsim::oracle::x_values::LegalXValues;
use mtgsim::ui::decision::DecisionProvider;
use mtgsim::ui::random::RandomDecisionProvider;

/// The variants that may answer `subject()` with `None`, and the rule that
/// makes each one so. Beside the assertion rather than in the engine so that
/// adding a name here is a visible decision in review; `ChoiceKind::subject`'s
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
    fn of(kind: &ChoiceKind, at_cleanup: bool) -> Self {
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

    fn record(&self, game: &GameState, ctx: &ChoiceContext) {
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
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        self.record(game, context);
        self.inner.pick_n(game, player, context, options, bounds)
    }

    fn pick_number(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
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
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
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
        context: &ChoiceContext,
        items: &[ChoiceOption],
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
    let games = [(0xA4_0001u64, 2), (0xA4_0002, 2), (0xA4_0004, 4), (0xA4_0005, 4)];
    // The decks are windows of the sorted registry, so a card registered
    // ahead of a window changes it; fifty turns raise every kind below on
    // the registry as TR-3b left it, where forty raised no blockers. TR-5c's
    // Loyal Sentry moved the last window's game to one that blocked nothing,
    // so its seed moved too.
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
        ChoiceKind::PriorityAction,
        ChoiceKind::DeclareAttackers,
        ChoiceKind::DeclareBlockers,
        ChoiceKind::AssignCombatDamage { attacker_id: id },
        ChoiceKind::AssignTrampleDamage { attacker_id: id, defending_target: ObjectOrPlayerId::Player(1) },
        ChoiceKind::ChooseXValue { spell_id: id, values: Box::new(LegalXValues::zero_through(2)) },
        ChoiceKind::ChooseAlternativeCost { spell_id: id },
        ChoiceKind::ChooseAdditionalCosts { spell_id: id },
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::creature(), TargetCount::UpTo(2)),
            spell_id: id,
        },
        ChoiceKind::GenericManaAllocation { spell_or_ability_id: id, mana_cost: ManaCost::build(&[], 2) },
        ChoiceKind::OrderCostReductions { spell_id: id },
        ChoiceKind::ManaAbilityWindow { spell_or_ability_id: id, remaining_cost: ManaCost::build(&[], 1) },
        ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id: id, count: 1 },
        ChoiceKind::ChooseReplacementEffect { affected_object: Some(id) },
        ChoiceKind::ChooseReplacementEffect { affected_object: None },
        ChoiceKind::ApplyOptionalReplacement { affected_object: None, source: id },
        ChoiceKind::ApplyOptionalEffect { source: id },
        ChoiceKind::AllocateNextDamage { source: id, remaining: 3 },
        ChoiceKind::ChooseDamageSource { source: id },
        ChoiceKind::ChooseOpponent { source: id },
        ChoiceKind::ChoosePlayerAsObjectEnters { object: id, source: id, role: EntryRole::CounterPutter },
        ChoiceKind::ChooseAuxiliaryZoneChange { entering: id, source: id, to: Zone::Graveyard },
        ChoiceKind::ChooseCopySource { source: id },
        ChoiceKind::CommanderToCommandZoneSba { commander: id },
        ChoiceKind::Discard { source: Some(id) },
        ChoiceKind::Scry { source: Some(id), n: 2 },
        ChoiceKind::ScryOrder { source: Some(id), bottom: true },
        ChoiceKind::LegendRule { legend_name: "Isamaru, Hound of Konda".to_string() },
        ChoiceKind::OrderTriggers { player: 0, tier: TriggerTier::Second },
        ChoiceKind::AcceptAsObjectEnters { object: id, source: id, offer: EntryOffer::Pay(Cost::PayLife(2)) },
        ChoiceKind::ChooseDelayedTriggerEvent { source: id },
        ChoiceKind::ChooseEnteringDefender { object: id },
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
    let place = |kind: &ChoiceKind| match kind {
        ChoiceKind::PriorityAction => 0,
        ChoiceKind::DeclareAttackers => 1,
        ChoiceKind::DeclareBlockers => 2,
        ChoiceKind::AssignCombatDamage { .. } => 3,
        ChoiceKind::AssignTrampleDamage { .. } => 4,
        ChoiceKind::ChooseXValue { .. } => 5,
        ChoiceKind::ChooseAlternativeCost { .. } => 6,
        ChoiceKind::ChooseAdditionalCosts { .. } => 7,
        ChoiceKind::SelectRecipients { .. } => 8,
        ChoiceKind::GenericManaAllocation { .. } => 9,
        ChoiceKind::OrderCostReductions { .. } => 10,
        ChoiceKind::ManaAbilityWindow { .. } => 11,
        ChoiceKind::ChooseSacrificeForCost { .. } => 12,
        ChoiceKind::ChooseReplacementEffect { .. } => 13,
        ChoiceKind::OrderTriggers { .. } => 14,
        ChoiceKind::ApplyOptionalReplacement { .. } => 15,
        ChoiceKind::ApplyOptionalEffect { .. } => 16,
        ChoiceKind::AllocateNextDamage { .. } => 17,
        ChoiceKind::ChooseDamageSource { .. } => 18,
        ChoiceKind::ChooseDelayedTriggerEvent { .. } => 19,
        ChoiceKind::ChooseOpponent { .. } => 20,
        ChoiceKind::ChoosePlayerAsObjectEnters { .. } => 21,
        ChoiceKind::ChooseEnteringDefender { .. } => 22,
        ChoiceKind::ChooseAuxiliaryZoneChange { .. } => 23,
        ChoiceKind::ChooseCopySource { .. } => 24,
        ChoiceKind::CommanderToCommandZoneSba { .. } => 25,
        ChoiceKind::Discard { .. } => 26,
        ChoiceKind::Scry { .. } => 27,
        ChoiceKind::ScryOrder { .. } => 28,
        ChoiceKind::LegendRule { .. } => 29,
        ChoiceKind::AcceptAsObjectEnters { .. } => 30,
    };
    let reached: BTreeSet<usize> = kinds.iter().map(place).collect();
    assert_eq!(reached, (0..31).collect::<BTreeSet<_>>(), "one fixture per variant, the match's every place");
}
