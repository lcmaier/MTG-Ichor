use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::*;
use crate::types::mana::ManaType;

/// Create the CardData for a Plains
pub fn plains() -> CardData {
    CardDataBuilder::new("Plains")
        .card_type(CardType::Land)
        .supertype(Supertype::Basic)
        .subtype(Subtype::Land(LandType::Plains))
        .mana_ability_single(ManaType::White)
        .build()
}

/// Create the CardData for an Island
pub fn island() -> CardData {
    CardDataBuilder::new("Island")
        .card_type(CardType::Land)
        .supertype(Supertype::Basic)
        .subtype(Subtype::Land(LandType::Island))
        .mana_ability_single(ManaType::Blue)
        .build()
}

/// Create the CardData for a Swamp
pub fn swamp() -> CardData {
    CardDataBuilder::new("Swamp")
        .card_type(CardType::Land)
        .supertype(Supertype::Basic)
        .subtype(Subtype::Land(LandType::Swamp))
        .mana_ability_single(ManaType::Black)
        .build()
}

/// Create the CardData for a Mountain
pub fn mountain() -> CardData {
    CardDataBuilder::new("Mountain")
        .card_type(CardType::Land)
        .supertype(Supertype::Basic)
        .subtype(Subtype::Land(LandType::Mountain))
        .mana_ability_single(ManaType::Red)
        .build()
}

/// Create the CardData for a Forest
pub fn forest() -> CardData {
    CardDataBuilder::new("Forest")
        .card_type(CardType::Land)
        .supertype(Supertype::Basic)
        .subtype(Subtype::Land(LandType::Forest))
        .mana_ability_single(ManaType::Green)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::AbilityType;

    #[test]
    fn test_all_basic_lands() {
        let lands = vec![
            (plains(), "Plains", ManaType::White),
            (island(), "Island", ManaType::Blue),
            (swamp(), "Swamp", ManaType::Black),
            (mountain(), "Mountain", ManaType::Red),
            (forest(), "Forest", ManaType::Green),
        ];

        for (land, expected_name, _expected_mana) in &lands {
            assert_eq!(land.name, *expected_name);
            assert!(land.types.contains(&CardType::Land));
            assert!(land.supertypes.contains(&Supertype::Basic));
            assert!(land.mana_cost.is_none());
            assert_eq!(land.abilities.len(), 1);
            assert_eq!(land.abilities[0].ability_type, AbilityType::Mana);
        }
    }
}
