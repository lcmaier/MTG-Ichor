// src//utils//card_types.rs
use crate::utils::constants::card_types::*;

impl CardType {
    
    pub fn is_spell(&self) -> bool {
        !matches!(self, CardType::Land)
    }
    
    pub fn is_permanent(&self) -> bool {
        matches!(self,
            CardType::Artifact |
            CardType::Battle |
            CardType::Creature |
            CardType::Enchantment |
            CardType::Land |
            CardType::Planeswalker)
    }
}