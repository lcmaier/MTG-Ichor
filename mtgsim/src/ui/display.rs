// Text formatting helpers for CLI output and logging.
//
// All functions are pure formatters over &GameState — no mutations.
// Lives in ui/ because these are presentation helpers, not game-state queries.

use std::collections::HashSet;
use std::sync::Arc;

use crate::engine::combat::validation::CombatError;
use crate::engine::costs::CannotPay;
use crate::engine::put_on_stack::SorceryTiming;
use crate::engine::layers::compute_characteristics;
use crate::engine::layers::copy::{CopiableValues, copiable_values, copiable_values_on_battlefield};
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::engine::targeting::TargetRef;
use crate::events::event::{DamageTarget, GameEvent, NamesAsAnnounced};
use crate::objects::card_data::{AbilityDef, AbilityText, AbilityType, CardData, paragraphs};
use crate::oracle::characteristics::{
    controller_or_owner, get_effective_abilities, get_effective_power, get_effective_toughness, is_creature,
};
use crate::oracle::legality::CannotPlayLand;
use crate::oracle::mana_helpers::{CannotActivate, CannotCast};
use crate::state::battlefield::AttackTarget;
use crate::state::game_state::{AbilityIdentity, GameState, PhaseType, StepType};
use crate::types::card_types::{CardType, CardTypes, Subtype, Subtypes, Supertype};
use crate::types::colors::Color;
use crate::types::costs::{AdditionalCost, AlternativeCost, Cost};
use crate::types::ids::{AbilityId, DelayedTriggerId, IdMap, ObjectId, PlayerId};
use crate::types::keywords::KeywordFlag;
use crate::types::triggers::DelayedDuration;
use crate::types::mana::ManaSymbol;
use crate::ui::choice_types::{ChoiceKind, ChoiceOption, Rejection};
use crate::ui::decision::PriorityAction;

/// The name the object has now, through the layers: a Clone copying Grizzly
/// Bears is Grizzly Bears (CR 707.2).
pub fn card_name(game: &GameState, id: ObjectId) -> String {
    compute_characteristics(game, id)
        .map(|chars| chars.name.clone())
        .unwrap_or_else(|| "<unknown>".to_string())
}

/// The name printed on the card, for a record an observer writes. A read
/// through the layers counts a walk and fills the memo, which the trace sink
/// and the dispatch audit may not do (`the_sink_changes_nothing_the_game_does`,
/// `an_audited_game_counts_and_traces_what_an_unaudited_one_does`).
pub fn printed_name(game: &GameState, id: ObjectId) -> String {
    // AS PRINTED: the card's own name, for a record no rule reads.
    game.objects.get(&id)
        .map(|obj| obj.card_data.name.clone())
        .unwrap_or_else(|| "<unknown>".to_string())
}

/// A battlefield permanent: its name, power and toughness, keywords and status
/// on the first line, and the text of each of its other abilities on a line of
/// its own. "Elvish Archers 2/1 [first strike] (tapped)", or "Forest" then
/// "{T}: Add {G}.".
pub fn format_permanent(game: &GameState, id: ObjectId) -> String {
    let name = card_name(game, id);
    let entry = match game.battlefield.get(&id) {
        Some(e) => e,
        None => return name,
    };

    let mut parts = vec![name];

    // P/T for creatures
    if is_creature(game, id) {
        let p = get_effective_power(game, id).unwrap_or(0);
        let t = get_effective_toughness(game, id).unwrap_or(0);
        let dmg = entry.damage_marked;
        if dmg > 0 {
            parts.push(format!("{}/{} ({}dmg)", p, t, dmg));
        } else {
            parts.push(format!("{}/{}", p, t));
        }
    }

    let keywords = collect_keywords(game, id);
    if !keywords.is_empty() {
        parts.push(format!("[{}]", keywords.join(", ")));
    }

    // Status flags
    let mut flags = Vec::new();
    if entry.tapped {
        flags.push("tapped");
    }
    if crate::oracle::characteristics::has_summoning_sickness(game, id) {
        flags.push("sick");
    }
    if entry.attacking.is_some() {
        flags.push("attacking");
    }
    if entry.blocking.is_some() {
        flags.push("blocking");
    }
    if !flags.is_empty() {
        parts.push(format!("({})", flags.join(", ")));
    }

    let status = parts.join(" ");
    let mut lines = vec![status.as_str()];
    lines.extend(ability_texts(game, id));
    lines.join("\n")
}

/// A permanent's keywords in `KeywordFlag`'s order, since the effective set is
/// a hash set.
fn collect_keywords(game: &GameState, id: ObjectId) -> Vec<&'static str> {
    let Some(chars) = compute_characteristics(game, id) else { return Vec::new() };
    let mut flags: Vec<KeywordFlag> = chars.keyword_flags.iter().copied().collect();
    flags.sort();
    flags.into_iter().map(keyword_name).collect()
}

/// A keyword as it prints, one arm per flag and no wildcard: a new flag does
/// not compile until this says what it prints as.
pub fn keyword_name(flag: KeywordFlag) -> &'static str {
    match flag {
        KeywordFlag::Deathtouch => "deathtouch",
        KeywordFlag::Defender => "defender",
        KeywordFlag::DoubleStrike => "double strike",
        KeywordFlag::FirstStrike => "first strike",
        KeywordFlag::Flash => "flash",
        KeywordFlag::Flying => "flying",
        KeywordFlag::Haste => "haste",
        KeywordFlag::Hexproof => "hexproof",
        KeywordFlag::Indestructible => "indestructible",
        KeywordFlag::Intimidate => "intimidate",
        KeywordFlag::Lifelink => "lifelink",
        KeywordFlag::Menace => "menace",
        KeywordFlag::Reach => "reach",
        KeywordFlag::Shroud => "shroud",
        KeywordFlag::Trample => "trample",
        KeywordFlag::Vigilance => "vigilance",
    }
}

/// The text of each ability a permanent has that is not a keyword flag, in its
/// effective list's order: the abilities CR 305.7 or a Layer 6 effect left it,
/// never the card's printed text, which can describe abilities the object does
/// not have (a copy of a vanilla creature, a creature under Humility). A
/// printed ability the engine builds as several shows once, as printed: its
/// parts share a paragraph and, when granted, the grant.
fn ability_texts(game: &GameState, id: ObjectId) -> Vec<&'static str> {
    let mut shown: Vec<(AbilityText, Option<u64>)> = Vec::new();
    for ability in get_effective_abilities(game, id).iter() {
        match ability.ability_type {
            AbilityType::Mana | AbilityType::Activated | AbilityType::Triggered | AbilityType::Static => {}
            // An instant's or sorcery's (CR 113.3a), which no permanent is.
            AbilityType::Spell => continue,
        }
        let part = (ability.rules_text, ability.id.granting_row());
        if ability.rules_text.paragraph().is_some() && shown.contains(&part) {
            continue;
        }
        shown.push(part);
    }
    shown.into_iter().map(|(text, _)| text.words).collect()
}

/// The card `id` as printed, before any effect touched it, one entry per
/// face: its name and mana cost, type line, rules text and numbers, a line
/// each. Beside what the object is now, it shows what an effect changed.
pub fn printed_faces(game: &GameState, id: ObjectId) -> Vec<String> {
    // AS PRINTED: the card itself, for a display no rule reads.
    game.objects.get(&id).map(|obj| vec![printed_face(&obj.card_data)]).unwrap_or_default()
}

fn printed_face(card: &CardData) -> String {
    let mut lines = vec![match &card.mana_cost {
        Some(cost) => format!("{} {cost}", card.name),
        None => card.name.clone(),
    }];
    lines.push(type_line(&card.supertypes, &card.types, &card.subtypes));
    lines.extend(paragraphs(&card.rules_text).map(str::to_string));
    match (card.power.zip(card.toughness), card.loyalty, card.defense) {
        (Some((power, toughness)), _, _) => lines.push(format!("{power}/{toughness}")),
        (None, Some(loyalty), _) => lines.push(format!("Loyalty {loyalty}")),
        (None, None, Some(defense)) => lines.push(format!("Defense {defense}")),
        (None, None, None) => {}
    }
    lines.join("\n")
}

