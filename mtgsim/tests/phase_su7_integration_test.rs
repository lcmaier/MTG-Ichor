//! SU-7, why an option is not offered (`setup-architecture.md` §7c): each
//! family's one check, shared by the enumeration and the enforcement, and the
//! why's "At this question" section, which words its reasons.

use std::cell::RefCell;
use std::collections::VecDeque;

use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::combat::validation::{validate_attackers, AttackConstraints, CombatError};
use mtgsim::engine::costs::CannotPay;
use mtgsim::engine::put_on_stack::SorceryTiming;
use mtgsim::objects::card_data::{AbilityType, CardDataBuilder};
use mtgsim::oracle::characteristics::get_effective_abilities;
use mtgsim::oracle::legality::{can_attack, can_play_land, candidate_priority_actions, playable_lands, CannotPlayLand};
use mtgsim::oracle::mana_helpers::{
    can_activate, can_begin_to_activate, can_begin_to_cast, can_cast, castable_spells, CannotActivate, CannotCast,
};
use mtgsim::scenario::Scenario;
use mtgsim::state::battlefield::AttackTarget;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_in_hand, put_spell_on_stack};
use mtgsim::types::card_types::CardType;
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption, Rejection};
use mtgsim::types::mana::ManaType;
use mtgsim::ui::decision::{DecisionProvider, PriorityAction};
use mtgsim::ui::display::named;
use mtgsim::ui::why::{why, OpenQuestion, Why, WhyAbout, WhyLine};

fn built(text: &str) -> GameState {
    let scenario = Scenario::parse(text).unwrap_or_else(|refusal| panic!("{refusal}"));
    scenario.build(&CardRegistry::default_registry()).unwrap_or_else(|refusal| panic!("{refusal}")).game.state
}

/// The one object with this name, in any zone.
fn find(game: &GameState, name: &str) -> ObjectId {
    let mut ids: Vec<ObjectId> = game.objects.iter().filter(|(_, o)| o.card_data.name == name).map(|(id, _)| *id).collect();
    assert_eq!(ids.len(), 1, "one {name}");
    ids.remove(0)
}

/// `about`'s why at `player`'s priority question, as the priority loop asks
/// it: the candidates in order, and the answer rejected before, if any.
fn at_priority(game: &GameState, player: PlayerId, about: WhyAbout, rejected: Option<Rejection>) -> Why {
    let options: Vec<ChoiceOption> = candidate_priority_actions(game, player).into_iter().map(ChoiceOption::Action).collect();
    let context = ChoiceContext { kind: ChoiceKind::PriorityAction, rejected };
    why(game, about, Some(&OpenQuestion { player, context: &context, options: &options }))
}

/// The "At this question" section's lines, each as its text and its rule.
fn question_lines(answer: &Why) -> Vec<(String, Option<&'static str>)> {
    let section = &answer.sections[0];
    assert_eq!(section.heading, "At this question");
    section.lines.iter().map(|WhyLine { text, rule, .. }| (text.clone(), *rule)).collect()
}

fn has(answer: &Why, text: &str, rule: Option<&'static str>) -> bool {
    question_lines(answer).contains(&(text.to_string(), rule))
}

const MAIN: &str = "turn 3\nactive 0\nstep precombat main\nlibrary 0: Plains | x8\nlibrary 1: Plains | x8\n";

/// One Forest makes one mana, which pays for Giant Growth and not for
/// Grizzly Bears' {1}{G} (CR 601.2h).
#[test]
fn grizzly_bears_is_never_offered_while_the_mana_falls_short() {
    let game = built(&format!(
        "{MAIN}hand 0: Grizzly Bears\nhand 0: Giant Growth\nbattlefield: Forest | controller 0\nbattlefield: Savannah Lions | controller 0\n"
    ));
    let (bears, growth) = (find(&game, "Grizzly Bears"), find(&game, "Giant Growth"));
    assert_eq!(can_cast(&game, 0, bears).err(), Some(CannotCast::ManaShort));
    assert!(!castable_spells(&game, 0).contains(&bears));

    let answer = at_priority(&game, 0, WhyAbout::Object(bears), None);
    assert_eq!(question_lines(&answer)[0], ("Player 0: You have priority".to_string(), None));
    assert!(has(&answer, "Never offered to Player 0:", None), "{answer:#?}");
    assert!(has(&answer, "To cast it: the mana Player 0 can make now does not cover its cost {1}{G}.", Some("601.2h")));
    let answer = at_priority(&game, 0, WhyAbout::Object(growth), None);
    let cast = format!("Cast {}", named(&game, growth));
    assert!(has(&answer, "Offered to Player 0:", None) && has(&answer, &cast, None), "{answer:#?}");
}

