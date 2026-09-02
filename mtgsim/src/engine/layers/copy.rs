//! CR 707.2 — copiable values: what a copy effect captures, and the one place
//! it is captured.
//!
//! > **613.2c** After all rules and effects in layer 1 have been applied, the
//! > object's characteristics are its copiable values. (See rule 707.2.)
//!
//! So copiable values are the *output* of layer 1, not an input to it, and the
//! capture is `compute_to_ceiling` at the ceiling just past layer 1. Every
//! producer in `copy-effects-architecture.md` §3.3 — a resolution, an entry
//! replacement, a token, a stack copy — captures here and differs only in what
//! carries the result.

use crate::engine::layers::compute::{compute_to_ceiling, FrameCache, LAYER_ORDER};
use crate::engine::layers::types::{EffectiveCharacteristics, Layer};
use crate::objects::card_data::AbilityDef;
use crate::state::game_state::GameState;
use crate::types::card_types::{CardType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::ids::ObjectId;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::ManaCost;

use std::collections::HashSet;

/// The frame-cache ceiling CR 613.2c names: every layer-1 sublayer applied and
/// nothing after it.
///
/// A named constant rather than a literal at each call site, because the
/// relationship it asserts is the one a later sublayer split breaks. CR 613.2
/// gives layer 1 two sublayers — 1a copy, 1b face-down — and `LAYER_ORDER`
/// collapses them into one slot today. CV-6 will split them; a capture that
/// still stopped at index 1 would then read the values *before* CR 708.2a
/// synthesized a face-down creature's 2/2, and the bug would appear only on
/// boards with a face-down creature being copied. The `debug_assert` in
/// `copiable_values` is what makes that a test failure instead.
pub const END_OF_LAYER_1: usize = 1;

/// CR 707.2 — the values a copy acquires, captured once (CR 707.2b/2c).
///
/// This is `EffectiveCharacteristics` as of the end of layer 1 (CR 613.2c),
/// minus the two fields CR 707.2 excludes because they are control rather than
/// characteristics. It is a **value**: nothing in it points at the object it
/// was captured from, which is what makes a copy row independent of every other
/// layer 1 effect under CR 613.8a(b) — and so what keeps copy work off
/// critical-path item 7 (`copy-effects-architecture.md` §5.2).
///
/// **Deliberately not `EffectiveCharacteristics` reused.** That type carries
/// `controller` and `control_since_turn`, and `compute.rs`'s
/// `any_control_changing` fast path proves its own correctness on the claim
/// that *Layer 2 is the only channel that writes a controller*. A copy row that
/// could carry one would make that claim false by accident. The duplicated
/// field list is the price of an existing optimization's soundness argument.
#[derive(Debug, Clone, PartialEq)]
pub struct CopiableValues {
    pub name: String,
    /// `Option`, matching `EffectiveCharacteristics`. CR 708.2a's face-down
    /// characteristics include *no* mana cost and CR 111.6 gives a token none
    /// either — both are capture subjects in later CV phases, so a bare
    /// `ManaCost` could not represent them.
    pub mana_cost: Option<ManaCost>,
    pub colors: HashSet<Color>,
    pub types: HashSet<CardType>,
    pub subtypes: HashSet<Subtype>,
    pub supertypes: HashSet<Supertype>,
    pub keyword_flags: HashSet<KeywordFlag>,
    /// CR 707.2a — a copy acquires abilities because they derive from rules
    /// text, not by copying "the abilities" as a separate characteristic.
    ///
    /// Carries each `AbilityDef.is_characteristic_defining` **verbatim**: CR
    /// 604.3a(2)'s third clause makes a CDA ride along on a copy, unlike a
    /// Layer 6 grant, which must clear it. CR 707.9d's drop for "except"
    /// clauses is the one case that has to remove a flag, and it arrives with
    /// the exception applier in CV-2.
    pub abilities: Vec<AbilityDef>,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
}

impl CopiableValues {
    /// Take the copiable half of a ceiling-1 frame.
    ///
    /// Consumes the frame so the capture is a move rather than a second deep
    /// clone of a `Vec<AbilityDef>`; the frame has no other reader.
    fn from_frame(frame: EffectiveCharacteristics) -> Self {
        // Destructured rather than field-by-field, so that adding a field to
        // `EffectiveCharacteristics` is a compile error here. Whether a new
        // characteristic is copiable is a CR 707.2 question and must be
        // answered, not defaulted.
        let EffectiveCharacteristics {
            name,
            mana_cost,
            colors,
            types,
            subtypes,
            supertypes,
            keyword_flags,
            abilities,
            power,
            toughness,
            controller: _,
            control_since_turn: _,
        } = frame;
        CopiableValues {
            name,
            mana_cost,
            colors,
            types,
            subtypes,
            supertypes,
            keyword_flags,
            abilities,
            power,
            toughness,
        }
    }

    /// Overwrite every characteristic channel in `chars` with these values.
    ///
    /// This is what layer 1a *is*: not an adjustment to a frame but a
    /// replacement of the values later layers then modify. `controller` and
    /// `control_since_turn` are untouched, which is the field list above
    /// restated as behaviour.
    pub(super) fn apply_to(&self, chars: &mut EffectiveCharacteristics) {
        chars.name = self.name.clone();
        chars.mana_cost = self.mana_cost.clone();
        chars.colors = self.colors.clone();
        chars.types = self.types.clone();
        chars.subtypes = self.subtypes.clone();
        chars.supertypes = self.supertypes.clone();
        chars.keyword_flags = self.keyword_flags.clone();
        chars.abilities = self.abilities.clone();
        chars.power = self.power;
        chars.toughness = self.toughness;
    }

    /// The static abilities in the captured list, for the CR 613.7a rows a copy
    /// owes (`copy-effects-architecture.md` §4.7 leg 2).
    ///
    /// CDAs are skipped for the reason `register_static_effects` skips them:
    /// CR 604.3a(3) makes a CDA apply to exactly the object that has it, so
    /// `layers::cda` applies it off the effective ability list and a row here
    /// would apply it twice.
    pub fn registrable_static_abilities(&self) -> impl Iterator<Item = &AbilityDef> {
        use crate::objects::card_data::AbilityType;
        self.abilities
            .iter()
            .filter(|a| a.ability_type == AbilityType::Static && !a.is_characteristic_defining)
    }
}

/// CR 707.2 — capture `id`'s copiable values.
///
/// The single capture point. `None` when `id` names no object.
///
/// **Reads `game.objects`, not `game.battlefield`**, so a card in a graveyard
/// has copiable values too (CR 707.2 is not a battlefield rule) — which is what
/// CV-1b's Dimir Doppelganger needs and costs nothing to allow now.
pub fn copiable_values(game: &GameState, id: ObjectId) -> Option<CopiableValues> {
    debug_assert_eq!(
        LAYER_ORDER[END_OF_LAYER_1],
        Layer::Layer2Control,
        "END_OF_LAYER_1 must be the ceiling just past every layer-1 sublayer \
         (CR 613.2c). A sublayer added to LAYER_ORDER moves it."
    );

    // A capture is a walk, and the measurement's frames-per-walk ratio is what
    // CR 613.7a's existence re-check costs. Counting it here keeps a copy-heavy
    // board honest in the same table as an anthem-heavy one.
    game.counters.record_layer_walk();
    let mut cache = FrameCache::new(None);
    let frame = compute_to_ceiling(game, id, END_OF_LAYER_1, &mut cache)?;
    Some(CopiableValues::from_frame(frame))
}
