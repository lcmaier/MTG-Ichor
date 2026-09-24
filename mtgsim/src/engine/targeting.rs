use crate::engine::resolve::ResolvedTarget;
use crate::engine::layers::compute::compute_characteristics;
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::oracle::characteristics::{has_type};
use crate::state::game_state::GameState;
use crate::types::card_types::CardType;
use crate::types::effects::{Effect, ObjectFilter, EffectRecipient, SelectionFilter, TargetCount};
use crate::types::ids::{ObjectId, PlayerId};

/// One instance of the word "target" (CR 115.3) — the clause it was chosen
/// against, and what was chosen for it.
///
/// **The clause is recorded rather than re-derived**, for the reason
/// `StackEntry` has always recorded one: an Aura's comes from its
/// `enchant_filter` (CR 303.4a) and nothing in the effect tree could show it.
/// CR 608.2b must re-ask the same question the announcement answered, so the
/// question travels with the answer.
#[derive(Debug, Clone, PartialEq)]
pub struct TargetInstance {
    /// What CR 601.2c announced this instance against.
    pub recipient: EffectRecipient,
    /// The objects and players chosen for it, in the order they were offered.
    pub chosen: Vec<ResolvedTarget>,
}

impl TargetInstance {
    pub fn new(recipient: EffectRecipient, chosen: Vec<ResolvedTarget>) -> Self {
        TargetInstance { recipient, chosen }
    }

    /// Whether this instance *targets* — CR 115.1's word, as opposed to a
    /// non-targeting `Choose`. Only a targeting instance participates in
    /// CR 608.2b.
    pub fn is_targeted(&self) -> bool {
        matches!(self.recipient, EffectRecipient::Target(_, _))
    }
}

/// What each instance of "target" holds, indexed by the instance.
///
/// **A flat buffer and a prefix sum, not a nesting.** `flat` is every
/// instance's targets concatenated in instance order; `bounds[i]` is where
/// instance `i` starts, with a trailing sentinel so `bounds.len()` is one more
/// than the instance count and the last instance needs no special case. Two
/// allocations however many instances there are, where a `Vec<Vec<_>>` is one
/// per instance plus one; and `all()` is the buffer rather than a `flatten`.
///
/// Starts rather than lengths, because the alternative that also fits —
/// `Vec<Range<u32>>` — can represent ranges that are out of order or leave
/// gaps, and this type should not be able to say that. Contiguity is structural
/// here rather than an invariant somebody maintains.
///
/// **Exactly one place indexes it**: `resolve_effect`'s walk, which hands each
/// atom its own instance as a flat slice. Every reader downstream sees
/// `&[ResolvedTarget]` and cannot reach a neighboring instance's targets by
/// accident.
///
/// Distinct from `StackEntry::chosen_targets`, and the difference is CR 608.2b:
/// the entry holds the **announcement**, clause and all, so the re-check can
/// ask the same question; this holds the **survivors** of that re-check. One
/// type for both is how a filtered list leaks back into the thing it was
/// filtered from.
#[derive(Debug, Clone, PartialEq)]
pub struct ChosenTargets {
    /// Every instance's targets, concatenated in instance order.
    flat: Vec<ResolvedTarget>,
    /// Where each instance starts in `flat`, plus the trailing sentinel.
    /// Invariant: non-empty, non-decreasing, and its last element is
    /// `flat.len()`.
    bounds: Vec<u32>,
}

impl ChosenTargets {
    /// **No instances at all** — not an empty list of them. A mana ability's
    /// resolution, a CR 615.5 rider that names nothing, or a filter asked
    /// outside CR 601.2c's loop. `EMPTY` read as "it could be full", which is
    /// not a thing a targetless effect can be.
    ///
    /// **The only spelling of empty, and there is deliberately no `Default`.**
    /// A derived one would have had to pick between `bounds: vec![0]` and
    /// `bounds: vec![]`, and both mean zero instances while the derived
    /// `PartialEq` calls them different values — two ways to say one thing, in
    /// a type that is compared in tests. `NONE` is a `const`, so it is also the
    /// value a builder starts from.
    pub const NONE: ChosenTargets = ChosenTargets { flat: Vec::new(), bounds: Vec::new() };

    /// The single-instance spelling, which is every spell the engine could
    /// cast before CR 601.2c's loop existed and most of them afterwards.
    pub fn one(targets: Vec<ResolvedTarget>) -> Self {
        let n = targets.len() as u32;
        ChosenTargets { flat: targets, bounds: vec![0, n] }
    }

    /// What instance `ix` holds. An out-of-range instance holds nothing rather
    /// than panicking: an atom whose declaring instance found no legal target
    /// is CR 608.2b's unaffected one, not a crash.
    pub fn instance(&self, ix: usize) -> &[ResolvedTarget] {
        match (self.bounds.get(ix), self.bounds.get(ix + 1)) {
            (Some(&from), Some(&to)) => &self.flat[from as usize..to as usize],
            _ => &[],
        }
    }

    /// Every target of every instance, in instance order. CR 608.2b's "all its
    /// targets" is asked of this.
    pub fn all(&self) -> &[ResolvedTarget] {
        &self.flat
    }

    /// How many instances of "target" this holds — never how many targets.
    pub fn len(&self) -> usize {
        self.bounds.len().saturating_sub(1)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Append the next instance's choice. CR 601.2c announces in printed
    /// order, and `ObjectFilter::OtherThanInstance` reads the ones already
    /// pushed, so the order is load-bearing rather than incidental.
    pub fn push(&mut self, targets: impl IntoIterator<Item = ResolvedTarget>) {
        if self.bounds.is_empty() {
            self.bounds.push(0);
        }
        self.flat.extend(targets);
        self.bounds.push(self.flat.len() as u32);
    }
}

impl<I: IntoIterator<Item = ResolvedTarget>> FromIterator<I> for ChosenTargets {
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        let mut out = ChosenTargets::NONE;
        for instance in iter {
            out.push(instance);
        }
        out
    }
}

