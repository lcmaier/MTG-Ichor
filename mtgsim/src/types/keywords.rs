/// Keyword abilities (rule 702)
///
/// These are the standard keyword abilities that can appear on cards.
/// This enum is used both for printed keywords and for granted keywords
/// (via continuous effects like "creatures you control have flying").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeywordAbility {
    Deathtouch,
    Defender,
    DoubleStrike,
    Enchant, // parameterized in the ability definition, not here
    Equip,   // parameterized in the ability definition, not here
    FirstStrike,
    Flash,
    Flying,
    Haste,
    Hexproof,
    Indestructible,
    Intimidate,
    Landwalk, // parameterized by land type in ability definition
    Lifelink,
    Menace,
    Protection, // parameterized by quality in ability definition
    Reach,
    Shroud,
    Trample,
    Vigilance,
    Ward, // parameterized by cost in ability definition
    // Add more as needed — this covers the most common ones
}
