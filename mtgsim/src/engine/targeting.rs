use crate::engine::layers::condition::ConditionReferents;
use crate::engine::resolve::ResolvedTarget;
use crate::engine::layers::compute::compute_characteristics;
use crate::engine::object_filter::{self, EarlierTargets, FilterReferents, TextReferents};
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::oracle::characteristics::{has_type};
use crate::state::game_state::GameState;
use crate::types::card_types::CardType;
use crate::types::effects::{Effect, ObjectFilter, EffectRecipient, SelectionFilter, TargetBounds, TargetCount};
use crate::oracle::characteristics::ObjectInformation;
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
    /// The objects and players chosen for it, in the order they were offered,
    /// each object as the existence it was chosen in.
    pub chosen: Vec<TargetRef>,
}

/// One object or player chosen for an instance of "target", remembered by
/// identity: an object by its id **and** the zone-change epoch it had when
/// it was chosen (CR 400.7), so a creature that left and came back is not
/// the one chosen.
///
/// CR 608.2b's "a target that's no longer in the zone it was in when it was
/// targeted is illegal" is the comparison: a bare id finds the new object
/// and calls it legal. Only the announcement holds these. The survivors CR
/// 608.2b keeps are plain [`ResolvedTarget`]s, legal as the resolution
/// begins, which is all a primitive reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetRef {
    Object(crate::types::ids::ObjectRef),
    Player(PlayerId),
}

impl TargetRef {
    /// What the object or player is now, which is all a reader past the
    /// existence check needs.
    pub fn as_resolved_target(self) -> ResolvedTarget {
        match self {
            TargetRef::Object(object) => ResolvedTarget::Object(object.id),
            TargetRef::Player(player) => ResolvedTarget::Player(player),
        }
    }

    /// Is this still the object that was chosen: in the store, at the epoch
    /// it was chosen at (CR 400.7)? A player is always that player; CR
    /// 800.4a's departure is the target check's question, not this one's.
    pub fn still_exists(self, game: &GameState) -> bool {
        match self {
            TargetRef::Object(object) => game.find(object).is_some(),
            TargetRef::Player(_) => true,
        }
    }
}

impl TargetInstance {
    pub fn new(recipient: EffectRecipient, chosen: Vec<TargetRef>) -> Self {
        TargetInstance { recipient, chosen }
    }

    /// Record `chosen` as CR 601.2c announced it, each object at the epoch
    /// it has now. `Err` for an object that is not in the store, which a
    /// choice made from the legal candidates cannot name.
    pub fn from_announcement(game: &GameState, recipient: EffectRecipient, chosen: &[ResolvedTarget]) -> Result<Self, String> {
        let chosen = chosen
            .iter()
            .map(|&target| game.target_ref(target).ok_or_else(|| format!("{target:?} was announced but is not an object")))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(TargetInstance { recipient, chosen })
    }

    /// What was chosen for this instance, as it is now.
    pub fn as_resolved_targets(&self) -> impl Iterator<Item = ResolvedTarget> + '_ {
        self.chosen.iter().map(|chosen| chosen.as_resolved_target())
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

/// Check that the number of targets is within the bounds `count` announced.
fn validate_target_count(count: &TargetCount, bounds: TargetBounds, actual: usize) -> Result<(), String> {
    if actual < bounds.fewest || actual > bounds.most {
        return Err(format!("{count:?} allows {} to {} target(s), got {actual}", bounds.fewest, bounds.most));
    }
    Ok(())
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

/// The two facts a filter leaf may need that are **not characteristics**, so
/// that no layer can change them and no frame answers them.
///
/// Both are identity questions and both were already in the tree as one
/// `Option<ObjectId>` parameter; `earlier_targets` is the second, added for CR 601.2c's
/// "another target". A filter may carry both — "another target creature you
/// control other than this one" is a legal English sentence — so they are
/// fields rather than an enum.
///
/// **`exclude_id` is not a third field**, though it is the same kind of fact:
/// CR 115.5 bites on the `Spell` and `DamageSource` arms, which answer
/// membership without walking an `ObjectFilter`, so it stays in the enumeration
/// whatever the filter's shape.
#[derive(Clone, Copy)]
pub struct FilterIdentity<'a> {
    /// The source [`ObjectFilter::NotSource`] excludes: the effect's own, or
    /// for a selection the object "this" names in the text being announced —
    /// the spell, or the permanent whose ability it is (CR 113.7a), so
    /// "another target creature" is other than Cryptoplasm. `None` where no
    /// text is being announced, and the leaf is refused rather than answered.
    pub source: Option<ObjectId>,
    /// The instances of "target" announced before this one, which
    /// [`ObjectFilter::OtherThanInstance`] reads. `None` outside CR 601.2c's
    /// loop and CR 608.2b's re-check, where that leaf is likewise refused.
    pub earlier_targets: EarlierTargets<'a>,
    /// The other objects the text names, as CR 608.2h reads them: "this" once
    /// it has left, and a trigger's triggering event. A target's "defending
    /// player" reads its creature here. Nothing outside a trigger's placement
    /// and resolution.
    pub named_objects: ConditionReferents<'a>,
}

impl FilterIdentity<'static> {
    /// Neither fact is available — a plain selection or a look-ahead frame.
    pub const NONE: FilterIdentity<'static> = FilterIdentity {
        source: None,
        earlier_targets: EarlierTargets::None,
        named_objects: ConditionReferents::NONE,
    };
}