/// Whether a clause's criteria read an earlier instance of "target" —
/// `ObjectFilter::OtherThanInstance`, the "another target" family.
///
/// **A cost question, not a correctness one.** The announcement loop feeds each
/// instance's choice forward so the next one can exclude it, and building that
/// list means enumerating every legal candidate rather than stopping at the
/// first. For the spells that print — one instance, or several that may share —
/// nothing ever reads it, and doing it anyway cost 3.5% more memo hits per game
/// on a board where no game played differently (A4i's A/B). So the loop asks
/// this first and skips the enumeration when the answer is no.
pub fn clause_reads_earlier_instances(recipient: &EffectRecipient) -> bool {
    fn filter_reads(filter: &ObjectFilter) -> bool {
        match filter {
            ObjectFilter::OtherThanInstance(_) => true,
            ObjectFilter::And(a, b) | ObjectFilter::Or(a, b) => {
                filter_reads(a) || filter_reads(b)
            }
            ObjectFilter::Not(inner) => filter_reads(inner),
            _ => false,
        }
    }
    match recipient {
        EffectRecipient::Target(SelectionFilter::Permanent(f), _)
        | EffectRecipient::Choose(SelectionFilter::Permanent(f), _) => filter_reads(f),
        _ => false,
    }
}

/// What the announced instances look like to `ObjectFilter::OtherThanInstance`,
/// without copying them.
///
/// Two shapes answer the same question. Inside CR 601.2c's loop the earlier
/// choices are a [`ChosenTargets`] being built; inside CR 608.2b's re-check
/// they are the `&[TargetInstance]` already sitting on the `StackEntry`, and
/// collecting *that* into a `ChosenTargets` was a whole copy of the
/// announcement per resolution to serve one by-index read.
///
/// An enum rather than a trait object because both arms are known here, it
/// stays `Copy`, and `NONE` needs no promoted static.
#[derive(Clone, Copy)]
pub enum EarlierTargets<'a> {
    /// Outside both, where the leaf is refused rather than answered.
    None,
    /// CR 601.2c's loop: the instances announced so far.
    Chosen(&'a ChosenTargets),
    /// CR 608.2b's re-check: the announcement on the entry, borrowed.
    Announced(&'a [TargetInstance]),
}

impl EarlierTargets<'_> {
    fn len(&self) -> usize {
        match self {
            EarlierTargets::None => 0,
            EarlierTargets::Chosen(c) => c.len(),
            EarlierTargets::Announced(i) => i.len(),
        }
    }

    fn instance(&self, ix: usize) -> &[ResolvedTarget] {
        match self {
            EarlierTargets::None => &[],
            EarlierTargets::Chosen(c) => c.instance(ix),
            EarlierTargets::Announced(i) => i.get(ix).map_or(&[], |inst| inst.chosen.as_slice()),
        }
    }
}

/// The two facts a filter leaf may need that are **not characteristics**, so
/// that no layer can change them and no frame answers them.
///
/// Both are identity questions and both were already in the tree as one
/// `Option<ObjectId>` parameter; `earlier_targets` is the second, added for CR 601.2c's
/// "another target". A filter may carry both — "another target creature you
/// control other than this one" is a legal English sentence — so they are
/// fields rather than an enum.
///
/// **`exclude_id` is not a third field, and the fold was withdrawn after it was
/// built** (A4i's review, 2026-09-17). It is the same kind of fact, but CR 115.5
/// — "a spell or ability on the stack is an illegal target for itself" — bites on
/// the `Spell` and `DamageSource` arms, and those answer membership directly
/// without ever walking an `ObjectFilter`. A leaf-level fact cannot reach them, so
/// `exclude_id` stays applied in the enumeration, uniformly, whatever the filter's
/// shape.
#[derive(Clone, Copy)]
pub(crate) struct FilterIdentity<'a> {
    /// The source [`ObjectFilter::NotSource`] excludes: the effect's own.
    /// `None` in a selection context, where the leaf is refused rather than
    /// answered.
    pub source: Option<ObjectId>,
    /// The instances of "target" announced before this one, which
    /// [`ObjectFilter::OtherThanInstance`] reads. `None` outside CR 601.2c's
    /// loop and CR 608.2b's re-check, where that leaf is likewise refused.
    pub earlier_targets: EarlierTargets<'a>,
}

impl FilterIdentity<'static> {
    /// Neither fact is available — a plain selection or a look-ahead frame.
    pub const NONE: FilterIdentity<'static> = FilterIdentity {
        source: None,
        earlier_targets: EarlierTargets::None,
    };
}

/// Where a resolution reads CR 601.2c's clauses from, for an atom that refers
/// back to one (`EffectRecipient::SameInstanceAs`).
///
/// **The stack reads the announcement.** `StackEntry::chosen_targets` recorded
/// each clause beside its choice, so the index the announcement filled is the
/// index the resolution reads, and nothing is derived from the effect tree at
/// resolution. `resolve_effect` used to walk the tree again and rest on the
/// second walk numbering the atoms as the first had; there is no second walk
/// now. An Aura's instance comes from its enchant ability and sits in no tree
/// (CR 303.4a), which is why the announcement is the one list that is right
/// for every spell.
///
/// **A bare effect has no announcement** — CR 615.5's rider, and a test that
/// staged its `ResolutionContext` by hand — so its clauses are read off the
/// tree, and only when a `SameInstanceAs` atom asks: a declaring atom is its
/// own clause, and [`instance_of`] never opens this for one.
#[derive(Clone, Copy)]
pub enum DeclaredInstances<'a> {
    /// The stack's: what CR 601.2c announced, clause by clause.
    Announced(&'a [TargetInstance]),
    /// Nothing was announced: the effect's own declaring atoms, in printed order.
    Effect(&'a Effect),
}

impl<'a> DeclaredInstances<'a> {
    fn clause(self, ix: usize) -> Option<&'a EffectRecipient> {
        match self {
            DeclaredInstances::Announced(announced) => announced.get(ix).map(|inst| &inst.recipient),
            DeclaredInstances::Effect(effect) => effect.instance(ix),
        }
    }
}