/// The review board: one Everywhere is one mana, so Grizzly Bears' {1}{G} is
/// never offered (`codebase-state.md` item 162, MA-1's exact check). A cast
/// a seat began and could not pay is still reversed, and asked again the why
/// says both tiers: never offered now, since Everywhere is tapped and one
/// {G} floats, and offered and then reversed (CR 732.1).
#[test]
fn on_the_review_board_one_everywhere_does_not_pay_for_the_bears() {
    let text = format!("{MAIN}hand 0: Lightning Bolt\nhand 0: Grizzly Bears\nbattlefield: Everywhere | controller 0\n");
    let game = built(&text);
    let bears = find(&game, "Grizzly Bears");
    assert_eq!(can_cast(&game, 0, bears).err(), Some(CannotCast::ManaShort));
    let answer = at_priority(&game, 0, WhyAbout::Object(bears), None);
    assert!(has(&answer, "To cast it: the mana Player 0 can make now does not cover its cost {1}{G}.", Some("601.2h")));

    let mut reversed = built(&text.replace("Everywhere | controller 0", "Everywhere | controller 0, tapped"));
    reversed.players[0].mana_pool.add(ManaType::Green, 1);
    let bears = find(&reversed, "Grizzly Bears");
    let answer = at_priority(&reversed, 0, WhyAbout::Object(bears), Some(Rejection::Reversed(PriorityAction::CastSpell(bears))));
    assert!(has(&answer, "To cast it: the mana Player 0 can make now does not cover its cost {1}{G}.", Some("601.2h")));
    assert!(has(&answer, "Offered to Player 0, then reversed:", None), "{answer:#?}");
}

/// CR 601.2g's window offers the mana abilities whose costs can be paid, and
/// says of the rest which cost cannot (`mana-architecture.md` §3.13): a
/// tapped Mountain, and a creature the Hierophants' grant cannot tap the turn
/// it arrived (CR 302.6).
// COVERS-PARTIAL: ATOM-118.3-002
#[test]
fn the_window_says_why_each_mana_ability_is_not_offered() {
    let game = built(&format!(
        "{MAIN}hand 0: Grizzly Bears\n\
         battlefield: Mountain | controller 0, tapped\n\
         battlefield: Forest | controller 0\n\
         battlefield: Citanul Hierophants | controller 0, arrived this turn\n"
    ));
    let bears = find(&game, "Grizzly Bears");
    let options: Vec<ChoiceOption> = mtgsim::oracle::mana_supply::ManaAbilityWindowOffer::read(&game, 0)
        .options(&game)
        .into_iter()
        .map(|(source, ability)| ChoiceOption::Action(PriorityAction::ActivateAbility(source, ability)))
        .collect();
    let remaining_cost = mtgsim::types::mana::ManaCost::build(&[ManaType::Green], 1);
    let context = ChoiceContext { kind: ChoiceKind::ManaAbilityWindow { spell_or_ability_id: bears, remaining_cost }, rejected: None };
    let asked = |name: &str| why(&game, WhyAbout::Object(find(&game, name)), Some(&OpenQuestion { player: 0, context: &context, options: &options }));

    let mountain = asked("Mountain");
    let tapped = "To activate “{T}: Add {R}.”: it is already tapped, so it can't be tapped to pay {T}.";
    assert!(has(&mountain, tapped, Some("118.3")), "{mountain:#?}");
    let hierophants = asked("Citanul Hierophants");
    let sick = "To activate “{T}: Add {G}.”: it has not been under Player 0's control since their most recent turn began, \
                so it can't pay {T} or {Q}.";
    assert!(has(&hierophants, sick, Some("302.6")), "{hierophants:#?}");
    let forest = asked("Forest");
    assert!(has(&forest, "Offered to Player 0:", None) && !has(&forest, "Never offered to Player 0:", None), "{forest:#?}");
}