/// `Legendary Creature — Elf Warrior`: a type line in printed order.
pub fn type_line(supertypes: &HashSet<Supertype>, types: &CardTypes, subtypes: &Subtypes) -> String {
    let front: Vec<String> = in_printed_order(supertypes, types).iter().map(|word| word.text()).collect();
    let subtypes: Vec<String> = subtypes.iter().map(Subtype::word).collect();
    if subtypes.is_empty() { front.join(" ") } else { format!("{} — {}", front.join(" "), subtypes.join(" ")) }
}

/// A type line word by word against the object's copiable values (CR 707.2),
/// which are what it prints unless it is a copy: [`type_line_now`].
#[derive(Debug, Default, PartialEq)]
pub struct TypeLine {
    /// The supertypes, then the card types.
    pub front: Vec<TypeWord>,
    /// The words after the long dash (CR 205.3b).
    pub subtypes: Vec<TypeWord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeWord {
    pub text: String,
    pub status: TypeWordStatus,
}

/// Where a word of [`TypeLine`] stands against the copiable values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeWordStatus {
    Kept,
    /// The copiable values have it and the object does not.
    Lost,
    /// An effect gave it.
    Gained,
}

impl std::fmt::Display for TypeLine {
    /// The line the object has now: the words it lost are left out.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let has = |words: &[TypeWord]| -> Vec<String> {
            words.iter().filter(|w| w.status != TypeWordStatus::Lost).map(|w| w.text.clone()).collect()
        };
        let subtypes = has(&self.subtypes);
        write!(f, "{}", has(&self.front).join(" "))?;
        if !subtypes.is_empty() {
            write!(f, " — {}", subtypes.join(" "))?;
        }
        Ok(())
    }
}

/// The type line of `id` as it is now, as Arena shows one. A card type or
/// supertype stands in Oracle's order, printed or given; a subtype an effect
/// gave goes after the ones the copiable values have, in the order given; a
/// word the object lost stays where it stood, marked lost. A word given and
/// taken away is not there.
pub fn type_line_now(game: &GameState, id: ObjectId) -> TypeLine {
    match (copiable_values(game, id), compute_characteristics(game, id)) {
        (Some(base), Some(now)) => type_line_against(&base, &now),
        _ => TypeLine::default(),
    }
}

/// [`type_line_now`] for a reader asking about many objects, a board's
/// worth: the permanents' copiable values come from one pass of the board.
pub struct TypeLines(IdMap<ObjectId, CopiableValues>);

impl TypeLines {
    pub fn new(game: &GameState) -> TypeLines {
        TypeLines(copiable_values_on_battlefield(game))
    }

    /// `id`'s type line, `now` being its characteristics.
    pub fn of(&self, game: &GameState, id: ObjectId, now: &EffectiveCharacteristics) -> TypeLine {
        match self.0.get(&id) {
            Some(base) => type_line_against(base, now),
            None => copiable_values(game, id).map(|base| type_line_against(&base, now)).unwrap_or_default(),
        }
    }
}

fn type_line_against(base: &CopiableValues, now: &EffectiveCharacteristics) -> TypeLine {
    let base_front = in_printed_order(&base.supertypes, &base.types);
    let now_front = in_printed_order(&now.supertypes, &now.types);
    let mut words: Vec<FrontWord> = base_front.iter().chain(&now_front).copied().collect();
    words.sort_by_key(FrontWord::rank);
    words.dedup();
    let front = words
        .iter()
        .filter_map(|word| {
            standing(base_front.contains(word), now_front.contains(word)).map(|status| TypeWord { text: word.text(), status })
        })
        .collect();

    let mut subtypes: Vec<TypeWord> = base
        .subtypes
        .iter()
        .map(|subtype| TypeWord {
            text: subtype.word(),
            status: if now.subtypes.contains(subtype) { TypeWordStatus::Kept } else { TypeWordStatus::Lost },
        })
        .collect();
    subtypes.extend(
        now.subtypes
            .iter()
            .filter(|subtype| !base.subtypes.contains(subtype))
            .map(|subtype| TypeWord { text: subtype.word(), status: TypeWordStatus::Gained }),
    );
    if let Some(status) = standing(base.subtypes.has_every_creature_type(), now.subtypes.has_every_creature_type()) {
        subtypes.push(TypeWord { text: "(every creature type)".to_string(), status });
    }
    TypeLine { front, subtypes }
}

/// A word's status from whether the copiable values have it and the object
/// does: `None` for neither.
fn standing(in_base: bool, in_now: bool) -> Option<TypeWordStatus> {
    match (in_base, in_now) {
        (true, true) => Some(TypeWordStatus::Kept),
        (true, false) => Some(TypeWordStatus::Lost),
        (false, true) => Some(TypeWordStatus::Gained),
        (false, false) => None,
    }
}

/// A word in front of a type line's dash.
#[derive(Clone, Copy, PartialEq)]
enum FrontWord {
    Supertype(Supertype),
    CardType(CardType),
}

impl FrontWord {
    fn text(&self) -> String {
        match self {
            FrontWord::Supertype(supertype) => format!("{supertype:?}"),
            FrontWord::CardType(card_type) => format!("{card_type:?}"),
        }
    }

    /// Its place on a type line: supertypes before card types (CR 205.4a),
    /// each in Oracle's order, which no card outside the Un-sets and playtest
    /// cards breaks for any two it prints (a Scryfall census of every pair,
    /// 2026-10-02); the CR gives none. Each word has its own place, so a
    /// sort leaves one word where two lines had it.
    fn rank(&self) -> u8 {
        match self {
            FrontWord::Supertype(supertype) => match supertype {
                Supertype::Basic => 0,
                Supertype::Legendary => 1,
                Supertype::Ongoing => 2,
                Supertype::Snow => 3,
                Supertype::World => 4,
            },
            FrontWord::CardType(card_type) => match card_type {
                CardType::Kindred => 5,
                CardType::Enchantment => 6,
                CardType::Artifact => 7,
                CardType::Land => 8,
                CardType::Creature => 9,
                CardType::Planeswalker => 10,
                CardType::Battle => 11,
                CardType::Instant => 12,
                CardType::Sorcery => 13,
                // Each the one card type on its cards.
                CardType::Conspiracy => 14,
                CardType::Dungeon => 15,
                CardType::Phenomenon => 16,
                CardType::Plane => 17,
                CardType::Scheme => 18,
                CardType::Vanguard => 19,
            },
        }
    }
}

fn in_printed_order(supertypes: &HashSet<Supertype>, types: &CardTypes) -> Vec<FrontWord> {
    let mut words: Vec<FrontWord> = supertypes
        .iter()
        .map(|supertype| FrontWord::Supertype(*supertype))
        .chain(types.iter().map(|card_type| FrontWord::CardType(*card_type)))
        .collect();
    words.sort_by_key(FrontWord::rank);
    words
}

/// "Grizzly Bears (#12)", or "Grizzly Bears (Clone, #12)" for a copy: the
/// object under the name it has now, in the shape `format_event`'s lines use,
/// so a name on the board and one in the log read the same.
pub fn named(game: &GameState, id: ObjectId) -> String {
    match compute_characteristics(game, id) {
        Some(chars) => object_label(game, id, &chars.name),
        None => format!("{id} (gone)"),
    }
}

pub fn player_name(player: PlayerId) -> String {
    format!("Player {player}")
}

pub fn attack_target_name(game: &GameState, target: &AttackTarget) -> String {
    match target {
        AttackTarget::Player(player) => player_name(*player),
        AttackTarget::Planeswalker(id) | AttackTarget::Battle(id) => named(game, *id),
    }
}

fn damage_target_name(game: &GameState, target: &DamageTarget) -> String {
    match target {
        DamageTarget::Player(player) => player_name(*player),
        DamageTarget::Object(id) => named(game, *id),
    }
}

// ---------------------------------------------------------------------------
// Prompts
// ---------------------------------------------------------------------------