/// Which instance an atom's recipient resolves against, and the clause that
/// instance was announced with.
///
/// `cursor` is the count of instances *declared* so far by the walk, which is
/// how a declaring atom learns its own index without the tree being numbered.
/// Returns `None` for a recipient that names no instance — `Implicit`,
/// `Controller`, the filtered sweeps and `Host`, none of which reads a
/// chosen target.
pub(crate) fn instance_of<'a>(
    recipient: &'a EffectRecipient,
    declared: DeclaredInstances<'a>,
    cursor: &mut usize,
) -> Option<(usize, &'a EffectRecipient)> {
    match recipient {
        EffectRecipient::Target(_, _) | EffectRecipient::Choose(_, _) => {
            let ix = *cursor;
            *cursor += 1;
            Some((ix, recipient))
        }
        // The declaring atom's clause, not this atom's: an `Instance` atom
        // behaves exactly as the atom that announced the instance did.
        EffectRecipient::SameInstanceAs(ix) => declared.clause(*ix).map(|r| (*ix, r)),
        _ => None,
    }
}

impl GameState {
    /// Validate that chosen targets are legal for the given EffectRecipient.
    ///
    /// Called at cast/activation time (rule 601.2c) and again at resolution
    /// time (rule 608.2b) to check if targets are still legal.
    /// `you` is CR 109.5's "you" for any `ByController(PlayerRef::You)` node
    /// inside the filter — "the controller of the object the ability is on",
    /// which for a spell or activated ability being cast is the player casting
    /// it, and for an Aura's enchant clause is the Aura's controller.
    /// `earlier_targets` is the instances announced before this one, which is what
    /// `ObjectFilter::OtherThanInstance` reads — empty for a spell whose
    /// clauses name no earlier instance, which is all but the "another target"
    /// family.
    pub fn validate_targets(
        &self,
        recipient: &EffectRecipient,
        targets: &[ResolvedTarget],
        you: PlayerId,
        earlier_targets: &ChosenTargets,
    ) -> Result<(), String> {
        match recipient {
            EffectRecipient::Implicit
            | EffectRecipient::ThisObject
            | EffectRecipient::FilteredPermanents { .. }
            | EffectRecipient::FilteredObjectsIn { .. }
            | EffectRecipient::Host => {
                if !targets.is_empty() {
                    return Err("Spell has no targets but targets were provided".to_string());
                }
                Ok(())
            }

            EffectRecipient::Controller => {
                // "You" doesn't use the targets list — the controller is implicit
                Ok(())
            }

            // An instance is validated against the clause that *declared* it;
            // a back-reference never reaches here, because the CR 601.2c loop
            // walks the declared clauses rather than the atoms.
            // A trigger's bound fact is announced by nothing (CR 608.2k — it is
            // "a specific untargeted object"), so there is no clause here.
            EffectRecipient::TriggeringObject | EffectRecipient::TriggeringPlayer => Err(format!(
                "{recipient:?} is a triggered ability's bound fact, not an instance of \"target\" to validate"
            )),

            EffectRecipient::SameInstanceAs(ix) => Err(format!(
                "EffectRecipient::SameInstanceAs({ix}) is a back-reference to an instance of \
                 \"target\", not a clause to validate against (CR 115.3). Validate the \
                 clause `Effect::instances` lists at that index."
            )),

            // Target and Choose validate identically: hexproof, shroud and
            // protection are not checked for `Target` — `codebase-state.md`,
            // "Before card breadth" item 7, which is RS-2's.
            EffectRecipient::Target(filter, count)
            | EffectRecipient::Choose(filter, count) => {
                self.validate_target_count(count, targets.len())?;
                // CR 601.2c, first sentence: "the same target can't be chosen
                // multiple times for any one instance of the word 'target'."
                // Victimize's "two target creature cards" needs two different
                // cards; Decimate's four *instances* may share one artifact
                // land, and do not come through here together.
                //
                // `validate_pick_n` already refuses a duplicate *index*, so no
                // shipped `DecisionProvider` can produce this. That is the
                // provider contract, not the rule — and the rule is what a
                // future target-changing effect (CR 115.7) will be checked
                // against.
                for (i, t) in targets.iter().enumerate() {
                    if targets[..i].contains(t) {
                        return Err(format!(
                            "{:?} was chosen twice for one instance of \"target\" (CR 601.2c)",
                            t
                        ));
                    }
                }
                for t in targets {
                    self.validate_selection(
                        filter,
                        t,
                        you,
                        EarlierTargets::Chosen(earlier_targets),
                    )?;
                }
                Ok(())
            }
        }
    }

    /// Check that the number of targets matches the TargetCount spec.
    fn validate_target_count(
        &self,
        count: &TargetCount,
        actual: usize,
    ) -> Result<(), String> {
        match count {
            TargetCount::Exactly(n) => {
                if actual != *n as usize {
                    return Err(format!(
                        "Expected exactly {} target(s), got {}", n, actual
                    ));
                }
            }
            TargetCount::UpTo(n) => {
                if actual > *n as usize {
                    return Err(format!(
                        "Expected up to {} target(s), got {}", n, actual
                    ));
                }
            }
        }
        Ok(())
    }

    /// Validate a single selected object/player against a SelectionFilter.
    ///
    /// `you` resolves an `ObjectFilter::ByController(PlayerRef::You)` node
    /// (CR 109.5). Only the `Permanent` arm can contain one; the others ignore
    /// it.
    pub(crate) fn validate_selection(
        &self,
        filter: &SelectionFilter,
        target: &ResolvedTarget,
        you: PlayerId,
        earlier_targets: EarlierTargets<'_>,
    ) -> Result<(), String> {
        match filter {
            SelectionFilter::Creature => self.validate_creature_target(target),
            SelectionFilter::Player => self.validate_player_target(target),
            SelectionFilter::Any => self.validate_any_target(target),
            SelectionFilter::Permanent(pf) => {
                self.validate_permanent_target(target, pf, you, earlier_targets)
            }
            SelectionFilter::Spell => self.validate_spell_target(target),
            SelectionFilter::DamageSource => self.validate_damage_source(target),
        }
    }