/// CR 117.1a: a creature spell waits for its caster's main phase with the
/// stack empty, a spell with flash does not, and the cast refuses what the
/// enumeration does, with the same reason.
// COVERS: ATOM-117.1a-002
#[test]
fn a_creature_spell_waits_for_sorcery_timing_and_a_flash_spell_does_not() {
    let text = "turn 4\nactive 1\nstep precombat main\nlibrary 0: Plains | x8\nlibrary 1: Plains | x8\n\
                hand 0: Grizzly Bears\nhand 0: Containment Priest\nhand 1: Hill Giant\n\
                battlefield: Plains | controller 0\nbattlefield: Plains | controller 0\nbattlefield: Forest | controller 0\n";
    let mut game = built(text);
    let (bears, priest, giant) = (find(&game, "Grizzly Bears"), find(&game, "Containment Priest"), find(&game, "Hill Giant"));
    let not_yours = CannotCast::Timing(SorceryTiming::NotYourTurn);
    assert_eq!(can_cast(&game, 0, bears).err(), Some(not_yours));
    assert!(can_cast(&game, 0, priest).is_ok(), "flash: any time its caster could cast an instant (CR 702.8a)");
    assert_eq!(can_cast(&game, 0, giant).err(), Some(CannotCast::NotInHand), "another player's hand");

    let answer = at_priority(&game, 0, WhyAbout::Object(bears), None);
    let timing = "To cast it: it is not an instant and has no flash, so it is cast only in its caster's main phase \
                  with the stack empty, and it is not Player 0's turn.";
    assert!(has(&answer, timing, Some("117.1a")), "{answer:#?}");
    let answer = at_priority(&game, 0, WhyAbout::Object(giant), None);
    let not_in_hand = "To cast it: it is not in Player 0's hand, and a spell is cast from its caster's hand.";
    assert!(has(&answer, not_in_hand, Some("601.3")), "{answer:#?}");

    let refused = game.cast_spell(0, bears, &mtgsim::ui::decision::ScriptedDecisionProvider::new());
    assert_eq!(refused, Err(not_yours.to_string()), "the cast asks the same check");

    let mut on_its_turn = built(&text.replace("active 1", "active 0"));
    let bears = find(&on_its_turn, "Grizzly Bears");
    assert!(can_begin_to_cast(&on_its_turn, 0, bears).is_ok());
    put_spell_on_stack(&mut on_its_turn, mtgsim::test_support::lightning_bolt(), 1);
    let stack = CannotCast::Timing(SorceryTiming::StackNotEmpty);
    assert_eq!(can_begin_to_cast(&on_its_turn, 0, bears), Err(stack));
    let refused = on_its_turn.cast_spell(0, bears, &mtgsim::ui::decision::ScriptedDecisionProvider::new());
    assert_eq!(refused, Err(stack.to_string()), "a creature spell with a spell on the stack");
    let upkeep = built(&text.replace("active 1", "active 0").replace("precombat main", "upkeep"));
    let bears = find(&upkeep, "Grizzly Bears");
    assert_eq!(can_begin_to_cast(&upkeep, 0, bears), Err(CannotCast::Timing(SorceryTiming::NotAMainPhase)));

    let blank = put_in_hand(&mut game, CardDataBuilder::new("Blank").card_type(CardType::Instant).build(), 0);
    assert_eq!(can_begin_to_cast(&game, 0, blank), Err(CannotCast::NoSpellAbility), "nothing would resolve");
}

/// What CR 601.2c and 601.2h would refuse: a target with no legal choice, and
/// a mandatory additional cost that can't be paid.
#[test]
fn a_missing_target_and_an_unpayable_additional_cost_are_named() {
    let game = built(&format!(
        "{MAIN}hand 0: Giant Growth\nhand 0: Altar's Reap\nbattlefield: Swamp | controller 0\nbattlefield: Forest | controller 0\n"
    ));
    let (growth, reap) = (find(&game, "Giant Growth"), find(&game, "Altar's Reap"));
    assert_eq!(can_cast(&game, 0, growth).err(), Some(CannotCast::NoLegalTarget));
    let sacrifice = CannotPay::TooFewToSacrifice { matching: 0, needed: 1 };
    assert_eq!(can_cast(&game, 0, reap).err(), Some(CannotCast::AdditionalCost(sacrifice)));
    let answer = at_priority(&game, 0, WhyAbout::Object(growth), None);
    assert!(has(&answer, "To cast it: one of its targets has no legal choice.", Some("601.2c")), "{answer:#?}");
    let answer = at_priority(&game, 0, WhyAbout::Object(reap), None);
    let words = "To cast it: its additional cost can't be paid: Player 0 controls nothing it could sacrifice.";
    assert!(has(&answer, words, Some("701.21a")), "{answer:#?}");
}

