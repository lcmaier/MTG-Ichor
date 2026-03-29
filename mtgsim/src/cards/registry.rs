use std::collections::HashMap;
use std::sync::Arc;

use crate::objects::card_data::CardData;

use super::basic_lands;

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

        // Non-land cards will be registered here as they're implemented

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