    /// CR 609.7a — a permanent, or a spell on the stack.
    ///
    /// **No property is asked**, and that is the rule's own last sentence:
    /// "a source doesn't need to be capable of dealing damage to be a legal
    /// choice". A creature with no power, a land, an enchantment already used
    /// this turn — all legal.
    ///
    /// The stack half asks `is_spell`, because CR 609.7a lists a *spell* and
    /// an activated ability on the stack is not one; its source is the
    /// permanent, which the battlefield half already offers. The one object
    /// this loses is the spell or ability **currently resolving**, whose
    /// `StackEntry` `resolve_top_of_stack` has already taken — an effect
    /// choosing its own resolving spell as the source of future damage, which
    /// nothing printed does.
    fn validate_damage_source(&self, target: &ResolvedTarget) -> Result<(), String> {
        match target {
            ResolvedTarget::Object(id) => {
                if self.battlefield.contains_key(id) {
                    return Ok(());
                }
                if self.is_spell_on_stack(*id) {
                    return Ok(());
                }
                Err(format!(
                    "Object {} is neither a permanent nor a spell on the stack, so it is \
                     not a legal source of damage (CR 609.7a)",
                    id
                ))
            }
            ResolvedTarget::Player(_) => {
                Err("Expected a source of damage, got a player".to_string())
            }
        }
    }

    /// Validate a target is a creature on the battlefield.
    fn validate_creature_target(&self, target: &ResolvedTarget) -> Result<(), String> {
        match target {
            ResolvedTarget::Object(id) => {
                self.require_on_battlefield(*id)?;
                self.get_object(*id)?;
                if !has_type(self, *id, CardType::Creature) {
                    return Err(format!("Target {} is not a creature", id));
                }
                Ok(())
            }
            ResolvedTarget::Player(_) => {
                Err("Expected a creature target, got a player".to_string())
            }
        }
    }

    /// Validate a target is a valid player.
    fn validate_player_target(&self, target: &ResolvedTarget) -> Result<(), String> {
        match target {
            ResolvedTarget::Player(pid) => {
                if *pid >= self.players.len() {
                    return Err(format!("Player {} does not exist", pid));
                }
                // CR 800.4a — a player who has left the game is not a player,
                // so not a legal target: not offered at CR 601.2c, and a
                // spell that targeted them before they left has an illegal
                // target at CR 608.2b. Without this a "target player draws"
                // resolving after its target left would draw for a seat the
                // game no longer has.
                if !self.in_game(*pid) {
                    return Err(format!("Player {} has left the game", pid));
                }
                Ok(())
            }
            ResolvedTarget::Object(_) => {
                Err("Expected a player target, got an object".to_string())
            }
        }
    }

    /// Validate "any target" — creature or planeswalker on battlefield, or player.
    fn validate_any_target(&self, target: &ResolvedTarget) -> Result<(), String> {
        match target {
            ResolvedTarget::Player(pid) => {
                if *pid >= self.players.len() {
                    return Err(format!("Player {} does not exist", pid));
                }
                // CR 800.4a, the same sentence `validate_player_target` reads:
                // a player who has left the game is not a player. **Not made
                // redundant by the enumeration filtering the same seat**, which
                // is why it went missing here for as long as it did: CR 608.2b
                // re-asks at resolution, and a seat can leave after it was
                // legally chosen.
                if !self.in_game(*pid) {
                    return Err(format!("Player {} has left the game", pid));
                }
                Ok(())
            }
            ResolvedTarget::Object(id) => {
                self.require_on_battlefield(*id)?;
                self.get_object(*id)?;
                if has_type(self, *id, CardType::Creature)
                    || has_type(self, *id, CardType::Planeswalker)
                {
                    Ok(())
                } else {
                    Err(format!(
                        "Target {} is not a creature or planeswalker", id
                    ))
                }
            }
        }
    }

    /// Validate a target is a permanent on the battlefield matching the filter.
    fn validate_permanent_target(
        &self,
        target: &ResolvedTarget,
        filter: &ObjectFilter,
        you: PlayerId,
        earlier_targets: EarlierTargets<'_>,
    ) -> Result<(), String> {
        match target {
            ResolvedTarget::Object(id) => {
                self.require_on_battlefield(*id)?;
                if !self.object_matches_filter_for_instance(*id, filter, you, earlier_targets)? {
                    return Err(format!(
                        "Target {} does not match permanent filter {:?}", id, filter
                    ));
                }
                Ok(())
            }
            ResolvedTarget::Player(_) => {
                Err("Expected a permanent target, got a player".to_string())
            }
        }
    }

    /// Validate a target is a spell on the stack.
    ///
    /// CR 112.1's "a spell is a card on the stack", asked of the `StackEntry`:
    /// membership alone let "counter target spell" name an activated ability's
    /// object, and CR 701.6a's move to the graveyard then manufactured a second
    /// copy of the ability's source card. The complement — "counter target
    /// activated or triggered ability", which `Primitive::CounterAbility`
    /// already resolves — is a filter of its own and negates this predicate; no
    /// registered card asks for it yet.
    fn validate_spell_target(&self, target: &ResolvedTarget) -> Result<(), String> {
        match target {
            ResolvedTarget::Object(id) => {
                if !self.is_spell_on_stack(*id) {
                    return Err(format!(
                        "Target {} is not a spell on the stack (CR 112.1)",
                        id
                    ));
                }
                Ok(())
            }
            ResolvedTarget::Player(_) => {
                Err("Expected a spell target, got a player".to_string())
            }
        }
    }