/// CR 305.1 and 305.2: one land a turn, from the hand, in its owner's main
/// phase, and `play_land` refuses what `playable_lands` leaves out.
#[test]
fn a_land_is_played_once_a_turn_from_its_owners_hand() {
    let text = format!("{MAIN}player 0: lands played 1\nhand 0: Forest\nhand 0: Grizzly Bears\ngraveyard 0: Mountain\n");
    let mut game = built(&text);
    let (forest, bears, mountain) = (find(&game, "Forest"), find(&game, "Grizzly Bears"), find(&game, "Mountain"));
    let used = CannotPlayLand::NoLandDropLeft { played: 1, allowed: 1 };
    assert_eq!(can_play_land(&game, 0, forest), Err(used));
    assert_eq!(can_play_land(&game, 0, bears), Err(CannotPlayLand::NotALand));
    assert_eq!(can_play_land(&game, 0, mountain), Err(CannotPlayLand::NotInHand));
    assert!(playable_lands(&game, 0).is_empty());
    let answer = at_priority(&game, 0, WhyAbout::Object(forest), None);
    assert!(has(&answer, "To play it: Player 0 has played a land this turn already.", Some("305.2")), "{answer:#?}");
    let refused = game.play_land(0, forest, &mtgsim::test_support::test_ctx());
    assert_eq!(refused, Err(used.to_string()), "play_land asks the same check");

    let theirs = built(&text.replace("active 0", "active 1").replace("player 0: lands played 1\n", ""));
    let forest = find(&theirs, "Forest");
    assert_eq!(can_play_land(&theirs, 0, forest), Err(CannotPlayLand::Timing(SorceryTiming::NotYourTurn)));
    let answer = at_priority(&theirs, 0, WhyAbout::Object(forest), None);
    let words = "To play it: a land is played only in its owner's main phase with the stack empty, and it is not Player 0's turn.";
    assert!(has(&answer, words, Some("305.1")), "{answer:#?}");
}

/// CR 305.9: a land is never cast. Only the enumeration knew it, so a cast
/// naming a land went through as a spell.
#[test]
fn a_land_cannot_be_cast_as_a_spell() {
    let mut game = built(&format!("{MAIN}hand 0: Forest\n"));
    let forest = find(&game, "Forest");
    let refused = game.cast_spell(0, forest, &mtgsim::ui::decision::ScriptedDecisionProvider::new());
    assert_eq!(refused, Err(CannotCast::Land.to_string()));
    assert!(game.stack.is_empty() && game.players[0].hand.contains(&forest));
}