/// The question a prompt asks, one arm per kind and no wildcard, so a new kind
/// is asked a question at birth. Both clients print it; the CLI adds how to
/// type an answer.
pub fn question(game: &GameState, kind: &ChoiceKind) -> String {
    let n = |id: &ObjectId| named(game, *id);
    match kind {
        ChoiceKind::PriorityAction => "You have priority".to_string(),
        ChoiceKind::DeclareAttackers => "Declare attackers".to_string(),
        ChoiceKind::DeclareBlockers => "Declare blockers".to_string(),
        ChoiceKind::AssignCombatDamage { attacker_id } => format!("Assign {}'s combat damage", n(attacker_id)),
        ChoiceKind::AssignTrampleDamage { attacker_id, defending_target } => format!(
            "Assign {}'s trample damage; what is left goes to {}",
            n(attacker_id),
            damage_target_name(game, defending_target)
        ),
        ChoiceKind::ChooseXValue { spell_id, .. } => format!("Choose X for {}", n(spell_id)),
        ChoiceKind::ChooseAlternativeCost { spell_id } => format!("Choose how to pay for {}", n(spell_id)),
        ChoiceKind::ChooseAdditionalCosts { spell_id } => format!("Choose additional costs for {}", n(spell_id)),
        // "Choose" is no targeting (CR 115.10): an edict's pick, an Aura's
        // host as it enters (CR 303.4f).
        ChoiceKind::SelectRecipients { recipient: crate::types::effects::EffectRecipient::Choose(..), spell_id } => {
            format!("Choose for {}", n(spell_id))
        }
        ChoiceKind::SelectRecipients { spell_id, .. } => format!("Choose targets for {}", n(spell_id)),
        ChoiceKind::GenericManaAllocation { spell_or_ability_id, mana_cost } => {
            format!("Split the generic part of {mana_cost} for {}", n(spell_or_ability_id))
        }
        ChoiceKind::OrderCostReductions { spell_id } => {
            format!("Order the cost reductions for {}; the first applies first", n(spell_id))
        }
        ChoiceKind::ManaAbilityWindow { spell_or_ability_id, remaining_cost } => {
            format!("Pay {remaining_cost} more for {}: activate a mana ability, or stop", n(spell_or_ability_id))
        }
        ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id, count } => {
            format!("Sacrifice {count} for {}", n(spell_or_ability_id))
        }
        ChoiceKind::ChooseReplacementEffect { affected_object } => match affected_object {
            Some(id) => format!("Choose the replacement effect that applies to {}", n(id)),
            None => "Choose the replacement effect that applies to you".to_string(),
        },
        ChoiceKind::OrderTriggers { .. } => {
            "Order your triggered abilities; the first goes on the stack first and resolves last".to_string()
        }
        ChoiceKind::ApplyOptionalReplacement { source, .. } => format!("Apply {}'s replacement effect?", n(source)),
        ChoiceKind::ApplyOptionalEffect { source } => format!("{}: you may", n(source)),
        ChoiceKind::AllocateNextDamage { source, remaining } => {
            format!("Choose the damage {} prevents ({remaining} left)", n(source))
        }
        ChoiceKind::ChooseDamageSource { source } => format!("Choose a source of damage for {}", n(source)),
        ChoiceKind::ChooseDelayedTriggerEvent { source } => {
            format!("Choose which event triggers {}'s delayed ability", n(source))
        }
        ChoiceKind::ChooseEnteringController { object } => {
            format!("Choose the opponent who controls {} as it enters", n(object))
        }
        ChoiceKind::ChooseAuxiliaryZoneChange { entering, source, to } => {
            format!("Choose what goes to the {to:?} as {} changes how {} enters", n(source), n(entering))
        }
        ChoiceKind::ChooseCopySource { source } => format!("Choose what {} copies", n(source)),
        ChoiceKind::CommanderToCommandZoneSba { commander } => format!("Put {} into the command zone?", n(commander)),
        ChoiceKind::Discard { source } => match source {
            Some(id) => format!("Discard for {}", n(id)),
            None => "Discard down to your maximum hand size".to_string(),
        },
        ChoiceKind::Scry { n: count, .. } => format!("Scry {count}: choose the cards that go on the bottom"),
        ChoiceKind::ScryOrder { bottom, .. } => {
            format!("Order the cards going on the {}, top-most first", if *bottom { "bottom" } else { "top" })
        }
        ChoiceKind::LegendRule { legend_name } => format!("Legend rule: choose the {legend_name} to keep"),
    }
}

/// An option as a client labels it, one arm per kind of option and no
/// wildcard. An activation is its object and the ability's own text:
/// "Everywhere (#20) · {T}: Add {W}.".
pub fn option_label(game: &GameState, option: &ChoiceOption) -> String {
    let n = |id: &ObjectId| named(game, *id);
    match option {
        ChoiceOption::Object(id) => n(id),
        ChoiceOption::Player(player) => player_name(*player),
        ChoiceOption::Action(PriorityAction::Pass) => "Pass".to_string(),
        ChoiceOption::Action(PriorityAction::CastSpell(id)) => format!("Cast {}", n(id)),
        ChoiceOption::Action(PriorityAction::PlayLand(id)) => format!("Play {}", n(id)),
        ChoiceOption::Action(PriorityAction::ActivateAbility(id, ability)) => {
            format!("{} · {}", n(id), ability_text(game, *id, *ability))
        }
        ChoiceOption::AttackerTarget(attacker, target) => {
            format!("{} attacks {}", n(attacker), attack_target_name(game, target))
        }
        ChoiceOption::BlockerAttacker(blocker, attacker) => format!("{} blocks {}", n(blocker), n(attacker)),
        ChoiceOption::NormalCost => "Its mana cost".to_string(),
        ChoiceOption::AlternativeCost(cost) => alternative_cost_label(cost),
        ChoiceOption::AdditionalCost(cost) => additional_cost_label(cost),
        ChoiceOption::Number(number) => number.to_string(),
        ChoiceOption::Color(color) => color_name(*color).to_string(),
        ChoiceOption::CounterType(counter) => counter.name().to_string(),
        ChoiceOption::ManaType(mana) => ManaSymbol::Colored(*mana).to_string(),
    }
}

/// Why a seat is being asked again: the answer the engine rejected, and the
/// rule that rejected it.
pub fn rejection(game: &GameState, rejected: &Rejection) -> String {
    match rejected {
        Rejection::Reversed(_) => {
            let (words, _) = rejection_words(game, rejected);
            format!("{words} (CR 732.1)")
        }
        Rejection::IllegalBlocks { why, .. } => format!("Those blocks are illegal: {}", combat_error(game, why)),
    }
}

/// [`rejection`]'s words with the rule apart, for a client that shows the
/// rule on its own.
pub fn rejection_words(game: &GameState, rejected: &Rejection) -> (String, Option<&'static str>) {
    match rejected {
        Rejection::Reversed(action) => (
            format!(
                "{} could not be completed, so it was reversed and its payments canceled",
                option_label(game, &ChoiceOption::Action(action.clone())),
            ),
            Some("732.1"),
        ),
        Rejection::IllegalBlocks { why, .. } => {
            let (words, rule) = combat_refusal(game, why, Declaring::Blockers);
            (format!("Those blocks are illegal: {words}"), rule)
        }
    }
}

/// Which declaration a combat error refused, since CR 508.1a and 509.1a
/// state the same requirement for each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Declaring {
    Attackers,
    Blockers,
}

/// The rule a combat declaration broke, as the re-asked question shows it.
fn combat_error(game: &GameState, error: &CombatError) -> String {
    match combat_refusal(game, error, Declaring::Blockers) {
        (words, Some(rule)) => format!("{words} (CR {rule})"),
        (words, None) => words,
    }
}

/// Why a creature can't attack or block as `error` says, and the rule: one
/// arm per error and no wildcard.
pub fn combat_refusal(game: &GameState, error: &CombatError, refused: Declaring) -> (String, Option<&'static str>) {
    let n = |id: &ObjectId| named(game, *id);
    let declaration_rule = match refused {
        Declaring::Attackers => "508.1a",
        Declaring::Blockers => "509.1a",
    };
    match error {
        CombatError::NotOnBattlefield(id) => (format!("{} is not on the battlefield", n(id)), None),
        CombatError::NotACreature(id) => (format!("{} is not a creature", n(id)), None),
        CombatError::NotControlledByPlayer(id, player) => {
            (format!("{} is not controlled by {}", n(id), player_name(*player)), None)
        }
        CombatError::CreatureIsTapped(id) => (format!("{} is tapped", n(id)), Some(declaration_rule)),
        CombatError::CreatureHasSummoningSickness(id) => (
            format!("{} has not been under its controller's control since their turn began", n(id)),
            Some("302.6"),
        ),
        CombatError::InvalidAttackTarget(id) => (format!("{} can't attack that", n(id)), None),
        CombatError::AttackerNotAttackingThisPlayer(blocker, attacker) => (
            format!("{} is not attacking you, so {} can't block it", n(attacker), n(blocker)),
            Some("509.1a"),
        ),
        CombatError::TooManyBlocks(id, 1) => (format!("{} can block only one attacker", n(id)), Some("509.1a")),
        CombatError::TooManyBlocks(id, max) => (format!("{} can block only {max} attackers", n(id)), None),
        CombatError::HasDefender(id) => (format!("{} has defender and can't attack", n(id)), Some("702.3b")),
        CombatError::CantBlockFlyer(blocker, attacker) => (
            format!("{} has flying, and {} has neither flying nor reach", n(attacker), n(blocker)),
            Some("702.9b"),
        ),
        // A restriction's own words, until RS-3 gives restrictions ids.
        CombatError::ConstraintViolation(text) => (text.clone(), None),
    }
}