    /// Check whether an object is on the battlefield.
    fn require_on_battlefield(&self, id: ObjectId) -> Result<(), String> {
        if !self.battlefield.contains_key(&id) {
            return Err(format!("Object {} is not on the battlefield", id));
        }
        Ok(())
    }

    /// Check whether a permanent matches an `ObjectFilter`.
    ///
    /// Same question as `compute::object_matches_filter`, asked from outside
    /// the layer walk: this one queries each characteristic by id (so every
    /// read is still post-layers), while that one is handed a partially-applied
    /// frame it must not re-enter. Both must agree — "Enchant creature you
    /// control" has to mean the same thing to SBA 704.5n that it would mean to
    /// a static ability.
    ///
    /// A third caller: `ObjectSet::Filter` on a `ReplacementDef`
    /// asks it too, which is what makes Kalitas's "a nontoken creature an
    /// opponent controls" the same predicate a targeting restriction would
    /// use.
    ///
    /// `you` resolves `PlayerRef::You`; `Owner` is the *selected* permanent's
    /// owner, since a selection filter describes the selection rather than the
    /// source.
    pub(crate) fn object_matches_filter(
        &self,
        id: ObjectId,
        filter: &ObjectFilter,
        you: PlayerId,
    ) -> Result<bool, String> {
        self.get_object(id)?;
        // One layer walk for the whole filter, and only if a leaf reads a
        // characteristic: `All`, `Token` and `ByOwner` never do, so Rest in
        // Peace's "cards" stays free on every graveyard-bound zone change.
        let frame: std::cell::OnceCell<Option<std::sync::Arc<EffectiveCharacteristics>>> =
            std::cell::OnceCell::new();
        self.object_matches_filter_with(id, filter, you, FilterIdentity::NONE, &|| {
            frame
                .get_or_init(|| compute_characteristics(self, id))
                .as_deref()
                .ok_or_else(|| format!("Object {} not found", id))
        })
    }

    /// [`Self::object_matches_filter`] asked inside CR 601.2c's loop, where the
    /// instances announced so far are known — so
    /// [`ObjectFilter::OtherThanInstance`] has something to be other than.
    ///
    /// The selection-side twin of [`Self::object_matches_filter_of_source`],
    /// and the two identity facts are deliberately separate: "each other" is
    /// about the *effect's source* and "another target" is about an *earlier
    /// choice*, and a filter can carry both.
    pub(crate) fn object_matches_filter_for_instance(
        &self,
        id: ObjectId,
        filter: &ObjectFilter,
        you: PlayerId,
        earlier_targets: EarlierTargets<'_>,
    ) -> Result<bool, String> {
        self.get_object(id)?;
        let frame: std::cell::OnceCell<Option<std::sync::Arc<EffectiveCharacteristics>>> =
            std::cell::OnceCell::new();
        let identity = FilterIdentity { source: None, earlier_targets };
        self.object_matches_filter_with(id, filter, you, identity, &|| {
            frame
                .get_or_init(|| compute_characteristics(self, id))
                .as_deref()
                .ok_or_else(|| format!("Object {} not found", id))
        })
    }

    /// [`Self::object_matches_filter`] asked on behalf of an **effect**, which
    /// has a source — so [`ObjectFilter::NotSource`] has one to exclude.
    ///
    /// The one caller is `replacement::gather::set_affects`, which is the one
    /// place a filter is asked "is this object inside this effect's affected
    /// set" rather than "is this object a legal selection". Palisade Giant's
    /// "other permanents you control" is the printed customer, and
    /// `compute::object_matches_filter` has answered the same leaf off
    /// `FilterPlayers::source` since the layer system.
    ///
    /// `chars` is CR 614.12's look-ahead frame when the caller holds one, and
    /// `None` when the object is a plain permanent — the two spellings of one
    /// question, kept as one function because `set_affects` chooses between
    /// them per call.
    pub(crate) fn object_matches_filter_of_source(
        &self,
        id: ObjectId,
        filter: &ObjectFilter,
        you: PlayerId,
        source: ObjectId,
        chars: Option<&EffectiveCharacteristics>,
    ) -> Result<bool, String> {
        let frame: std::cell::OnceCell<Option<std::sync::Arc<EffectiveCharacteristics>>> =
            std::cell::OnceCell::new();
        let identity = FilterIdentity { source: Some(source), earlier_targets: EarlierTargets::None };
        self.object_matches_filter_with(id, filter, you, identity, &|| match chars {
            Some(chars) => Ok(chars),
            None => frame
                .get_or_init(|| compute_characteristics(self, id))
                .as_deref()
                .ok_or_else(|| format!("Object {} not found", id)),
        })
    }

    /// [`Self::object_matches_filter`] against a frame the caller already
    /// holds — CR 614.12's look-ahead for an entering permanent
    /// (`engine::replacement::EntryFrame`), where the finished board would
    /// answer for the card and not for the permanent it is about to become.
    pub(crate) fn object_matches_filter_in_frame(
        &self,
        id: ObjectId,
        filter: &ObjectFilter,
        you: PlayerId,
        chars: &EffectiveCharacteristics,
    ) -> Result<bool, String> {
        self.object_matches_filter_with(id, filter, you, FilterIdentity::NONE, &|| Ok(chars))
    }