/// CR 602: whose abilities, which ones, when, and what their costs and
/// targets need. Each reason is the one the window would refuse, and
/// activating another player's permanent's ability is refused for its own.
// COVERS: ATOM-602.2-001
#[test]
fn each_ability_says_why_it_is_not_offered() {
    let mut game = built(&format!(
        "{MAIN}battlefield: Everywhere | controller 0\n\
         battlefield: Temple Bell | controller 0, tapped\n\
         battlefield: Merfolk Thaumaturgist | controller 0, arrived this turn\n\
         battlefield: Elvish Warmaster | controller 0\n\
         battlefield: Glorious Anthem | controller 0\n\
         battlefield: Words of Worship | controller 1\n"
    ));
    let first = |name: &str, reason: CannotActivate| {
        let id = find(&game, name);
        let abilities = get_effective_abilities(&game, id);
        let activated = |a: &&mtgsim::objects::card_data::AbilityDef| {
            matches!(a.ability_type, AbilityType::Activated | AbilityType::Mana)
        };
        let ability = abilities.iter().filter(activated).find(|a| can_activate(&game, 0, id, a).is_err()).expect("refused");
        assert_eq!(can_activate(&game, 0, id, ability), Err(reason), "{name}");
        (id, ability.rules_text.words)
    };
    first("Everywhere", CannotActivate::ManaAbility);
    first("Temple Bell", CannotActivate::Cost(CannotPay::AlreadyTapped));
    first("Merfolk Thaumaturgist", CannotActivate::Cost(CannotPay::SummoningSick));
    first("Elvish Warmaster", CannotActivate::ManaShort);
    first("Words of Worship", CannotActivate::NotYours);
    let anthem = find(&game, "Glorious Anthem");
    let abilities = get_effective_abilities(&game, anthem);
    assert_eq!(can_begin_to_activate(&game, 0, &abilities[0]), Err(CannotActivate::NotActivated));

    let bell = WhyAbout::Object(find(&game, "Temple Bell"));
    let words = "To activate “{T}: Each player draws a card.”: it is already tapped, so it can't be tapped to pay {T}.";
    assert!(has(&at_priority(&game, 0, bell, None), words, Some("118.3")));
    let everywhere = at_priority(&game, 0, WhyAbout::Object(find(&game, "Everywhere")), None);
    assert!(question_lines(&everywhere).iter().any(|(_, rule)| *rule == Some("605.3a")), "{everywhere:#?}");
    let theirs = at_priority(&game, 0, WhyAbout::Object(find(&game, "Words of Worship")), None);
    let words = "To activate its abilities: Player 1 controls it, and only its controller activates its abilities.";
    assert!(has(&theirs, words, Some("602.2")), "{theirs:#?}");
    let worship = find(&game, "Words of Worship");
    let refused = game.activate_ability(0, worship, 0, &mtgsim::ui::decision::ScriptedDecisionProvider::new());
    assert_eq!(refused, Err(CannotActivate::NotYours.to_string()), "the activation asks the same check");
}

/// CR 602.5d: equip is activated only as a sorcery, and activating it on the
/// opponent's turn is refused for the reason the window gives.
// COVERS: ATOM-602.5d-001
#[test]
fn a_sorcery_speed_ability_waits_for_its_controllers_main_phase() {
    let text = "turn 4\nactive 1\nstep precombat main\nlibrary 0: Plains | x8\nlibrary 1: Plains | x8\n\
                battlefield: Forest | controller 0\nbattlefield: Grizzly Bears | controller 0\nbattlefield: Bonesplitter | controller 0\n";
    let mut game = built(text);
    let bonesplitter = find(&game, "Bonesplitter");
    let abilities = get_effective_abilities(&game, bonesplitter);
    let equip = abilities.iter().position(|a| a.rules_text.words == "Equip {1}").expect("equip");
    let not_yours = CannotActivate::Timing(SorceryTiming::NotYourTurn);
    assert_eq!(can_activate(&game, 0, bonesplitter, &abilities[equip]), Err(not_yours));
    let answer = at_priority(&game, 0, WhyAbout::Object(bonesplitter), None);
    let words = "To activate “Equip {1}”: it is activated only as a sorcery, in its controller's main phase with the stack \
                 empty, and it is not Player 0's turn.";
    assert!(has(&answer, words, Some("602.5d")), "{answer:#?}");
    let refused = game.activate_ability(0, bonesplitter, equip, &mtgsim::ui::decision::ScriptedDecisionProvider::new());
    assert_eq!(refused, Err(SorceryTiming::NotYourTurn.to_string()), "the activation asks the same check");

    let alone = built(&text.replace("active 1", "active 0").replace("battlefield: Grizzly Bears | controller 0\n", ""));
    let bonesplitter = find(&alone, "Bonesplitter");
    let abilities = get_effective_abilities(&alone, bonesplitter);
    let no_target = can_activate(&alone, 0, bonesplitter, &abilities[equip]);
    assert_eq!(no_target, Err(CannotActivate::NoLegalTarget), "no creature to equip");
}

/// A seat that asks each `about` its why at every pick, then answers with the
/// next scripted pick.
struct Asking {
    about: Vec<WhyAbout>,
    picks: RefCell<VecDeque<Vec<usize>>>,
    whys: RefCell<Vec<Vec<Why>>>,
}

impl Asking {
    fn new(about: Vec<WhyAbout>, picks: Vec<Vec<usize>>) -> Asking {
        Asking { about, picks: RefCell::new(picks.into()), whys: RefCell::new(Vec::new()) }
    }
}