/// The end of a sentence saying which of CR 307.1's conditions `player` misses.
fn timing_miss(player: PlayerId, miss: SorceryTiming) -> String {
    match miss {
        SorceryTiming::NotYourTurn => format!("it is not {}'s turn", player_name(player)),
        SorceryTiming::NotAMainPhase => "it is not a main phase".to_string(),
        SorceryTiming::StackNotEmpty => "the stack is not empty".to_string(),
    }
}

/// Why `player` is not offered `card` to cast, and the rule: one arm per
/// reason and no wildcard.
pub fn cannot_cast(game: &GameState, player: PlayerId, card: ObjectId, reason: &CannotCast) -> (String, Option<&'static str>) {
    let who = player_name(player);
    match reason {
        CannotCast::NotInHand => (format!("it is not in {who}'s hand, and a spell is cast from its caster's hand"), Some("601.3")),
        CannotCast::Land => ("a land is played, never cast".to_string(), Some("305.9")),
        CannotCast::NoSpellAbility => ("it has no spell ability, so nothing would resolve".to_string(), None),
        CannotCast::Timing(miss) => (
            format!(
                "it is not an instant and has no flash, so it is cast only in its caster's main phase with the stack empty, and {}",
                timing_miss(player, *miss),
            ),
            Some("117.1a"),
        ),
        CannotCast::NoLegalTarget => ("one of its targets has no legal choice".to_string(), Some("601.2c")),
        CannotCast::AdditionalCost(cost) => {
            let (words, rule) = cannot_pay(game, player, card, cost);
            (format!("its additional cost can't be paid: {words}"), rule)
        }
        CannotCast::ManaShort => {
            // The total CR 601.2f would lock in, which is what the
            // enumeration compared (`cost-architecture.md` §3.6).
            let printed = game.objects.get(&card).and_then(|obj| obj.card_data.mana_cost.clone());
            let cost = printed.map_or_else(String::new, |printed| {
                format!(" {}", crate::engine::cost_determination::preview_mana_cost(game, card, &printed))
            });
            (format!("the mana {who} can make now does not cover its cost{cost}"), Some("601.2h"))
        }
    }
}

/// Why `player` is not offered `card` to play as a land, and the rule.
pub fn cannot_play_land(reason: &CannotPlayLand, player: PlayerId) -> (String, Option<&'static str>) {
    let who = player_name(player);
    match reason {
        CannotPlayLand::NotInHand => (format!("it is not in {who}'s hand, and a land is played from its owner's hand"), Some("305.1")),
        CannotPlayLand::NotALand => ("it is not a land".to_string(), Some("305.1")),
        CannotPlayLand::Timing(miss) => (
            format!("a land is played only in its owner's main phase with the stack empty, and {}", timing_miss(player, *miss)),
            Some("305.1"),
        ),
        CannotPlayLand::NoLandDropLeft { played: 1, allowed: 1 } => {
            (format!("{who} has played a land this turn already"), Some("305.2"))
        }
        CannotPlayLand::NoLandDropLeft { played, allowed } => {
            (format!("{who} has played {played} of the {allowed} lands they may play this turn"), Some("305.2a"))
        }
    }
}

/// Why `player` is not offered `ability` of `source` to activate, and the
/// rule.
pub fn cannot_activate(
    game: &GameState,
    player: PlayerId,
    source: ObjectId,
    ability: &AbilityDef,
    reason: &CannotActivate,
) -> (String, Option<&'static str>) {
    let who = player_name(player);
    match reason {
        CannotActivate::NotOnBattlefield => {
            ("it is not on the battlefield, where its abilities function".to_string(), Some("113.6"))
        }
        CannotActivate::NotYours => {
            let controller = controller_or_owner(game, source).map_or_else(|| "another player".to_string(), player_name);
            (format!("{controller} controls it, and only its controller activates its abilities"), Some("602.2"))
        }
        CannotActivate::ManaAbility => (
            "this engine offers a mana ability only while a cost asks for mana, though a player may activate one whenever they have priority".to_string(),
            Some("605.3a"),
        ),
        CannotActivate::NotActivated => ("it is not an activated ability".to_string(), Some("602.1")),
        CannotActivate::Timing(miss) => (
            format!(
                "it is activated only as a sorcery, in its controller's main phase with the stack empty, and {}",
                timing_miss(player, *miss),
            ),
            Some("602.5d"),
        ),
        CannotActivate::Cost(cost) => cannot_pay(game, player, source, cost),
        CannotActivate::ManaShort => {
            let cost = ability.costs.iter().find_map(|cost| match cost {
                Cost::Mana(mana) => Some(format!(" {mana}")),
                _ => None,
            });
            (format!("the mana {who} can make now does not cover its cost{}", cost.unwrap_or_default()), Some("602.2b"))
        }
        CannotActivate::NoLegalTarget => ("one of its targets has no legal choice".to_string(), Some("601.2c")),
    }
}

/// Why a cost of `source`'s can't be paid, and the rule.
pub fn cannot_pay(game: &GameState, player: PlayerId, source: ObjectId, reason: &CannotPay) -> (String, Option<&'static str>) {
    let who = player_name(player);
    match reason {
        CannotPay::SourceGone(_) => ("it is not on the battlefield to pay with".to_string(), Some("118.3")),
        CannotPay::AlreadyTapped => ("it is already tapped, so it can't be tapped to pay {T}".to_string(), Some("118.3")),
        CannotPay::NotTapped => ("it is untapped, so it can't be untapped to pay {Q}".to_string(), Some("118.3")),
        CannotPay::SummoningSick => {
            let controller = controller_or_owner(game, source).map_or_else(|| who.clone(), player_name);
            (
                format!("it has not been under {controller}'s control since their most recent turn began, so it can't pay {{T}} or {{Q}}"),
                Some("302.6"),
            )
        }
        CannotPay::PoolShort => (format!("{who}'s mana pool does not hold the mana"), Some("601.2h")),
        CannotPay::TooLittleLife { life, amount } => (format!("{who} has {life} life, less than the {amount} it costs"), Some("119.4")),
        CannotPay::TooFewToSacrifice { matching: 0, .. } => (format!("{who} controls nothing it could sacrifice"), Some("701.21a")),
        CannotPay::TooFewToSacrifice { matching, needed } => (
            format!("{who} controls only {matching} of the {needed} permanents it sacrifices"),
            Some("701.21a"),
        ),
        CannotPay::Unchecked => ("the engine checks no cost of this kind yet, so it never pays one".to_string(), None),
    }
}

/// The text of the ability with this id on the object's effective list, which
/// is where the activation's options came from.
fn ability_text(game: &GameState, id: ObjectId, ability: AbilityId) -> &'static str {
    get_effective_abilities(game, id)
        .iter()
        .find(|def| def.id == ability)
        .map_or("an ability it no longer has", |def| def.rules_text.words)
}

pub(crate) fn alternative_cost_label(cost: &AlternativeCost) -> String {
    match cost {
        AlternativeCost::Flashback(costs) => keyword_and_cost("Flashback", costs),
        AlternativeCost::Overload(costs) => keyword_and_cost("Overload", costs),
        AlternativeCost::Dash(costs) => keyword_and_cost("Dash", costs),
        AlternativeCost::Escape(costs) => keyword_and_cost("Escape", costs),
        AlternativeCost::Evoke(costs) => keyword_and_cost("Evoke", costs),
        AlternativeCost::Bestow(costs) => keyword_and_cost("Bestow", costs),
        AlternativeCost::Custom(text, _) => text.clone(),
    }
}

