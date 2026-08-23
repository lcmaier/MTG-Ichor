/// Keywords whose entire meaning is their presence (CR 702).
///
/// # What belongs here, and what does not
///
/// Rule 702 defines 189 keyword abilities. They do not all want the same
/// representation, and the axis that decides is **does the engine branch on
/// the keyword, or execute it?** — crossed with whether the keyword takes a
/// parameter:
///
/// |  | No parameter | Parameterized |
/// |---|---|---|
/// | **Engine branches on it** | ① **this enum** — flying, trample, deathtouch, vigilance, … | ② a set of *values* — protection from [quality], [type]walk |
/// | **Engine executes it** | ③ a plain [`AbilityDef`] — storm, prowess, persist, devoid | ④ an [`AbilityDef`] with args — equip [cost], ward [cost], cycling [cost] |
///
/// [`AbilityDef`]: crate::objects::card_data::AbilityDef
///
/// CR 702.9b is the tell for ①: "A creature with flying can't be blocked
/// except by creatures with flying and/or reach." That is a rule about the
/// blocking system keyed on a name — there is no ability body to store, so
/// presence is the whole payload and a `HashSet` says everything there is to
/// say. Compare CR 702.6a, which spells out equip's body in full, cost and
/// target included.
///
/// Every variant below is quadrant ①, and every one is consumed somewhere:
/// combat (`engine::combat`), state-based actions (`engine::sba`), casting
/// (`engine::cast`, flash), or damage (`engine::keywords`).
///
/// # Quadrants ③ and ④ are already here, in the right place
///
/// Devoid is CR 702.114, a keyword ability — and it lives in
/// `cards::phase_le_cards` as a characteristic-defining [`AbilityDef`] on
/// `CardData::abilities`, not as a variant here. That is quadrant ③, and it
/// is what the rest of ③ and ④ should look like. `EffectModification::
/// GrantAbility` is the Layer 6 channel for them; `GrantKeywordFlag` is the
/// channel for this enum.
///
/// # Five variants used to be here
///
/// `Enchant`, `Equip`, `Landwalk`, `Protection` and `Ward` were removed once
/// Layer 6 gained a real ability-granting channel, because a fieldless variant
/// cannot hold what those keywords are *made of*:
///
/// - **Equip** (CR 702.6a) is an *activated* ability, "[Cost]: Attach this
///   permanent to target creature you control." The cost is the ability —
///   `equip {3}` on every Sword, `equip {1}` on Skullclamp. And CR 702.6d
///   lets a permanent have *several* equip abilities, which a `HashSet` of one
///   variant structurally cannot express.
/// - **Enchant** (CR 702.5a) is a static ability, and it was already modelled
///   properly elsewhere: `CardData::enchant_filter` is what the Aura targeting
///   path actually reads. The variant was a duplicate of a working field.
/// - **Protection** (CR 702.16a) is quadrant ②. Rules branch on it (702.16b–e
///   govern targeting, enchanting, blocking and damage), but it carries a
///   quality, and a permanent can have protection from several qualities at
///   once. It wants `HashSet<Quality>` on the frame, not a name here.
/// - **Landwalk** (CR 702.14a) is quadrant ② for the same reason, keyed on a
///   land type.
/// - **Ward** (CR 702.21a) is a *triggered* ability, so it needs CR 603
///   regardless of how it is stored.
///
/// Removing them cost nothing — none was constructed anywhere in the crate —
/// and keeping them would have made `GrantKeywordFlag(Protection)` look like the
/// way to write "target creature gains protection from the color of your
/// choice", which is common Magic and which it cannot express.
///
/// # Still open
///
/// - Quadrant ② has no frame representation yet. Built with the first card
///   that needs one, not before.
/// - Naming an ability by its keyword ("this `AbilityDef` is *equip*") wants a
///   separate complete-CR-702 `KeywordName` enum — a different type doing a
///   different job from this one. Wanted by UI display and by cards that
///   reference a keyword by name.
/// - `Hexproof` here is CR 702.11's base form, which is fieldless and correct.
///   "Hexproof from [quality]" (702.11d) is quadrant ② and is not modelled.
///
/// See `codebase-state.md` Deferred Migrations item 10.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeywordFlag {
    Deathtouch,
    Defender,
    DoubleStrike,
    FirstStrike,
    Flash,
    Flying,
    Haste,
    Hexproof,
    Indestructible,
    Intimidate,
    Lifelink,
    Menace,
    Reach,
    Shroud,
    Trample,
    Vigilance,
    // Quadrant ① only. A keyword with a parameter, or with an ability body the
    // engine executes, goes on `CardData::abilities` as an `AbilityDef` — see
    // the type docs above before adding a variant here.
}