impl DecisionProvider for Asking {
    fn pick_n(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, options: &[ChoiceOption], _: (usize, usize)) -> Vec<usize> {
        let question = OpenQuestion { player, context, options };
        self.whys.borrow_mut().push(self.about.iter().map(|&about| why(game, about, Some(&question))).collect());
        self.picks.borrow_mut().pop_front().unwrap_or_default()
    }
    fn pick_number(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, _: u64, _: u64) -> u64 {
        panic!("only picks here")
    }
    fn allocate(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, _: u64, _: &[ChoiceOption], _: &[u64], _: Option<&[u64]>) -> Vec<u64> {
        panic!("only picks here")
    }
    fn choose_ordering(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, _: &[ChoiceOption]) -> Vec<usize> {
        panic!("only picks here")
    }
}

/// CR 508.1a at the declare-attackers question, and the players: the
/// opponent is among the options, and the attacking player is not. Declaring
/// the tapped creature or the one that arrived this turn is refused.
// COVERS: ATOM-508.1a-001, ATOM-508.1a-002
#[test]
fn the_declare_attackers_question_says_why_each_creature_stays_home() {
    let mut game = built(&format!(
        "{MAIN}battlefield: Grizzly Bears | controller 0\n\
         battlefield: Savannah Lions | controller 0, arrived this turn\n\
         battlefield: Wall of Stone | controller 0\n\
         battlefield: Hill Giant | controller 0, tapped\n"
    ));
    let names = ["Grizzly Bears", "Savannah Lions", "Wall of Stone", "Hill Giant"];
    let [bears, lions, wall, giant] = names.map(|name| find(&game, name));
    let mut about: Vec<WhyAbout> = [bears, lions, wall, giant].map(WhyAbout::Object).to_vec();
    about.extend([WhyAbout::Player(0), WhyAbout::Player(1)]);
    let seat = Asking::new(about, vec![vec![]]);
    game.process_declare_attackers(&seat).expect("no attackers");
    let whys = seat.whys.into_inner().remove(0);

    let n = |id| named(&game, id);
    let attack = format!("{} attacks Player 1", n(bears));
    assert!(has(&whys[0], &attack, None), "{:#?}", whys[0]);
    let lions_words = format!("To attack: {} has not been under its controller's control since their turn began.", n(lions));
    assert!(has(&whys[1], &lions_words, Some("302.6")), "{:#?}", whys[1]);
    assert!(has(&whys[2], &format!("To attack: {} has defender and can't attack.", n(wall)), Some("702.3b")));
    assert!(has(&whys[3], &format!("To attack: {} is tapped.", n(giant)), Some("508.1a")));
    assert!(has(&whys[4], "To be attacked: a creature attacks one of its controller's opponents.", Some("506.2")));
    assert!(has(&whys[5], "Offered to Player 0:", None) && has(&whys[5], &attack, None));

    let declare = |id| validate_attackers(&game, 0, &[(id, AttackTarget::Player(1))], &AttackConstraints::none());
    assert_eq!(declare(lions), Err(CombatError::CreatureHasSummoningSickness(lions)), "the declaration asks the same check");
    assert_eq!(can_attack(&game, 0, lions), declare(lions));
    assert_eq!(declare(giant), Err(CombatError::CreatureIsTapped(giant)));
}

/// CR 509.1a and 702.9b at the declare-blockers question: Wall of Stone can
/// block Hill Giant and not Serra Angel, and a tapped creature blocks
/// nothing, said once.
#[test]
fn the_declare_blockers_question_says_who_can_block_whom() {
    let mut game = built(
        "turn 4\nactive 1\nstep declare attackers\nlibrary 0: Plains | x8\nlibrary 1: Plains | x8\n\
         player 1 this turn: attackers declared 1\n\
         battlefield: Wall of Stone | controller 0\n\
         battlefield: Grizzly Bears | controller 0, tapped\n\
         battlefield: Hill Giant | controller 1, tapped, attacking player 0\n\
         battlefield: Serra Angel | controller 1, attacking player 0\n",
    );
    let [wall, bears, angel] = ["Wall of Stone", "Grizzly Bears", "Serra Angel"].map(|name| find(&game, name));
    let seat = Asking::new([wall, bears, angel].map(WhyAbout::Object).to_vec(), vec![vec![]]);
    game.process_declare_blockers(&seat).expect("no blocks");
    let whys = seat.whys.into_inner().remove(0);

    let n = |id| named(&game, id);
    let giant = find(&game, "Hill Giant");
    assert!(has(&whys[0], &format!("{} blocks {}", n(wall), n(giant)), None), "{:#?}", whys[0]);
    let flying = format!("{} has flying, and {} has neither flying nor reach.", n(angel), n(wall));
    assert!(has(&whys[0], &format!("To block {}: {flying}", n(angel)), Some("702.9b")), "{:#?}", whys[0]);
    let tapped = question_lines(&whys[1]);
    assert!(tapped.contains(&(format!("To block: {} is tapped.", n(bears)), Some("509.1a"))), "{tapped:#?}");
    assert_eq!(tapped.iter().filter(|(text, _)| text.starts_with("To block")).count(), 1, "said once");
    assert!(has(&whys[2], &format!("To be blocked by {}: {flying}", n(wall)), Some("702.9b")), "{:#?}", whys[2]);
}