pub(crate) fn additional_cost_label(cost: &AdditionalCost) -> String {
    match cost {
        AdditionalCost::Kicker(costs) => keyword_and_cost("Kicker", costs),
        AdditionalCost::Buyback(costs) => keyword_and_cost("Buyback", costs),
        AdditionalCost::Entwine(costs) => keyword_and_cost("Entwine", costs),
        AdditionalCost::Casualty(n) => format!("Casualty {n}"),
        AdditionalCost::Bargain => "Bargain".to_string(),
        // An ability word (CR 207.2c), with no "[keyword] [cost]" form.
        AdditionalCost::Strive(_) => "Strive".to_string(),
        AdditionalCost::Custom(text, _) => text.clone(),
        // CR 601.2b announces only an optional cost, so this is never an
        // option; it prints with no name.
        AdditionalCost::Mandatory(_) => "Its additional cost".to_string(),
    }
}

/// "Kicker {2}": a cost keyword as it prints, "[keyword] [cost]" (CR 702.33a
/// and its neighbors), while every part of the cost is mana. A part that
/// isn't, such as escape's exile, prints the keyword alone: nothing carries
/// that part's printed words.
fn keyword_and_cost(keyword: &str, costs: &[Cost]) -> String {
    let mana: Option<Vec<String>> = costs
        .iter()
        .map(|cost| match cost {
            Cost::Mana(mana) if mana.symbols.is_empty() => Some("{0}".to_string()),
            Cost::Mana(mana) => Some(mana.to_string()),
            Cost::TapSelf
            | Cost::UntapSelf
            | Cost::PayLife(_)
            | Cost::SacrificeSelf
            | Cost::Sacrifice(..)
            | Cost::Discard(..)
            | Cost::ExileFromGraveyard(..)
            | Cost::RemoveCounters(..)
            | Cost::AddCounters(..) => None,
        })
        .collect();
    match mana {
        Some(symbols) if !symbols.is_empty() => format!("{keyword} {}", symbols.concat()),
        _ => keyword.to_string(),
    }
}

pub(crate) fn color_name(color: Color) -> &'static str {
    match color {
        Color::White => "White",
        Color::Blue => "Blue",
        Color::Black => "Black",
        Color::Red => "Red",
        Color::Green => "Green",
    }
}

/// Format the current phase/step for display.
pub fn format_phase(game: &GameState) -> String {
    let phase = phase_name(game.phase.phase_type);
    match game.phase.step {
        Some(step) => format!("{} — {}", phase, step_name(step)),
        None => phase.to_string(),
    }
}

/// A phase as a person reads it, which a scenario's `step` word reuses.
pub fn phase_name(phase: PhaseType) -> &'static str {
    match phase {
        PhaseType::Beginning => "Beginning",
        PhaseType::Precombat => "Precombat Main",
        PhaseType::Combat => "Combat",
        PhaseType::Postcombat => "Postcombat Main",
        PhaseType::Ending => "Ending",
    }
}

/// A step as a person reads it.
pub fn step_name(step: StepType) -> &'static str {
    match step {
        StepType::Untap => "Untap",
        StepType::Upkeep => "Upkeep",
        StepType::Draw => "Draw",
        StepType::BeginCombat => "Begin Combat",
        StepType::DeclareAttackers => "Declare Attackers",
        StepType::DeclareBlockers => "Declare Blockers",
        StepType::FirstStrikeDamage => "First Strike Damage",
        StepType::CombatDamage => "Combat Damage",
        StepType::EndCombat => "End Combat",
        StepType::End => "End",
        StepType::Cleanup => "Cleanup",
    }
}

// ---------------------------------------------------------------------------
// Event log formatting
// ---------------------------------------------------------------------------

/// "Grizzly Bears (#12)", the shape every line naming an object uses, or
/// "Grizzly Bears (Clone, #12)" when `name` is not its card's, so a copy reads
/// as what it is.
pub fn object_label(game: &GameState, id: ObjectId, name: &str) -> String {
    // AS PRINTED: the card's own name, beside the one it has now.
    match game.objects.get(&id).map(|obj| obj.card_data.name.as_str()) {
        Some(card) if card != name => format!("{name} ({card}, {id})"),
        _ => format!("{name} ({id})"),
    }
}

/// The object under the name the record kept for it, where a copy made it
/// other than its card's, else its card's; the bare id for an object the
/// store no longer holds.
fn name_with_id(game: &GameState, id: ObjectId, announced: &NamesAsAnnounced) -> String {
    let kept = announced.as_deref().and_then(|names| names.iter().find(|(named, _)| *named == id));
    match (kept, game.objects.get(&id)) {
        (Some((_, name)), _) => object_label(game, id, name),
        // AS PRINTED: no name was kept, so the card's own.
        (None, Some(obj)) => format!("{} ({})", obj.card_data.name, id),
        (None, None) => format!("{}", id),
    }
}

/// An object as it was in the zone it left: its look-back frame from the
/// battlefield (CR 603.10a), its card anywhere else.
fn as_it_left(game: &GameState, id: ObjectId, lki: &Option<Arc<EffectiveCharacteristics>>) -> String {
    match lki {
        Some(frame) => object_label(game, id, &frame.name),
        None => name_with_id(game, id, &None),
    }
}

/// Why a spell or ability fizzled, in CR 608.2b's words.
const DOES_NOT_RESOLVE: &str = "doesn't resolve: every target is illegal (CR 608.2b)";

/// The delayed triggered ability `identity` is, by the number its
/// creation's line gave it (CR 603.7), or `None` for an ability of an object.
fn delayed_trigger(identity: AbilityIdentity) -> Option<DelayedTriggerId> {
    identity.ability.delayed_trigger()
}

/// "ability", or "delayed trigger 3" for a delayed triggered ability.
fn ability_word(identity: AbilityIdentity) -> String {
    match delayed_trigger(identity) {
        Some(id) => format!("delayed trigger {}", id.0),
        None => "ability".to_string(),
    }
}