    /// The leaf table, over a frame supplied on demand.
    ///
    /// The second of the two `object_matches_filter`s in the engine
    /// (`codebase-state.md` item 14): `compute::object_matches_filter` asks
    /// whether a continuous effect applies mid-layer-walk and resolves
    /// `PlayerRef` through the effect's source; this one asks whether an
    /// object is a legal selection, or inside a replacement's or restriction's
    /// `ObjectSet`, and resolves it against `you`.
    ///
    /// `other_than` is the source [`ObjectFilter::NotSource`] excludes — `None`
    /// in a selection context, where there is no such object and the leaf is
    /// refused rather than answered.
    fn object_matches_filter_with<'f>(
        &self,
        id: ObjectId,
        filter: &ObjectFilter,
        you: PlayerId,
        identity: FilterIdentity<'_>,
        frame: &dyn Fn() -> Result<&'f EffectiveCharacteristics, String>,
    ) -> Result<bool, String> {
        let obj = self.get_object(id)?;
        match filter {
            ObjectFilter::All => Ok(true),
            ObjectFilter::ByType(card_type) => Ok(frame()?.types.contains(card_type)),
            ObjectFilter::BySubtype(subtype) => Ok(frame()?.subtypes.contains(subtype)),
            ObjectFilter::BySupertype(supertype) => {
                Ok(frame()?.supertypes.contains(supertype))
            }
            ObjectFilter::ByColor(color) => Ok(frame()?.colors.contains(color)),
            ObjectFilter::ByController(player_ref) => {
                use crate::types::effects::PlayerRef;
                let controller = frame()?.controller;
                Ok(match player_ref {
                    PlayerRef::You => controller == you,
                    PlayerRef::Opponent => controller != you,
                    PlayerRef::Player(pid) => controller == *pid,
                    PlayerRef::Owner => controller == obj.owner,
                })
            }
            // CR 111.1 / 707.2 — being a token is a property of the *object*,
            // not of its characteristics, so it is read off `GameObject` rather
            // than off the layer frame. A copy effect does not make a nontoken
            // permanent a token (CR 707.2: copiable values do not include it).
            ObjectFilter::Token => Ok(obj.is_token),
            // CR 108.3 / 400.3 — ownership, not control. See the variant's doc:
            // the two diverge the moment control moves, and a card always goes
            // to its *owner's* graveyard.
            ObjectFilter::ByOwner(player_ref) => {
                use crate::types::effects::PlayerRef;
                Ok(match player_ref {
                    PlayerRef::You => obj.owner == you,
                    PlayerRef::Opponent => obj.owner != you,
                    // Tautological here, and forced by the signature: this function takes
                    // `you` and no source id, so `Owner` can only mean the tested object's own
                    // owner, which the `ByController` arm above already reads it as.
                    // `compute.rs` resolves the same `PlayerRef::Owner` against the *source's*
                    // owner, because a `FilterPlayers` has the source; the disagreement is
                    // recorded in `codebase-state.md`, since no card reads either spelling yet.
                    PlayerRef::Owner => true,
                    PlayerRef::Player(pid) => obj.owner == *pid,
                })
            }
            // "Each other" is relative to an effect's source. An *affected set*
            // has one — Palisade Giant's "other permanents you control" — and a
            // *selection* does not, so the second is refused rather than
            // answered `true`: a filter that silently included the source would
            // be the opposite of the word.
            //
            // Answered off the ids, like `compute::object_matches_filter`'s
            // identical arm: no layer can make an object something other than
            // itself, so this needs no frame.
            ObjectFilter::NotSource => match identity.source {
                Some(source) => Ok(id != source),
                None => Err(format!(
                    "ObjectFilter::NotSource on {} has no source to be other than in a selection context",
                    id
                )),
            },
            // CR 601.2c's "another target": answered off the ids, like
            // `NotSource` above and for the same reason.
            //
            // **An instance that announced nothing excludes nothing**, which
            // is not a silent default: the only way to reach this leaf with an
            // empty instance is a `TargetCount::UpTo` clause the player took
            // zero targets for (CR 115.6), and "another creature" than no
            // creature is every creature. An instance the walk never reached
            // is the refused case below.
            ObjectFilter::OtherThanInstance(ix) => {
                if *ix >= identity.earlier_targets.len() {
                    return Err(format!(
                        "ObjectFilter::OtherThanInstance({ix}) on {id} names an instance of \
                         \"target\" that has not been announced (CR 601.2c). Only the \
                         announcement loop and the CR 608.2b re-check hold the earlier_targets \
                         instances; a filter asked anywhere else cannot carry this leaf."
                    ));
                }
                Ok(!identity
                    .earlier_targets
                    .instance(*ix)
                    .contains(&ResolvedTarget::Object(id)))
            }
            ObjectFilter::PowerLE(max_power) => frame()?
                .power
                .map(|p| p <= *max_power)
                .ok_or_else(|| format!("Object {} has no power", id)),
            ObjectFilter::And(a, b) => {
                let matches_a = self.object_matches_filter_with(id, a, you, identity, frame)?;
                let matches_b = self.object_matches_filter_with(id, b, you, identity, frame)?;
                Ok(matches_a && matches_b)
            }
            // Short-circuits, where `And` above does not, and the asymmetry is
            // deliberate: a leaf can answer `Err` rather than `false` (`PowerLE`
            // on something with no power), `set_affects` collapses `Err` to
            // `false`, and a matched left arm makes the right arm's answer
            // irrelevant. Evaluating it anyway would turn a true `Or` into a
            // silent `false`.
            ObjectFilter::Or(a, b) => {
                if self.object_matches_filter_with(id, a, you, identity, frame)? {
                    return Ok(true);
                }
                self.object_matches_filter_with(id, b, you, identity, frame)
            }
            ObjectFilter::Not(inner) => {
                let matches = self.object_matches_filter_with(id, inner, you, identity, frame)?;
                Ok(!matches)
            }
        }
    }

    /// CR 608.2b, asked of every instance at once as the spell or ability
    /// begins to resolve.
    ///
    /// Returns the survivors — each instance filtered to the targets still
    /// legal **now** — or `None` when the spell does not resolve at all.
    ///
    /// **Two rules, and they are not the same rule.** "If all its targets …
    /// are now illegal, the spell doesn't resolve" is asked across every
    /// instance together; "if *some* are illegal, it resolves but does nothing
    /// to them" is asked per target. The one-recipient model could only ask the
    /// first, which is why Plague Spores' land half used to die with its
    /// creature half and Jagged Lightning used to damage a creature that had
    /// gained protection (`backlog.md` §2.20).
    ///
    /// **Filtered once, here, not per atom.** CR 608.2b checks targets as the
    /// spell *begins* to resolve, so an earlier atom that changes the board
    /// does not make a later atom's target illegal — Plague Spores destroying
    /// the creature does not un-target the land.
    ///
    /// A non-targeting `Choose` instance is kept whole: CR 115.1's targeting
    /// rules are what 608.2b is about, and a choice does not fizzle.
    pub fn surviving_targets(
        &self,
        instances: &[TargetInstance],
        you: PlayerId,
    ) -> Option<ChosenTargets> {
        // The announcement, unfiltered, so that `OtherThanInstance` re-reads
        // what CR 601.2c chose. "Another target creature" was a criterion of
        // the *announcement*; the objects have not changed, so it stays
        // satisfied, and re-asking it against a half-filtered list would make
        // one instance's fizzle silently legalize another's.
        //
        // Borrowed from the entry rather than copied: the announcement is
        // already `instances`, and the only thing the leaf needs of it is a
        // by-index read.
        let announced_targets = EarlierTargets::Announced(instances);

        // CR 115.6 — "a spell or ability that requires targets may allow zero
        // targets to be chosen … that spell or ability is targeted only if one
        // or more targets have been chosen for it." An `UpTo` clause the player
        // took nothing for leaves nothing for 608.2b to find illegal, so it
        // cannot be what makes the spell fail to resolve.
        let mut announced = false;
        let mut survived = false;
        let mut survivors = ChosenTargets::NONE;
        for inst in instances {
            if !inst.is_targeted() {
                // A `Choose` does not fizzle and is not re-checked, so it is
                // carried through whole.
                survivors.push(inst.chosen.iter().copied());
                continue;
            }
            announced |= !inst.chosen.is_empty();
            let before = survivors.all().len();
            survivors.push(inst.chosen.iter().copied().filter(|t| {
                self.is_single_target_legal(&inst.recipient, t, you, announced_targets)
            }));
            survived |= survivors.all().len() > before;
        }

        if announced && !survived {
            return None;
        }
        Some(survivors)
    }

    /// Whether the battlefield (or the player list, or the stack) holds `n`
    /// legal choices for one instance of "target".
    ///
    /// **`exclude_id` is CR 115.5** — "a spell or ability on the stack is an
    /// illegal target for itself" — and every caller inside CR 601.2c's loop
    /// passes the object being cast or activated. The older comment here called
    /// it "the Aura, which can't enchant itself"; that case cannot arise, since
    /// an Aura spell is on the *stack* when its target is chosen and an enchant
    /// filter only matches permanents. Where the parameter actually bites is
    /// the stack-reading filters, `Spell` and `DamageSource`.
    ///
    /// For player filters, all players are considered (player hexproof and
    /// shroud are `backlog.md` §2.15's).
    ///
    /// **The battlefield scans are ordered, and the reason is cost rather than
    /// answer.** Every candidate it tests is a `validate_selection`, which is a
    /// layer walk, and a short circuit over a `HashMap` stops after a different
    /// number of them in every process; `state/diagnostics.rs` records layer
    /// walks as a fixture, so the count is observable and the order has to be
    /// too. The sort is on a hot path — `mana_helpers` asks this per castable
    /// spell per priority check — and measured (2026-09-01) below the noise
    /// floor against the walks it bounds.
    /// `n` is how many **distinct** choices one instance needs — Jagged
    /// Lightning's "each of two target creatures" is not castable into a board
    /// with one creature, because 601.2c's first sentence forbids choosing it
    /// twice. `earlier_targets` is the instances already announced, which is what makes
    /// Incremental Growth's third clause need a third creature rather than the
    /// same one again.
    ///
    /// **Greedy is exact for the shapes that print.** An "another target" chain
    /// reuses one filter, so counting candidates that pass it and subtracting
    /// the ones already taken is the same answer a matching would give. A card
    /// whose instances carry *different* filters and also exclude each other
    /// would need the matching, and none prints — `codebase-state.md` carries
    /// the item with that reachability line.
    pub(crate) fn has_legal_choices(
        &self,
        filter: &SelectionFilter,
        exclude_id: Option<ObjectId>,
        you: PlayerId,
        n: usize,
        earlier_targets: EarlierTargets<'_>,
    ) -> bool {
        if n == 0 {
            return true;
        }
        // CR 800.4a — the seats that are still players. `players.len()` is the
        // seat count the game *began* with, which the rule never shrinks, so a
        // departed seat would otherwise be counted toward `n` by both arms that
        // read it. A closure rather than a `let`, so the battlefield filters
        // below do not pay for it on `mana_helpers`' hot path.
        let seats_in_game = || (0..self.players.len()).filter(|&p| self.in_game(p)).count();
        match filter {
            SelectionFilter::Player => {
                // Player hexproof and shroud (Leyline of Sanctity's class) are
                // `backlog.md` §2.15's; until then every player still in the
                // game is a legal choice.
                seats_in_game() >= n
            }
            SelectionFilter::Any => {
                // "Any target" = creature or planeswalker on battlefield, OR player
                let mut found = seats_in_game();
                if found >= n {
                    return true;
                }
                for id in self.battlefield_ids_ordered() {
                    if Some(id) == exclude_id {
                        continue;
                    }
                    let candidate = ResolvedTarget::Object(id);
                    if self.validate_selection(filter, &candidate, you, earlier_targets).is_ok() {
                        found += 1;
                        if found >= n {
                            return true;
                        }
                    }
                }
                false
            }
            SelectionFilter::Spell => {
                // Spells live on the stack, not the battlefield — and not
                // every object there is one (CR 112.1), which is what
                // `is_spell_on_stack` asks. The sibling arm below asks it too.
                self.stack
                    .iter()
                    .filter(|&&id| Some(id) != exclude_id && self.is_spell_on_stack(id))
                    .count()
                    >= n
            }
            // CR 609.7a's two reachable categories, in the order
            // `enumerate_legal_selections` offers them. Cheaper than the
            // `_` arm below and not the same answer: a source of damage
            // needs no `validate_selection` walk at all.
            SelectionFilter::DamageSource => {
                let permanents = self
                    .battlefield_ids_ordered()
                    .into_iter()
                    .filter(|&id| Some(id) != exclude_id)
                    .count();
                let spells = self
                    .stack
                    .iter()
                    .filter(|&&id| Some(id) != exclude_id && self.is_spell_on_stack(id))
                    .count();
                permanents + spells >= n
            }
            _ => {
                let mut found = 0usize;
                for id in self.battlefield_ids_ordered() {
                    if Some(id) == exclude_id {
                        continue;
                    }
                    let candidate = ResolvedTarget::Object(id);
                    if self.validate_selection(filter, &candidate, you, earlier_targets).is_ok() {
                        found += 1;
                        if found >= n {
                            return true;
                        }
                    }
                }
                false
            }
        }
    }

    /// Check if a single target is still legal for the given spec.
    /// Only meaningful for `Target` — `Choose` doesn't participate in fizzle.
    fn is_single_target_legal(
        &self,
        recipient: &EffectRecipient,
        target: &ResolvedTarget,
        you: PlayerId,
        earlier_targets: EarlierTargets<'_>,
    ) -> bool {
        match recipient {
            EffectRecipient::Target(filter, _) => {
                self.validate_selection(filter, target, you, earlier_targets).is_ok()
            }
            // Choose, Implicit, Controller — always "legal" (no fizzle).
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::PermanentState;
    use crate::types::card_types::{CardType, Supertype, Subtype, LandType};
    use crate::types::effects::{ObjectFilter, TargetCount};
    use crate::types::zones::Zone;

    fn setup_game_with_land() -> (GameState, ObjectId) {
        let mut game = GameState::new(2, 20);
        let land_data = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .subtype(Subtype::Land(LandType::Forest))
            .build();
        let obj = GameObject::new(land_data, 0, Zone::Battlefield);
        let id = game.add_object(obj);
        game.insert_battlefield_entity(id, PermanentState::new(id, 0, 1));
        (game, id)
    }

    #[test]
    fn test_validate_permanent_target_all() {
        let (game, land_id) = setup_game_with_land();
        let targets = vec![ResolvedTarget::Object(land_id)];
        let spec = EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::All), TargetCount::Exactly(1));
        assert!(game.validate_targets(&spec, &targets, 0, &ChosenTargets::NONE).is_ok());
    }

    #[test]
    fn test_validate_permanent_target_by_type_land() {
        let (game, land_id) = setup_game_with_land();
        let targets = vec![ResolvedTarget::Object(land_id)];
        let spec = EffectRecipient::Target(SelectionFilter::Permanent(
            ObjectFilter::ByType(CardType::Land)),
            TargetCount::Exactly(1),
        );
        assert!(game.validate_targets(&spec, &targets, 0, &ChosenTargets::NONE).is_ok());
    }

    #[test]
    fn test_validate_permanent_target_wrong_type() {
        let (game, land_id) = setup_game_with_land();
        let targets = vec![ResolvedTarget::Object(land_id)];
        let spec = EffectRecipient::Target(SelectionFilter::Permanent(
            ObjectFilter::ByType(CardType::Creature)),
            TargetCount::Exactly(1),
        );
        assert!(game.validate_targets(&spec, &targets, 0, &ChosenTargets::NONE).is_err());
    }

    #[test]
    fn test_validate_player_target() {
        let game = GameState::new(2, 20);
        let targets = vec![ResolvedTarget::Player(1)];
        let spec = EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1));
        assert!(game.validate_targets(&spec, &targets, 0, &ChosenTargets::NONE).is_ok());
    }

    #[test]
    fn test_validate_player_target_invalid() {
        let game = GameState::new(2, 20);
        let targets = vec![ResolvedTarget::Player(5)];
        let spec = EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1));
        assert!(game.validate_targets(&spec, &targets, 0, &ChosenTargets::NONE).is_err());
    }

    #[test]
    fn test_validate_spell_target_not_on_stack() {
        let game = GameState::new(2, 20);
        let fake_id = crate::types::ids::new_object_id();
        let targets = vec![ResolvedTarget::Object(fake_id)];
        let spec = EffectRecipient::Target(SelectionFilter::Spell, TargetCount::Exactly(1));
        assert!(game.validate_targets(&spec, &targets, 0, &ChosenTargets::NONE).is_err());
    }

    #[test]
    fn test_validate_no_targets() {
        let game = GameState::new(2, 20);
        let spec = EffectRecipient::Implicit;
        assert!(game.validate_targets(&spec, &[], 0, &ChosenTargets::NONE).is_ok());
        assert!(game.validate_targets(&spec, &[ResolvedTarget::Player(0)], 0, &ChosenTargets::NONE).is_err());
    }

    #[test]
    fn test_validate_wrong_target_count() {
        let (game, land_id) = setup_game_with_land();
        let targets = vec![
            ResolvedTarget::Object(land_id),
            ResolvedTarget::Object(land_id),
        ];
        let spec = EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::All), TargetCount::Exactly(1));
        assert!(game.validate_targets(&spec, &targets, 0, &ChosenTargets::NONE).is_err());
    }

    #[test]
    fn a_spell_whose_only_target_left_the_battlefield_does_not_resolve() {
        let (mut game, land_id) = setup_game_with_land();
        let spec = EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::All), TargetCount::Exactly(1));
        let instances = vec![TargetInstance::new(
            spec,
            vec![ResolvedTarget::Object(land_id)],
        )];

        // Target is legal while on battlefield
        let survivors = game
            .surviving_targets(&instances, 0)
            .expect("a legal target resolves");
        assert_eq!(survivors.instance(0), &[ResolvedTarget::Object(land_id)]);

        // Remove from battlefield — the one target is no longer legal, and
        // CR 608.2b's "all its targets" is therefore satisfied.
        game.battlefield.remove(&land_id);
        assert!(game.surviving_targets(&instances, 0).is_none());
    }
}