/// CR 732.1's tier: an answer the engine took and reversed is offered again,
/// and the why says it was reversed, and why.
#[test]
fn an_answer_reversed_at_this_question_is_said_on_its_own_tier() {
    let game = built(&format!("{MAIN}hand 0: Lightning Bolt\nbattlefield: Everywhere | controller 0\n"));
    let bolt = find(&game, "Lightning Bolt");
    let rejected = Rejection::Reversed(PriorityAction::CastSpell(bolt));
    let answer = at_priority(&game, 0, WhyAbout::Object(bolt), Some(rejected));
    assert!(has(&answer, "Offered to Player 0, then reversed:", None), "{answer:#?}");
    let words = format!("Cast {} could not be completed, so it was reversed and its payments canceled", named(&game, bolt));
    assert!(has(&answer, &words, Some("732.1")), "{answer:#?}");

    let mut game = built(
        "turn 4\nactive 1\nstep declare attackers\nlibrary 0: Plains | x8\nlibrary 1: Plains | x8\n\
         player 1 this turn: attackers declared 1\n\
         battlefield: Wall of Stone | controller 0\n\
         battlefield: Grizzly Bears [one] | controller 1, tapped, attacking player 0\n\
         battlefield: Grizzly Bears [two] | controller 1, tapped, attacking player 0\n",
    );
    let wall = find(&game, "Wall of Stone");
    let seat = Asking::new(vec![WhyAbout::Object(wall)], vec![vec![0, 1], vec![]]);
    game.process_declare_blockers(&seat).expect("asked again, then no blocks");
    let reasked = &seat.whys.borrow()[1][0];
    assert!(has(reasked, "Offered to Player 0, then reversed:", None), "{reasked:#?}");
    let words = format!("Those blocks are illegal: {} can block only one attacker", named(&game, wall));
    assert!(has(reasked, &words, Some("509.1a")), "{reasked:#?}");
}

/// A player is something a why is about: at a question whose options name
/// players, whether this one is among them.
#[test]
fn a_player_is_among_a_target_questions_options_or_not() {
    let game = built(&format!("{MAIN}hand 0: Lightning Bolt\nbattlefield: Forest | controller 1\n"));
    let bolt = find(&game, "Lightning Bolt");
    let context = ChoiceContext::new(ChoiceKind::SelectRecipients {
        recipient: mtgsim::types::effects::EffectRecipient::Target(
            mtgsim::types::effects::SelectionFilter::Any,
            mtgsim::types::effects::TargetCount::Exactly(1),
        ),
        spell_id: bolt,
    });
    let options = [ChoiceOption::Player(0), ChoiceOption::Player(1)];
    let question = OpenQuestion { player: 0, context: &context, options: &options };
    let answer = why(&game, WhyAbout::Player(1), Some(&question));
    assert_eq!(answer.title, "Player 1");
    assert!(has(&answer, "Offered to Player 0:", None) && has(&answer, "Player 1", None), "{answer:#?}");
    let forest = why(&game, WhyAbout::Object(find(&game, "Forest")), Some(&question));
    assert!(has(&forest, "Not among the options Player 0 is offered here.", None), "{forest:#?}");
    let at_priority = at_priority(&game, 0, WhyAbout::Player(1), None);
    assert!(has(&at_priority, "Not among the options Player 0 is offered here.", None), "{at_priority:#?}");
    assert_eq!(why(&game, WhyAbout::Player(1), None).sections[0].lines[0].text, "No question is open.");
}