/// Format one event, each object named as its record kept it.
pub fn format_event(game: &GameState, event: &GameEvent, announced: &NamesAsAnnounced) -> String {
    use crate::events::event::CounterSubject;
    use crate::events::event::GameEvent::*;
    let obj_name = |game: &GameState, id: ObjectId| name_with_id(game, id, announced);
    match event {
        ZoneChange { object_id, owner, from, to, cause, lki } => {
            // The cause says which rule moved it and the CR 603.10a frame says every
            // type it had, which is what a Gideon or an artifact creature needs.
            let was = lki.as_ref().map(|f| {
                // Sorted: `types` is a `HashSet`, and an unsorted log line
                // differs run to run.
                let mut names: Vec<String> = f.types.iter().map(|t| format!("{:?}", t)).collect();
                names.sort();
                format!(" ({})", names.join(" "))
            }).unwrap_or_default();
            format!("ZoneChange: {}{} [P{}] {:?} -> {:?} [{:?}]",
                    as_it_left(game, *object_id, lki), was, owner, from, to, cause)
        }
        AbilityActivated { identity, controller } => format!(
            "AbilityActivated: {} [P{}]", obj_name(game, identity.source.id), controller),
        AbilityTriggered { seq, origin, controller, .. } => match delayed_trigger(origin.identity()) {
            Some(id) => format!("AbilityTriggered: {}'s delayed trigger {} [P{}]", obj_name(game, origin.source()), id.0, controller),
            None => format!("AbilityTriggered: {} [P{}] #{}", obj_name(game, origin.source()), controller, seq.0),
        },
        AbilityResolved { identity, controller } => match delayed_trigger(*identity) {
            Some(id) => format!("AbilityResolved: {}'s delayed trigger {} [P{}]", obj_name(game, identity.source.id), id.0, controller),
            None => format!("AbilityResolved: {} [P{}]", obj_name(game, identity.source.id), controller),
        },
        DelayedTriggerCreated { id, source, controller, rules_text, duration } => {
            let fires = match duration {
                DelayedDuration::Once => "once",
                DelayedDuration::ThisTurn => "each time this turn",
            };
            format!(
                "DelayedTriggerCreated: {}'s delayed trigger {} [P{}], {fires}: \"{}\"",
                obj_name(game, *source),
                id.0,
                controller,
                rules_text.words
            )
        }
        Targeted { target, by, ability_source, controller, instances } => {
            let target = match target {
                TargetRef::Object(object) => obj_name(game, object.id),
                TargetRef::Player(player) => format!("P{player}"),
            };
            let by = match ability_source {
                Some(source) => format!("{}'s ability", obj_name(game, *source)),
                None => obj_name(game, *by),
            };
            let instances = if *instances > 1 { format!(", chosen for {instances} instances") } else { String::new() };
            format!("Targeted: {target} by {by} [P{controller}]{instances}")
        }
        Tapped { object_id } => format!("Tapped: {}", obj_name(game, *object_id)),
        Untapped { object_id } => format!("Untapped: {}", obj_name(game, *object_id)),
        CardDrawn { player_id, card_id } => {
            format!("CardDrawn: P{} drew {}", player_id, obj_name(game, *card_id))
        }
        ManaAdded { player_id, source_id, mana, tapped_for_mana } => {
            let mana_str: Vec<String> = mana.iter()
                .map(|(t, v)| format!("{:?}:{}", t, v))
                .collect();
            format!(
                "ManaAdded: P{} from {} [{}]{}",
                player_id,
                obj_name(game, *source_id),
                mana_str.join(", "),
                if *tapped_for_mana { " tapped for mana" } else { "" },
            )
        }
        DamageDealt { source_id, source_frame, target, amount, is_combat } => {
            let target_str = match target {
                crate::events::event::DamageTarget::Player(pid) => format!("P{}", pid),
                crate::events::event::DamageTarget::Object(oid) => obj_name(game, *oid),
            };
            // A source that had left dealt it as it last existed.
            let source = match source_frame {
                Some(frame) => object_label(game, *source_id, &frame.name),
                None => obj_name(game, *source_id),
            };
            format!(
                "DamageDealt: {} -> {} for {}{}",
                source,
                target_str,
                amount,
                if *is_combat { " (combat)" } else { "" },
            )
        }
        PhaseBegin { phase, player } => format!("PhaseBegin: {:?} [P{}]", phase, player),
        StepBegin { step, player } => format!("StepBegin: {:?} [P{}]", step, player),
        TurnBegin { player, turn_number } => format!("TurnBegin: P{} turn {}", player, turn_number),
        PermanentEnteredBattlefield { object_id, controller } => {
            format!("ETB: {} [P{}]", obj_name(game, *object_id), controller)
        }
        LifeChanged { player_id, old, new, source, .. } => {
            let src = match source {
                Some(id) => format!(" (source: {})", obj_name(game, *id)),
                None => String::new(),
            };
            format!("LifeChanged: P{} {} -> {}{}", player_id, old, new, src)
        }
        AttackersDeclared { attackers } => {
            let names: Vec<String> = attackers.iter().map(|id| obj_name(game, *id)).collect();
            format!("AttackersDeclared: [{}]", names.join(", "))
        }
        BlockersDeclared { blockers } => {
            let pairs: Vec<String> = blockers.iter()
                .map(|(b, a)| format!("{} blocks {}", obj_name(game, *b), obj_name(game, *a)))
                .collect();
            format!("BlockersDeclared: [{}]", pairs.join(", "))
        }
        SpellCast { spell_id, caster } => {
            format!("SpellCast: P{} casts {}", caster, obj_name(game, *spell_id))
        }
        SpellCountered { spell_id, controller, countered_by } => format!(
            "SpellCountered: {} [P{}] countered by {}", obj_name(game, *spell_id), controller, obj_name(game, *countered_by)),
        AbilityCountered { identity, controller, countered_by } => format!(
            "AbilityCountered: {}'s {} [P{}] countered by {}",
            obj_name(game, identity.source.id), ability_word(*identity), controller, obj_name(game, *countered_by)),
        SpellFizzled { spell_id, controller } => format!("SpellFizzled: {} [P{}] {DOES_NOT_RESOLVE}", obj_name(game, *spell_id), controller),
        AbilityFizzled { identity, controller } => format!(
            "AbilityFizzled: {}'s {} [P{}] {DOES_NOT_RESOLVE}", obj_name(game, identity.source.id), ability_word(*identity), controller),
        PlayerLost { player_id, reason } => {
            format!("PlayerLost: P{} ({:?})", player_id, reason)
        }
        PlayerWon { player_id } => format!("PlayerWon: P{}", player_id),
        CountersChanged { subject, counter, added } => {
            let verb = if *added >= 0 { "put on" } else { "removed from" };
            let whom = match subject {
                CounterSubject::Object(id) => obj_name(game, *id),
                CounterSubject::Player(pid) => format!("P{}", pid),
            };
            format!("CountersChanged: {} {:?} counter(s) {} {}", added.abs(), counter, verb, whom)
        }
        Scried { player_id, n, looked_at } => {
            // Both numbers, and they differ only on a short library — which is
            // the case Elrond, Master of Healing's ruling is about, so a log
            // that showed one would hide it.
            format!("Scried: P{} scry {} (looked at {})", player_id, n, looked_at)
        }
        LibraryShuffled { player_id } => format!("LibraryShuffled: P{}", player_id),
        CountersAnnihilated { object_id, pairs_removed } => {
            format!("CountersAnnihilated: {} ({} pairs)", obj_name(game, *object_id), pairs_removed)
        }
        Attached { attachment, host, former_host } => match former_host {
            Some(former) => format!(
                "Attached: {} to {} (from {})",
                obj_name(game, *attachment), obj_name(game, *host), obj_name(game, *former)
            ),
            None => format!("Attached: {} to {}", obj_name(game, *attachment), obj_name(game, *host)),
        },
        EquipmentDetached { equipment_id, former_host } => {
            format!("EquipmentDetached: {} from {}", obj_name(game, *equipment_id), obj_name(game, *former_host))
        }
        LeftTheGame { object_id, owner, from, lki } => {
            format!("LeftTheGame: {} (P{}, from {:?})", as_it_left(game, *object_id, lki), owner, from)
        }
        TokenCreated { object_id, owner, zone } => {
            format!("TokenCreated: {} (P{}, in {:?})", obj_name(game, *object_id), owner, zone)
        }
        TokenCeasedToExist { object_id } => {
            format!("TokenCeasedToExist: {}", obj_name(game, *object_id))
        }
        StateBasedActionPerformed => "StateBasedActionPerformed".to_string(),
    }
}