impl<'a> FilterIdentity<'a> {
    /// The text of `this_object` being announced, with the instances of
    /// "target" it has announced so far.
    pub fn for_text_of(this_object: ObjectId, earlier_targets: EarlierTargets<'a>) -> Self {
        FilterIdentity { source: Some(this_object), earlier_targets, named_objects: ConditionReferents::NONE }
    }

    /// The same, with the other objects the text names.
    pub fn naming(self, named_objects: ConditionReferents<'a>) -> Self {
        FilterIdentity { named_objects, ..self }
    }
}

/// Where a resolution reads CR 601.2c's clauses from, for an atom that refers
/// back to one (`EffectRecipient::SameInstanceAs`).
///
/// **The stack reads the announcement.** `StackEntry::chosen_targets` recorded
/// each clause beside its choice, so the index the announcement filled is the
/// index the resolution reads, and nothing is derived from the effect tree at
/// resolution. An Aura's instance comes from its enchant ability and sits in
/// no tree (CR 303.4a), which is why the announcement is the one list that is
/// right for every spell.
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
    /// `target` remembered by identity: [`Self::object_ref`]'s answer for an
    /// object, and `None` for one not in the store.
    pub fn target_ref(&self, target: ResolvedTarget) -> Option<TargetRef> {
        match target {
            ResolvedTarget::Object(id) => self.object_ref(id).map(TargetRef::Object),
            ResolvedTarget::Player(player) => Some(TargetRef::Player(player)),
        }
    }

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
    ///
    /// `bounds` are the count of targets the announcement read (CR 601.2c),
    /// since a count the text defines has no value outside it.
    pub fn validate_targets(
        &self,
        recipient: &EffectRecipient,
        targets: &[ResolvedTarget],
        bounds: TargetBounds,
        you: PlayerId,
        identity: FilterIdentity<'_>,
    ) -> Result<(), String> {
        match recipient {
            EffectRecipient::Implicit
            | EffectRecipient::ThisObject
            | EffectRecipient::EachOf(_)
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
            EffectRecipient::Triggering(_)
            | EffectRecipient::TriggeringPlayer
            | EffectRecipient::Referred => Err(format!(
                "{recipient:?} is a triggered ability's bound fact, not an instance of \"target\" to validate"
            )),
            // What a creature attacks, and whom, is read as the instruction
            // applies, and so is what a join's parts find.
            EffectRecipient::AttackedBy(_) | EffectRecipient::DefendingPlayer(_) | EffectRecipient::Group(_) => Err(format!(
                "{recipient:?} is read as the effect applies, not an instance of \"target\" to validate"
            )),
            // A choice is made as the effect applies (CR 608.2d); only its
            // chooser can be an instance, and `Effect::instances` lists that.
            EffectRecipient::ChosenBy(_) => Err(format!(
                "{recipient:?} is chosen as the effect applies, not an instance of \"target\" to validate"
            )),

            EffectRecipient::SameInstanceAs(ix) => Err(format!(
                "EffectRecipient::SameInstanceAs({ix}) is a back-reference to an instance of \
                 \"target\", not a clause to validate against (CR 115.3). Validate the \
                 clause `Effect::instances` lists at that index."
            )),

            // Target and Choose validate identically: hexproof, shroud and
            // protection are not checked for `Target` — `codebase-state.md`,
            // "Before card breadth" item 7, which is `cant-be-targeted`'s.
            EffectRecipient::Target(filter, count)
            | EffectRecipient::Choose(filter, count) => {
                validate_target_count(count, bounds, targets.len())?;
                // CR 601.2c's first sentence: one instance never picks a target twice
                // (Victimize's two cards; Decimate's four instances may share one).
                // `validate_pick_n` refuses a duplicate index already, but that is the
                // provider's contract, and CR 115.7's target changes check the rule.
                for (i, t) in targets.iter().enumerate() {
                    if targets[..i].contains(t) {
                        return Err(format!(
                            "{:?} was chosen twice for one instance of \"target\" (CR 601.2c)",
                            t
                        ));
                    }
                }
                for t in targets {
                    self.validate_selection(filter, t, you, identity)?;
                }
                Ok(())
            }
        }
    }


    /// Whether Aura `aura`, whose enchant ability is `filter`, may enchant
    /// `host` (CR 303.4a), its "you" being `you` (CR 109.5): what CR 704.5m
    /// leaves attached and what CR 303.4f offers an Aura entering the
    /// battlefield. Not targeting, so hexproof and shroud do not apply. Asked
    /// outside CR 601.2c's loop, so an `OtherThanInstance` leaf is refused
    /// rather than answered; "this" is the Aura.
    pub(crate) fn can_enchant(&self, filter: &SelectionFilter, aura: ObjectId, host: ObjectId, you: PlayerId) -> bool {
        let identity = FilterIdentity::for_text_of(aura, EarlierTargets::None);
        self.validate_selection(filter, &ResolvedTarget::Object(host), you, identity).is_ok()
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
        identity: FilterIdentity<'_>,
    ) -> Result<(), String> {
        match filter {
            SelectionFilter::Creature => self.validate_creature_target(target),
            SelectionFilter::Player => self.validate_player_target(target),
            SelectionFilter::Any => self.validate_any_target(target),
            SelectionFilter::Permanent(pf) => {
                self.validate_permanent_target(target, pf, you, identity)
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
    /// The stack half asks `is_spell_on_stack`, because CR 609.7a lists a
    /// *spell* and an activated ability on the stack is not one; its source is
    /// the permanent, which the battlefield half already offers.
    fn validate_damage_source(&self, target: &ResolvedTarget) -> Result<(), String> {
        match target {
            ResolvedTarget::Object(id) => {
                if crate::oracle::legality::damage_sources(self, None).iter().any(|source| source.id == *id) {
                    return Ok(());
                }
                Err(format!(
                    "Object {} is neither a permanent, a spell on the stack, nor an object \
                     something waiting refers to, so it is not a legal source of damage (CR 609.7a)",
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
        identity: FilterIdentity<'_>,
    ) -> Result<(), String> {
        match target {
            ResolvedTarget::Object(id) => {
                self.require_on_battlefield(*id)?;
                if !self.object_matches_filter_for_instance(*id, filter, you, identity) {
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

    /// `filter` asked of `object` where the text's words mean `referents` —
    /// the one table every asker reaches (`engine::object_filter`).
    ///
    /// Read by its last known information or the characteristics its caller
    /// holds where `object` carries them ([`ObjectInformation`]). Read
    /// current, the object must exist, and its frame is walked once, only if
    /// a leaf reads a characteristic: `All`, `Token` and `ByOwner` never do,
    /// so Rest in Peace's "cards" stays free on every graveyard-bound zone
    /// change. A gone object matches nothing its last known information does
    /// not answer for.
    pub(crate) fn object_matches_filter(
        &self,
        object: ObjectInformation<'_>,
        filter: &ObjectFilter,
        referents: &mut impl FilterReferents,
    ) -> bool {
        let id = match object {
            ObjectInformation::Characteristics(id, chars) => {
                return object_filter::evaluate(self, filter, id, &|| Some(chars), None, referents);
            }
            ObjectInformation::LastKnown(lki) => {
                return object_filter::evaluate(self, filter, lki.object().id, &|| Some(lki.characteristics()), Some(lki), referents);
            }
            ObjectInformation::Current(id) => id,
        };
        if !self.objects.contains_key(&id) {
            return false;
        }
        let frame: std::cell::OnceCell<Option<std::sync::Arc<EffectiveCharacteristics>>> =
            std::cell::OnceCell::new();
        let frame = || frame.get_or_init(|| compute_characteristics(self, id)).as_deref();
        object_filter::evaluate(self, filter, id, &frame, None, referents)
    }

    /// [`Self::object_matches_filter`] asked inside CR 601.2c's loop, where
    /// the instances announced so far are known — so
    /// [`ObjectFilter::OtherThanInstance`] has something to be other than —
    /// and the object whose text is announced, which
    /// [`ObjectFilter::NotSource`] is other than and whose owner `Owner` names.
    /// A resolution asks it with its effect's source and no instances
    /// (`codebase-state.md` item 229).
    ///
    /// The selection-side twin of [`Self::object_matches_filter_of_source`],
    /// and the two identity facts are deliberately separate: "another target
    /// creature" on Cryptoplasm is about the *ability's source* and on
    /// Incremental Growth about an *earlier choice*, and a filter can carry
    /// both.
    pub(crate) fn object_matches_filter_for_instance(
        &self,
        id: ObjectId,
        filter: &ObjectFilter,
        you: PlayerId,
        identity: FilterIdentity<'_>,
    ) -> bool {
        self.object_matches_filter(ObjectInformation::Current(id), filter, &mut TextReferents { game: self, you, identity })
    }

    /// [`Self::object_matches_filter`] asked for the text of `source` (CR
    /// 113.7a's "this"), with no instance of "target" announced — so
    /// [`ObjectFilter::NotSource`] has one to exclude and `Owner` one to name:
    /// a replacement's or restriction's affected set and its pattern's filters,
    /// an auxiliary move's, a cost modification's, a resolving effect's
    /// (`codebase-state.md` item 229), and a cost's own: "Sacrifice another
    /// creature:" prints on 162 cards (Scryfall, 2026-10-10). Palisade Giant's
    /// "other permanents you control" is the printed customer for an effect.
    ///
    /// `object` is read as the caller holds it, as
    /// [`Self::object_matches_filter`] takes it.
    pub(crate) fn object_matches_filter_of_source(
        &self,
        object: ObjectInformation<'_>,
        filter: &ObjectFilter,
        you: PlayerId,
        source: ObjectId,
    ) -> bool {
        self.object_matches_filter(object, filter, &mut TextReferents::for_effect(self, you, source))
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
    /// to them" is asked per target, so Plague Spores' land half survives its
    /// creature half (`backlog.md` §2.20).
    ///
    /// **Filtered once, here, not per atom.** CR 608.2b checks targets as the
    /// spell *begins* to resolve, so an earlier atom that changes the board
    /// does not make a later atom's target illegal — Plague Spores destroying
    /// the creature does not un-target the land.
    ///
    /// A non-targeting `Choose` instance keeps every choice that still
    /// exists: CR 115.1's targeting rules are what 608.2b is about, and a
    /// choice does not fizzle. One that left and came back is a new object
    /// with no relation to the chosen one (CR 400.7), so the effect cannot
    /// find it, which is the same for a target and a choice.
    ///
    /// A target that left and came back is illegal by 608.2b's own sentence:
    /// it is "no longer in the zone it was in when it was targeted", which
    /// its epoch says and its id cannot.
    pub fn surviving_targets(
        &self,
        instances: &[TargetInstance],
        you: PlayerId,
        this_object: ObjectId,
        named_objects: ConditionReferents<'_>,
    ) -> Option<ChosenTargets> {
        // The announcement, unfiltered: "another target" was a criterion of
        // CR 601.2c's announcement, and re-asking it against a half-filtered list
        // would let one instance's fizzle legalize another's. Borrowed, not copied.
        let announced_targets =
            FilterIdentity::for_text_of(this_object, EarlierTargets::Announced(instances)).naming(named_objects);

        // CR 115.6 — "a spell or ability that requires targets may allow zero
        // targets to be chosen … that spell or ability is targeted only if one
        // or more targets have been chosen for it." An `UpTo` clause the player
        // took nothing for leaves nothing for 608.2b to find illegal, so it
        // cannot be what makes the spell fail to resolve.
        let mut announced = false;
        let mut survived = false;
        let mut survivors = ChosenTargets::NONE;
        for inst in instances {
            let existing = inst.chosen.iter().filter(|chosen| chosen.still_exists(self)).map(|chosen| chosen.as_resolved_target());
            if !inst.is_targeted() {
                // A `Choose` does not fizzle and is not re-checked against its
                // clause, so all of it that still exists is carried through.
                survivors.push(existing);
                continue;
            }
            announced |= !inst.chosen.is_empty();
            let before = survivors.all().len();
            survivors.push(existing.filter(|t| self.is_single_target_legal(&inst.recipient, t, you, announced_targets)));
            survived |= survivors.all().len() > before;
        }

        if announced && !survived {
            return None;
        }
        Some(survivors)
    }

    /// Check if a single target is still legal for the given spec.
    /// Only meaningful for `Target` — `Choose` doesn't participate in fizzle.
    fn is_single_target_legal(
        &self,
        recipient: &EffectRecipient,
        target: &ResolvedTarget,
        you: PlayerId,
        identity: FilterIdentity<'_>,
    ) -> bool {
        match recipient {
            EffectRecipient::Target(filter, _) => {
                self.validate_selection(filter, target, you, identity).is_ok()
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
        assert!(game.validate_targets(&spec, &targets, crate::test_support::fixed_target_bounds(&spec), 0, FilterIdentity::NONE).is_ok());
    }

    #[test]
    fn test_validate_permanent_target_by_type_land() {
        let (game, land_id) = setup_game_with_land();
        let targets = vec![ResolvedTarget::Object(land_id)];
        let spec = EffectRecipient::Target(SelectionFilter::Permanent(
            ObjectFilter::ByType(CardType::Land)),
            TargetCount::Exactly(1),
        );
        assert!(game.validate_targets(&spec, &targets, crate::test_support::fixed_target_bounds(&spec), 0, FilterIdentity::NONE).is_ok());
    }

    #[test]
    fn test_validate_permanent_target_wrong_type() {
        let (game, land_id) = setup_game_with_land();
        let targets = vec![ResolvedTarget::Object(land_id)];
        let spec = EffectRecipient::Target(SelectionFilter::Permanent(
            ObjectFilter::ByType(CardType::Creature)),
            TargetCount::Exactly(1),
        );
        assert!(game.validate_targets(&spec, &targets, crate::test_support::fixed_target_bounds(&spec), 0, FilterIdentity::NONE).is_err());
    }

    #[test]
    fn test_validate_player_target() {
        let game = GameState::new(2, 20);
        let targets = vec![ResolvedTarget::Player(1)];
        let spec = EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1));
        assert!(game.validate_targets(&spec, &targets, crate::test_support::fixed_target_bounds(&spec), 0, FilterIdentity::NONE).is_ok());
    }

    #[test]
    fn test_validate_player_target_invalid() {
        let game = GameState::new(2, 20);
        let targets = vec![ResolvedTarget::Player(5)];
        let spec = EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1));
        assert!(game.validate_targets(&spec, &targets, crate::test_support::fixed_target_bounds(&spec), 0, FilterIdentity::NONE).is_err());
    }

    #[test]
    fn test_validate_spell_target_not_on_stack() {
        let game = GameState::new(2, 20);
        let fake_id = crate::types::ids::new_object_id();
        let targets = vec![ResolvedTarget::Object(fake_id)];
        let spec = EffectRecipient::Target(SelectionFilter::Spell, TargetCount::Exactly(1));
        assert!(game.validate_targets(&spec, &targets, crate::test_support::fixed_target_bounds(&spec), 0, FilterIdentity::NONE).is_err());
    }

    #[test]
    fn test_validate_no_targets() {
        let game = GameState::new(2, 20);
        let spec = EffectRecipient::Implicit;
        assert!(game.validate_targets(&spec, &[], crate::test_support::fixed_target_bounds(&spec), 0, FilterIdentity::NONE).is_ok());
        assert!(game.validate_targets(&spec, &[ResolvedTarget::Player(0)], crate::test_support::fixed_target_bounds(&spec), 0, FilterIdentity::NONE).is_err());
    }

    #[test]
    fn test_validate_wrong_target_count() {
        let (game, land_id) = setup_game_with_land();
        let targets = vec![
            ResolvedTarget::Object(land_id),
            ResolvedTarget::Object(land_id),
        ];
        let spec = EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::All), TargetCount::Exactly(1));
        assert!(game.validate_targets(&spec, &targets, crate::test_support::fixed_target_bounds(&spec), 0, FilterIdentity::NONE).is_err());
    }

    #[test]
    fn a_spell_whose_only_target_left_the_battlefield_does_not_resolve() {
        let (mut game, land_id) = setup_game_with_land();
        let spec = EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::All), TargetCount::Exactly(1));
        let instances = vec![TargetInstance::from_announcement(&game, spec, &[ResolvedTarget::Object(land_id)]).unwrap()];

        // Target is legal while on battlefield
        let survivors = game
            .surviving_targets(&instances, 0, crate::types::ids::ObjectId::UNASSIGNED, ConditionReferents::NONE)
            .expect("a legal target resolves");
        assert_eq!(survivors.instance(0), &[ResolvedTarget::Object(land_id)]);

        // Remove from battlefield — the one target is no longer legal, and
        // CR 608.2b's "all its targets" is therefore satisfied.
        game.battlefield.remove(&land_id);
        assert!(game.surviving_targets(&instances, 0, crate::types::ids::ObjectId::UNASSIGNED, ConditionReferents::NONE).is_none());
    }
}

