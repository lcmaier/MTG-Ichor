use std::collections::HashMap;
use std::sync::Arc;

use crate::objects::card_data::CardData;

use super::basic_lands;
use super::alpha;
use super::creatures;
use super::keyword_creatures;
use super::phase5_pre_cards;

/// Card registry: maps card names to factory functions that produce CardData.
///
/// Contributors add new cards by:
/// 1. Creating a function that returns CardData (using CardDataBuilder)
/// 2. Registering it here with `register()`
///
/// This keeps card definitions purely data-driven — no engine code needed.
pub struct CardRegistry {
    cards: HashMap<String, fn() -> Arc<CardData>>,
}

impl CardRegistry {
    pub fn new() -> Self {
        CardRegistry {
            cards: HashMap::new(),
        }
    }

    /// Register a card factory function
    pub fn register(&mut self, name: &str, factory: fn() -> Arc<CardData>) {
        self.cards.insert(name.to_string(), factory);
    }

    /// Look up a card by name and create a fresh Arc<CardData>
    pub fn create(&self, name: &str) -> Result<Arc<CardData>, String> {
        self.cards.get(name)
            .map(|factory| factory())
            .ok_or_else(|| format!("Card '{}' not found in registry", name))
    }

    /// Get all registered card names
    pub fn card_names(&self) -> Vec<&str> {
        self.cards.keys().map(|s| s.as_str()).collect()
    }

    /// Build the default registry with all known cards
    pub fn default_registry() -> Self {
        let mut registry = CardRegistry::new();

        // Basic lands
        registry.register("Plains", basic_lands::plains);
        registry.register("Island", basic_lands::island);
        registry.register("Swamp", basic_lands::swamp);
        registry.register("Mountain", basic_lands::mountain);
        registry.register("Forest", basic_lands::forest);

        // Alpha set spells (Phase 2)
        registry.register("Lightning Bolt", alpha::lightning_bolt);
        registry.register("Ancestral Recall", alpha::ancestral_recall);
        registry.register("Counterspell", alpha::counterspell);
        registry.register("Burst of Energy", alpha::burst_of_energy);
        registry.register("Volcanic Upheaval", alpha::volcanic_upheaval);

        // Vanilla creatures (Phase 3)
        registry.register("Grizzly Bears", creatures::grizzly_bears);
        registry.register("Hill Giant", creatures::hill_giant);
        registry.register("Savannah Lions", creatures::savannah_lions);
        registry.register("Earth Elemental", creatures::earth_elemental);

        // Keyword creatures (Phase 4)
        registry.register("Serra Angel", keyword_creatures::serra_angel);
        registry.register("Thornweald Archer", keyword_creatures::thornweald_archer);
        registry.register("Raging Cougar", keyword_creatures::raging_cougar);
        registry.register("Wall of Stone", keyword_creatures::wall_of_stone);
        registry.register("Elvish Archers", keyword_creatures::elvish_archers);
        registry.register("Ridgetop Raptor", keyword_creatures::ridgetop_raptor);
        registry.register("War Mammoth", keyword_creatures::war_mammoth);
        registry.register("Knight of Meadowgrain", keyword_creatures::knight_of_meadowgrain);
        registry.register("Rhox War Monk", keyword_creatures::rhox_war_monk);
        registry.register("Giant Spider", keyword_creatures::giant_spider);
        registry.register("Vampire Nighthawk", keyword_creatures::vampire_nighthawk);

        // Phase 5 pre cards
        registry.register("Isamaru, Hound of Konda", phase5_pre_cards::isamaru_hound_of_konda);
        registry.register("Night's Whisper", phase5_pre_cards::nights_whisper);
        registry.register("Doom Blade", phase5_pre_cards::doom_blade);
        registry.register("Angel's Mercy", phase5_pre_cards::angels_mercy);
        registry.register("Dark Ritual", phase5_pre_cards::dark_ritual);

        registry
    }
}

impl Default for CardRegistry {
    fn default() -> Self {
        Self::default_registry()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::card_types::CardType;

    #[test]
    fn test_default_registry_has_basic_lands() {
        let registry = CardRegistry::default_registry();

        for name in &["Plains", "Island", "Swamp", "Mountain", "Forest"] {
            let card = registry.create(name).unwrap();
            assert_eq!(card.name, *name);
            assert!(card.types.contains(&CardType::Land));
        }
    }

    #[test]
    fn test_registry_unknown_card() {
        let registry = CardRegistry::default_registry();
        assert!(registry.create("Black Lotus").is_err());
    }
}