/// Format every recorded event with resolved card names. The game must be
/// recording (`GameState::record_events`).
pub fn format_event_log(game: &GameState) -> Vec<String> {
    game.recorded_events()
        .records()
        .iter()
        .map(|record| format_event(game, &record.event, &record.names))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::PermanentState;
    use crate::state::game_state::{GameState, Phase};
    use crate::types::card_types::CardType;
    use crate::types::zones::Zone;

    #[test]
    fn test_card_name() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Forest").card_type(CardType::Land).build();
        let obj = GameObject::new(data, 0, Zone::Hand);
        let id = game.add_object(obj);

        assert_eq!(card_name(&game, id), "Forest");
    }

    #[test]
    fn test_format_permanent_creature() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = game.add_object(obj);
        let entry = PermanentState::new(id, 0, 0);
        game.insert_battlefield_entity(id, entry);

        let display = format_permanent(&game, id);
        assert!(display.contains("Grizzly Bears"));
        assert!(display.contains("2/2"));
    }

    #[test]
    fn test_format_permanent_tapped() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = game.add_object(obj);
        let mut entry = PermanentState::new(id, 0, 0);
        entry.tapped = true;
        game.insert_battlefield_entity(id, entry);

        let display = format_permanent(&game, id);
        assert!(display.contains("tapped"));
    }

    #[test]
    fn test_format_phase() {
        let mut game = GameState::new(2, 20);
        game.set_turn_position(Phase::new(PhaseType::Precombat));
        assert_eq!(format_phase(&game), "Precombat Main");

        game.set_turn_position(Phase::new(PhaseType::Beginning));
        assert!(format_phase(&game).contains("Untap"));
    }

    #[test]
    fn test_format_permanent_with_mana_ability() {
        use crate::types::card_types::*;
        use crate::types::mana::ManaType;

        let mut game = GameState::new(2, 20);
        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .mana_ability_single(ManaType::Green)
            .build();
        let obj = GameObject::new(forest, 0, Zone::Battlefield);
        let id = game.add_object(obj);
        let entry = PermanentState::new(id, 0, 0);
        game.insert_battlefield_entity(id, entry);

        assert_eq!(format_permanent(&game, id), "Forest\n{T}: Add {G}.");
    }

    #[test]
    fn a_clone_copying_grizzly_bears_shows_as_grizzly_bears() {
        use crate::engine::actions::ActionContext;
        use crate::test_support::{put_in_graveyard, put_on_battlefield, setup_two_player_game, RecordingDecisionProvider};
        use crate::types::zones::ZoneChangeCause;

        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, crate::cards::creatures::grizzly_bears(), 0);
        let clone = put_in_graveyard(&mut game, crate::cards::phase_cv_cards::clone(), 0);
        let dp = RecordingDecisionProvider::picking(0);
        game.change_zone(clone, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp)).unwrap();
        assert!(dp.kinds()[0] == "ChooseCopySource", "{:?}", dp.kinds());

        assert_eq!(card_name(&game, clone), "Grizzly Bears");
        assert_eq!(format_permanent(&game, clone), "Grizzly Bears 2/2 (sick)");
        assert_eq!(
            printed_faces(&game, clone),
            ["Clone {3}{U}\nCreature — Shapeshifter\nYou may have this creature enter as a copy of any creature on the battlefield.\n0/0"],
            "as printed, it is still Clone"
        );
    }
    #[test]
    fn the_log_names_a_copy_as_it_was_at_each_event() {
        use crate::engine::actions::{ActionContext, GameAction};
        use crate::test_support::{
            put_in_graveyard, put_on_battlefield, setup_two_player_game, test_ctx, RecordingDecisionProvider,
        };
        use crate::types::zones::ZoneChangeCause;

        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, crate::cards::creatures::grizzly_bears(), 0);
        let clone = put_in_graveyard(&mut game, crate::cards::phase_cv_cards::clone(), 0);
        let dp = RecordingDecisionProvider::picking(0);
        game.change_zone(clone, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp)).unwrap();
        game.execute_action(GameAction::Tap { object: clone }, &test_ctx()).unwrap();
        game.change_zone(clone, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx()).unwrap();
        // And one made a copy by a row, as Cytoshape does (CR 707.2), until
        // the row ends.
        let shaped = put_on_battlefield(&mut game, crate::test_support::vanilla_creature(1, 1, &[]), 0);
        let values = crate::engine::layers::copy::copiable_values(&game, bears).unwrap();
        let timestamp = game.allocate_timestamp();
        let row = crate::test_support::registered(
            shaped,
            crate::engine::layers::types::Layer::Layer1Copy,
            timestamp,
            crate::engine::layers::types::EffectModification::CopyFrom(Arc::new(values)),
        );
        let row = game.continuous_effects.add(row);
        game.execute_action(GameAction::Tap { object: shaped }, &test_ctx()).unwrap();
        game.continuous_effects.remove(row);
        game.execute_action(GameAction::Untap { object: shaped }, &test_ctx()).unwrap();

        // Formatted after it died: each line names it as it was then. A zone
        // change names the object as it was in the zone it left.
        let log = format_event_log(&game);
        let has = |line: String| assert!(log.contains(&line), "{line}\n{log:#?}");
        has(format!("ZoneChange: Clone ({clone}) [P0] Graveyard -> Battlefield [Returned]"));
        has(format!("ETB: Grizzly Bears (Clone, {clone}) [P0]"));
        has(format!("Tapped: Grizzly Bears (Clone, {clone})"));
        has(format!("ZoneChange: Grizzly Bears (Clone, {clone}) (Creature) [P0] Battlefield -> Graveyard [Destroyed]"));
        has(format!("Tapped: Grizzly Bears (Test Creature, {shaped})"));
        has(format!("Untapped: Test Creature ({shaped})"));
    }

    #[test]
    fn a_type_line_prints_in_printed_order() {
        use crate::cards::{dual_lands, phase_rc_cards, phase_rg_cards, phase_tr2b_cards};
        let line = |card: Arc<CardData>| type_line(&card.supertypes, &card.types, &card.subtypes);
        assert_eq!(line(phase_rc_cards::containment_priest()), "Creature — Human Cleric");
        assert_eq!(line(phase_rc_cards::dryad_arbor()), "Land Creature — Forest Dryad");
        assert_eq!(line(phase_tr2b_cards::nykthos_paragon()), "Enchantment Creature — Human Soldier");
        assert_eq!(line(phase_rg_cards::archelos_lagoon_mystic()), "Legendary Creature — Turtle Shaman");
        assert_eq!(line(dual_lands::bayou()), "Land — Swamp Forest");
    }

    fn words(section: &[TypeWord]) -> Vec<(&str, TypeWordStatus)> {
        section.iter().map(|word| (word.text.as_str(), word.status)).collect()
    }

    #[test]
    fn blood_moon_leaves_bayous_land_types_in_place_as_lost_and_adds_mountain_last() {
        use crate::test_support::{put_on_battlefield, setup_two_player_game};
        use TypeWordStatus::{Gained, Kept, Lost};

        let mut game = setup_two_player_game();
        let bayou = put_on_battlefield(&mut game, crate::cards::dual_lands::bayou(), 0);
        assert_eq!(type_line_now(&game, bayou).to_string(), "Land — Swamp Forest");
        put_on_battlefield(&mut game, crate::cards::phase_ld_cards::blood_moon(), 1);

        let line = type_line_now(&game, bayou);
        assert_eq!(words(&line.front), [("Land", Kept)]);
        assert_eq!(words(&line.subtypes), [("Swamp", Lost), ("Forest", Lost), ("Mountain", Gained)]);
        assert_eq!(line.to_string(), "Land — Mountain");
    }

    #[test]
    fn a_gained_type_takes_its_place_and_a_gained_subtype_goes_last() {
        use crate::engine::layers::types::{EffectModification, Layer};
        use crate::test_support::{put_on_battlefield, registered, setup_two_player_game};
        use crate::types::card_types::CreatureType;
        use TypeWordStatus::{Gained, Kept, Lost};

        let mut game = setup_two_player_game();
        let bear = CardDataBuilder::new("Test Bear")
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(CreatureType::Bear))
            .power_toughness(2, 2)
            .build();
        let bear = put_on_battlefield(&mut game, bear, 0);
        for modification in [
            EffectModification::AddType(CardType::Artifact),
            EffectModification::AddSupertype(Supertype::Legendary),
            EffectModification::AddSubtype(Subtype::Creature(CreatureType::Elf)),
            EffectModification::SetSubtypes(Subtypes::from([Subtype::Creature(CreatureType::Goblin)])),
        ] {
            let timestamp = game.allocate_timestamp();
            game.continuous_effects.add(registered(bear, Layer::Layer4Type, timestamp, modification));
        }

        let line = type_line_now(&game, bear);
        assert_eq!(words(&line.front), [("Legendary", Gained), ("Artifact", Gained), ("Creature", Kept)]);
        assert_eq!(words(&line.subtypes), [("Bear", Lost), ("Goblin", Gained)], "the Elf came and went");
        assert_eq!(line.to_string(), "Legendary Artifact Creature — Goblin");
    }

    #[test]
    fn a_copy_reads_against_what_it_copied_and_every_creature_type_is_one_word() {
        use crate::engine::actions::ActionContext;
        use crate::test_support::{put_in_graveyard, put_on_battlefield, setup_two_player_game, RecordingDecisionProvider};
        use crate::types::card_types::CreatureType;
        use crate::types::zones::ZoneChangeCause;

        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, crate::cards::phase_rd_cards::samite_healer(), 0);
        let clone = put_in_graveyard(&mut game, crate::cards::phase_cv_cards::clone(), 0);
        let dp = RecordingDecisionProvider::picking(0);
        game.change_zone(clone, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp)).unwrap();
        let line = type_line_now(&game, clone);
        assert!(line.front.iter().chain(&line.subtypes).all(|word| word.status == TypeWordStatus::Kept), "{line:?}");
        assert_eq!(line.to_string(), "Creature — Human Cleric", "its own Shapeshifter is no part of it");

        let mut every = (*CardDataBuilder::new("Every-Type Fixture")
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(CreatureType::Shapeshifter))
            .power_toughness(1, 1)
            .build())
        .clone();
        every.subtypes.insert_every_creature_type();
        let every = put_on_battlefield(&mut game, Arc::new(every), 0);
        assert_eq!(type_line_now(&game, every).to_string(), "Creature — Shapeshifter (every creature type)");

        // One pass of the board gives every object the line a pass for it alone would.
        use crate::test_support::put_in_hand;
        put_on_battlefield(&mut game, crate::cards::dual_lands::bayou(), 1);
        put_on_battlefield(&mut game, crate::cards::phase_ld_cards::blood_moon(), 1);
        put_in_hand(&mut game, crate::cards::phase_rc_cards::dryad_arbor(), 1);
        put_in_graveyard(&mut game, crate::cards::phase_rg_cards::archelos_lagoon_mystic(), 1);
        let lines = TypeLines::new(&game);
        for id in game.objects.keys() {
            let now = compute_characteristics(&game, *id).unwrap();
            assert_eq!(lines.of(&game, *id, &now), type_line_now(&game, *id), "{}", named(&game, *id));
        }
    }

    #[test]
    fn every_keyword_flag_prints_in_the_enums_order() {
        use crate::test_support::{put_on_battlefield, setup_two_player_game, vanilla_creature};

        let mut game = setup_two_player_game();
        let flags = [KeywordFlag::Shroud, KeywordFlag::Flying, KeywordFlag::Intimidate, KeywordFlag::Flash];
        let id = put_on_battlefield(&mut game, vanilla_creature(1, 1, &flags), 0);
        assert_eq!(format_permanent(&game, id), "Test Creature 1/1 [flash, flying, intimidate, shroud]");
    }

    fn activation_labels(game: &GameState, id: ObjectId) -> Vec<String> {
        get_effective_abilities(game, id)
            .iter()
            .filter(|ability| matches!(ability.ability_type, AbilityType::Mana | AbilityType::Activated))
            .map(|ability| option_label(game, &ChoiceOption::Action(PriorityAction::ActivateAbility(id, ability.id))))
            .collect()
    }

    /// An activation is its object and the ability's own text: Everywhere's
    /// five, a cost that sacrifices the source, a life payment, a sequence, and
    /// an amount above one.
    #[test]
    fn an_activation_is_labeled_by_its_object_and_its_text() {
        use crate::cards::{artifacts, dual_lands, phase_cm_cards, phase_re_cards, phase_tr2a_cards};
        use crate::test_support::{put_on_battlefield, setup_two_player_game};

        let mut game = setup_two_player_game();
        for (card, texts) in [
            (dual_lands::everywhere(), vec!["{T}: Add {W}.", "{T}: Add {U}.", "{T}: Add {B}.", "{T}: Add {R}.", "{T}: Add {G}."]),
            (phase_cm_cards::mind_stone(), vec!["{T}: Add {C}.", "{1}, {T}, Sacrifice this artifact: Draw a card."]),
            (phase_re_cards::yawgmoths_bargain(), vec!["Pay 1 life: Draw a card."]),
            (
                phase_tr2a_cards::elvish_warmaster(),
                vec!["{5}{G}{G}: Elves you control get +2/+2 and gain deathtouch until end of turn."],
            ),
            (artifacts::sol_ring(), vec!["{T}: Add {C}{C}."]),
        ] {
            let id = put_on_battlefield(&mut game, card, 0);
            let expected: Vec<String> = texts.iter().map(|text| format!("{} · {text}", named(&game, id))).collect();
            assert_eq!(activation_labels(&game, id), expected);
        }
    }

    /// What a permanent's abilities say is read off its effective list: Blood
    /// Moon leaves a dual CR 305.7's Mountain ability, and a creature lists and
    /// activates the ability Citanul Hierophants grants it.
    #[test]
    fn a_permanent_shows_the_text_of_the_abilities_it_has_now() {
        use crate::cards::{creatures, dual_lands, phase_ld_cards, phase_lf_cards};
        use crate::test_support::{put_on_battlefield, setup_two_player_game};

        let mut game = setup_two_player_game();
        let sea = put_on_battlefield(&mut game, dual_lands::underground_sea(), 0);
        assert_eq!(format_permanent(&game, sea), "Underground Sea\n{T}: Add {U}.\n{T}: Add {B}.");
        put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 1);
        assert_eq!(format_permanent(&game, sea), "Underground Sea\n{T}: Add {R}.");

        let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
        put_on_battlefield(&mut game, phase_lf_cards::citanul_hierophants(), 0);
        assert_eq!(format_permanent(&game, bears), "Grizzly Bears 2/2\n{T}: Add {G}.");
        assert_eq!(activation_labels(&game, bears), [format!("Grizzly Bears ({bears}) · {{T}}: Add {{G}}.")]);
    }

    /// A printed ability shows once however the engine builds it: Platinum
    /// Angel's second paragraph is two abilities, a can't-lose and a
    /// can't-win. The same words are two abilities when one is printed and
    /// one granted: Dryad Arbor's own mana ability and the one Citanul
    /// Hierophants grants it.
    #[test]
    fn a_printed_ability_shows_once_and_a_granted_one_beside_it() {
        use crate::cards::{phase_lf_cards, phase_rc_cards, phase_re_cards};
        use crate::test_support::{put_on_battlefield, setup_two_player_game};

        let mut game = setup_two_player_game();
        let angel = put_on_battlefield(&mut game, phase_re_cards::platinum_angel(), 0);
        assert_eq!(
            format_permanent(&game, angel),
            "Platinum Angel 4/4 [flying]\nYou can't lose the game and your opponents can't win the game."
        );
        let arbor = put_on_battlefield(&mut game, phase_rc_cards::dryad_arbor(), 0);
        put_on_battlefield(&mut game, phase_lf_cards::citanul_hierophants(), 0);
        assert_eq!(format_permanent(&game, arbor), "Dryad Arbor 1/1\n{T}: Add {G}.\n{T}: Add {G}.");
    }

    /// A cost keyword's option prints as the card does, "[keyword] [cost]",
    /// while the cost is mana, and the keyword alone once a part is not; a
    /// custom cost prints its own sentence.
    #[test]
    fn a_cost_option_prints_its_keyword_and_its_mana() {
        use crate::cards::phase_cm_cards;
        use crate::test_support::setup_two_player_game;
        use crate::types::effects::ObjectFilter;
        use crate::types::mana::{ManaCost, ManaType};

        let game = setup_two_player_game();
        let label = |option: ChoiceOption| option_label(&game, &option);
        let kicker = phase_cm_cards::kicked_lesson().additional_costs[0].clone();
        assert_eq!(label(ChoiceOption::AdditionalCost(kicker)), "Kicker {2}");
        let custom = phase_cm_cards::bargain_lesson().alternative_costs[0].clone();
        assert_eq!(label(ChoiceOption::AlternativeCost(custom)), "Pay {R} rather than pay this spell's mana cost");
        let escape = AlternativeCost::Escape(vec![
            Cost::Mana(ManaCost::build(&[ManaType::Black, ManaType::Black], 3)),
            Cost::ExileFromGraveyard(ObjectFilter::All, 5),
        ]);
        assert_eq!(label(ChoiceOption::AlternativeCost(escape)), "Escape");
    }

    /// A question names its object as it is now: a Clone copying Grizzly Bears
    /// is Grizzly Bears, and says it is a Clone.
    #[test]
    fn a_question_names_its_object_as_it_is_now() {
        use crate::engine::actions::ActionContext;
        use crate::test_support::{put_in_graveyard, put_on_battlefield, setup_two_player_game, RecordingDecisionProvider};
        use crate::types::zones::ZoneChangeCause;

        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, crate::cards::creatures::grizzly_bears(), 0);
        let clone = put_in_graveyard(&mut game, crate::cards::phase_cv_cards::clone(), 0);
        let dp = RecordingDecisionProvider::picking(0);
        game.change_zone(clone, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp)).unwrap();
        assert_eq!(
            question(&game, &ChoiceKind::AssignCombatDamage { attacker_id: clone }),
            format!("Assign Grizzly Bears (Clone, {clone})'s combat damage")
        );
    }

    /// A re-ask says what it rejected by the board's names and the rule.
    #[test]
    fn a_rejection_names_the_answer_and_the_rule() {
        use crate::test_support::{put_in_hand, put_on_battlefield, setup_two_player_game};

        let mut game = setup_two_player_game();
        let bears = put_in_hand(&mut game, crate::cards::creatures::grizzly_bears(), 0);
        let blocker = put_on_battlefield(&mut game, crate::cards::creatures::grizzly_bears(), 1);
        let attacker = put_on_battlefield(&mut game, crate::cards::creatures::grizzly_bears(), 0);
        assert_eq!(
            rejection(&game, &Rejection::Reversed(PriorityAction::CastSpell(bears))),
            format!("Cast Grizzly Bears ({bears}) could not be completed, so it was reversed and its payments canceled (CR 732.1)")
        );
        let blocks = Rejection::IllegalBlocks {
            blocks: vec![(blocker, attacker), (blocker, attacker)],
            why: CombatError::TooManyBlocks(blocker, 1),
        };
        assert_eq!(
            rejection(&game, &blocks),
            format!("Those blocks are illegal: Grizzly Bears ({blocker}) can block only one attacker (CR 509.1a)")
        );
    }
}
