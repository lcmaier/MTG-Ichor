/// Configuration that varies by format. Pure data, no behavior.
///
/// Covers Standard, Modern, Pioneer, Limited, and most two-player formats
/// out of the box. When Commander/Brawl are needed, a `Format` trait will
/// provide `config()` and override behavioral hooks; `GameConfig` becomes
/// a field of the `Format` implementor. The struct fields and their types
/// don't change — only where the behavior lives.

/// Mulligan rule in use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MulliganRule {
    /// Current official rule — draw 7, bottom N where N = number of mulligans taken.
    London,
    /// Older rule — shuffle back and draw one fewer.
    Paris,
    /// No mulligans (useful for tests).
    None,
}

/// Deck construction constraints for a format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeckLimits {
    /// Minimum deck size (60 standard, 40 limited, 99 commander).
    pub min_deck_size: usize,
    /// Maximum deck size (None = unlimited).
    pub max_deck_size: Option<usize>,
    /// Maximum copies of any non-basic card (4 for most formats, 1 for commander).
    /// `None` means no copy limit (e.g. Limited).
    pub max_copies: Option<u32>,
    /// Sideboard size (15 for constructed, None for limited/commander).
    pub sideboard_size: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameConfig {
    pub starting_life: i64,
    pub starting_hand_size: usize,
    pub max_hand_size: i32,
    /// Whether the first player draws on their first turn (false in standard 2-player).
    pub first_player_draws: bool,
    pub mulligan_rule: MulliganRule,
    pub deck_limits: DeckLimits,
}

impl GameConfig {
    /// Standard/Modern/Pioneer defaults.
    pub fn standard() -> Self {
        GameConfig {
            starting_life: 20,
            starting_hand_size: 7,
            max_hand_size: 7,
            first_player_draws: false,
            mulligan_rule: MulliganRule::London,
            deck_limits: DeckLimits {
                min_deck_size: 60,
                max_deck_size: None,
                max_copies: Some(4),
                sideboard_size: Some(15),
            },
        }
    }

    /// Limited (draft/sealed) defaults.
    pub fn limited() -> Self {
        GameConfig {
            starting_life: 20,
            starting_hand_size: 7,
            max_hand_size: 7,
            first_player_draws: false,
            mulligan_rule: MulliganRule::London,
            deck_limits: DeckLimits {
                min_deck_size: 40,
                max_deck_size: None,
                max_copies: None, // limited allows any number
                sideboard_size: None,  // all unused cards are sideboard
            },
        }
    }

    /// Minimal config for tests — no deck restrictions, no mulligans.
    pub fn test() -> Self {
        GameConfig {
            starting_life: 20,
            starting_hand_size: 7,
            max_hand_size: 7,
            first_player_draws: true, // simplifies tests
            mulligan_rule: MulliganRule::None,
            deck_limits: DeckLimits {
                min_deck_size: 0,
                max_deck_size: None,
                max_copies: None,
                sideboard_size: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_config() {
        let config = GameConfig::standard();
        assert_eq!(config.starting_life, 20);
        assert_eq!(config.starting_hand_size, 7);
        assert_eq!(config.max_hand_size, 7);
        assert!(!config.first_player_draws);
        assert_eq!(config.mulligan_rule, MulliganRule::London);
        assert_eq!(config.deck_limits.min_deck_size, 60);
        assert_eq!(config.deck_limits.max_copies, Some(4));
        assert_eq!(config.deck_limits.sideboard_size, Some(15));
    }

    #[test]
    fn test_limited_config() {
        let config = GameConfig::limited();
        assert_eq!(config.deck_limits.min_deck_size, 40);
        assert!(config.deck_limits.max_copies.is_none());
        assert!(config.deck_limits.sideboard_size.is_none());
    }

    #[test]
    fn test_test_config() {
        let config = GameConfig::test();
        assert!(config.first_player_draws);
        assert_eq!(config.mulligan_rule, MulliganRule::None);
        assert_eq!(config.deck_limits.min_deck_size, 0);
    }
}
