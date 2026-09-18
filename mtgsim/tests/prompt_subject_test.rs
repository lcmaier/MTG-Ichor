//! Every prompt a game raises says what it is about, and renders.
//!
//! `backlog.md` §2.21: a `ChoiceContext` is supposed to carry enough that a
//! client can say *why* a player is being asked and highlight *what* the
//! question is about, and until this test nothing enforced it. The discipline
//! lived in prose on each variant; the one consumer in the tree, `ui/cli.rs`,
//! is omniscient; and `RandomDecisionProvider` picks by index without asking
//! what a prompt means, so the fuzz harness cannot notice a variant that
//! dropped its source. `ChoiceKind::subject` and `ChoiceKind::describe` are
//! the contract (`codebase-state.md` item 141's payload rule): their
//! exhaustive matches make a new variant decide at compile time, and this
//! file checks the runtime half — whole games at two seats and four, each
//! deck a different sixty-card window of the registry, plus one of each
//! variant built by hand for the prompts no registered card raises.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use mtgsim::cards::registry::CardRegistry;
use mtgsim::events::event::DamageTarget;
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::{GameState, PhaseType, StepType};
use mtgsim::types::effects::{EffectRecipient, SelectionFilter, TargetCount};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::ManaCost;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::DecisionProvider;
use mtgsim::ui::random::RandomDecisionProvider;

/// The variants that may answer `subject()` with `None`, and the rule that
/// makes each one so. Beside the assertion rather than in the engine so that
/// adding a name here is a visible decision in review; `ChoiceKind::subject`'s
/// doc is the same list with the reasons. `Discard` is not here: its `None` is
/// allowed only in the cleanup step, which the recorder checks.
const SUBJECT_LESS: &[(&str, &str)] = &[
    ("PriorityAction", "117.1"),
    ("DeclareAttackers", "508.1a"),
    ("DeclareBlockers", "509.1a"),
    ("LegendRule", "704.5j"),
    // `affected_object: None` — the event is about the choosing player.
    ("ChooseReplacementEffect", "616.1"),
];

/// The rules text the engine targets, read once.
fn baseline() -> &'static str {
    static CR: OnceLock<String> = OnceLock::new();
    CR.get_or_init(|| {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../MTG-Rules/versions/tmnt.txt");
        std::fs::read_to_string(path).expect("the CR baseline is checked in beside the crate")
    })
}

/// Whether `rule` — `"601.2c"`, `"616.1"` — heads a line of the baseline. A
/// top-level rule prints with a trailing period and a lettered one with a
/// space, and neither prefix reaches a neighbor: `"117.1."` is not a prefix
/// of `"117.10."`.
fn cr_has_rule(rule: &str) -> bool {
    let dotted = format!("{rule}.");
    let spaced = format!("{rule} ");
    baseline().lines().any(|line| line.starts_with(&dotted) || line.starts_with(&spaced))
}

/// The variant's name, off `Debug` — `ChoiceKind` is not `PartialEq` (its
/// `SelectRecipients` carries filters), and reading the rendering is the idiom
/// `test_support::RecordingDecisionProvider` documents.
fn variant_name(kind: &ChoiceKind) -> String {
    let debug = format!("{kind:?}");
    debug.split([' ', '{', '(']).next().unwrap_or("").to_string()
}

/// One prompt as the engine raised it: what `subject()` and `describe()`
/// said, and whether the game was in its cleanup step — the one place a
/// `Discard` may have no source (CR 514.1).
struct Raised {
    variant: String,
    subject: Option<String>,
    rule: &'static str,
    text: String,
    at_cleanup: bool,
}

impl Raised {
    fn of(kind: &ChoiceKind, at_cleanup: bool) -> Self {
        let described = kind.describe();
        Raised {
            variant: variant_name(kind),
            subject: kind.subject().map(|id| id.to_string()),
            rule: described.rule,
            text: described.text,
            at_cleanup,
        }
    }

