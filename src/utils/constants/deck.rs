// src/utils/constants/deck.rs

use crate::utils::constants::id_types::PlayerId;
use crate::utils::constants::game_objects::{GameObj, LibraryState};

pub const MINIMUM_CONSTRUCTED_DECK_SIZE: usize = 60;
pub const MINIMUM_LIMITED_DECK_SIZE: usize = 40;
pub const COMMANDER_DECK_SIZE: usize = 100;


#[derive(Debug, Clone)]
pub struct Deck {
    pub cards: Vec<GameObj<LibraryState>>,
    pub owner: PlayerId,
    pub deck_format: DeckFormat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeckFormat {
    Constructed,
    Limited,
    Commander,
    // other deck formats as needed (differentiated by minimum/fixed size)
}

impl DeckFormat {
    pub fn min_size(&self) -> usize {
        match self {
            DeckFormat::Constructed => MINIMUM_CONSTRUCTED_DECK_SIZE,
            DeckFormat::Limited => MINIMUM_LIMITED_DECK_SIZE,
            DeckFormat::Commander => COMMANDER_DECK_SIZE
        }
    }

    pub fn max_size(&self) -> Option<usize> {
        match self {
            DeckFormat::Commander => Some(COMMANDER_DECK_SIZE), // commander decks must be exactly 100 cards (including commander(s))
            _ => None
        }
    }
}