    /// What every prompt must satisfy, whichever way it was built.
    fn check(&self) {
        assert!(!self.text.is_empty(), "{}: describe() rendered nothing", self.variant);
        assert!(
            cr_has_rule(self.rule),
            "{}: describe() cites CR {}, which the baseline does not have",
            self.variant,
            self.rule
        );
        match &self.subject {
            Some(id) => assert!(
                self.text.contains(id.as_str()),
                "{}: the line does not name its subject {id}: {}",
                self.variant,
                self.text
            ),
            None => {
                let allowed = SUBJECT_LESS.iter().any(|(name, _)| *name == self.variant)
                    || (self.variant == "Discard" && self.at_cleanup);
                assert!(
                    allowed,
                    "{} (CR {}) was asked without a subject: {}",
                    self.variant,
                    self.rule,
                    self.text
                );
            }
        }
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
/// subject it did not declare, every line names its subject, and every rule
/// cited is in the baseline.
#[test]
fn every_prompt_a_game_raises_says_what_it_is_about() {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut total = 0;
    let games = [(0xA4_0001u64, 2), (0xA4_0002, 2), (0xA4_0004, 4), (0xC0FF_EE42, 4)];
    for (window, (seed, players)) in games.into_iter().enumerate() {
        for raised in walk(seed, players, 40, window) {
            raised.check();
            *seen.entry(raised.variant).or_default() += 1;
            total += 1;
        }
    }
    // What the walk reached, for `--nocapture` and the record.
    eprintln!("{total} prompts: {seen:?}");
    // The guard: a walk that raised little would pass the checks for free.
    assert!(total >= 1_000, "only {total} prompts across four games: {seen:?}");
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
fn every_variant_decides_its_subject_and_renders() {
    let id = ObjectId::UNASSIGNED;
    let kinds = vec![
        ChoiceKind::PriorityAction,
        ChoiceKind::DeclareAttackers,
        ChoiceKind::DeclareBlockers,
        ChoiceKind::AssignCombatDamage { attacker_id: id },
        ChoiceKind::AssignTrampleDamage { attacker_id: id, defending_target: DamageTarget::Player(1) },
        ChoiceKind::ChooseXValue { spell_id: id, x_count: 2 },
        ChoiceKind::ChooseAlternativeCost { spell_id: id },
        ChoiceKind::ChooseAdditionalCosts { spell_id: id },
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Creature, TargetCount::UpTo(2)),
            spell_id: id,
        },
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Choose(SelectionFilter::Creature, TargetCount::Exactly(1)),
            spell_id: id,
        },
        ChoiceKind::GenericManaAllocation { spell_or_ability_id: id, mana_cost: ManaCost::build(&[], 2) },
        ChoiceKind::OrderCostReductions { spell_id: id },
        ChoiceKind::ManaAbilityWindow { spell_or_ability_id: id, remaining_cost: ManaCost::build(&[], 1) },
        ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id: id, count: 1 },
        ChoiceKind::ChooseReplacementEffect { affected_object: Some(id) },
        ChoiceKind::ChooseReplacementEffect { affected_object: None },
        ChoiceKind::ApplyOptionalReplacement { affected_object: None, source: id },
        ChoiceKind::AllocateNextDamage { source: id, remaining: 3 },
        ChoiceKind::ChooseDamageSource { source: id },
        ChoiceKind::ChooseEnteringController { object: id },
        ChoiceKind::ChooseAuxiliaryZoneChange { entering: id, source: id, to: Zone::Graveyard },
        ChoiceKind::ChooseCopySource { spell_id: id },
        ChoiceKind::CommanderToCommandZoneSba { commander: id },
        ChoiceKind::Discard { source: Some(id) },
        ChoiceKind::Scry { source: Some(id), n: 2 },
        ChoiceKind::ScryOrder { source: Some(id), bottom: true },
        ChoiceKind::LegendRule { legend_name: "Isamaru, Hound of Konda".to_string() },
    ];
    let mut names = BTreeSet::new();
    for kind in &kinds {
        Raised::of(kind, false).check();
        names.insert(variant_name(kind));
    }
    assert_eq!(
        names.len(),
        25,
        "one fixture per variant; a variant was added without one: {names:?}"
    );
}
