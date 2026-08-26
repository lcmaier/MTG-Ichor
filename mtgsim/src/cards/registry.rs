use std::collections::HashMap;
use std::sync::Arc;

use crate::objects::card_data::CardData;

use super::basic_lands;
use super::dual_lands;
use super::alpha;
use super::artifacts;
use super::creatures;
use super::keyword_creatures;
use super::utility_creatures;
use super::phase5_pre_cards;
use super::phase_lc_cards;
use super::phase_ld_cards;
use super::phase_le_cards;
use super::phase_lf_cards;
use super::phase_lg_cards;

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

    /// Get all registered card names, alphabetically.
    ///
    /// Sorted, not raw `HashMap` key order: `fuzz_games` builds its decks by
    /// drawing from this list with a seeded RNG, so an order that changes per
    /// process makes the same seed build a different deck.
    pub fn card_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.cards.keys().map(|s| s.as_str()).collect();
        names.sort_unstable();
        names
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
        registry.register("Giant Growth", alpha::giant_growth);

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
        registry.register("Glorious Anthem", phase5_pre_cards::glorious_anthem);

        // Layer-system cards (Phases LC-LF).
        //
        // Only the ones whose definition is faithful to the printed card, because
        // this registry feeds `cli_play` as well as `fuzz_games`. The other
        // `phase_l*_cards` entries stay out and each has a reason:
        //
        // - Chromatic Ward and Windswept Heights are invented cards.
        // - Urborg Effect is a deliberate stand-in (an Enchantment carrying the
        //   land's ability) rather than Urborg itself.
        // - The four `*_spell` cards are Auras and an Artifact re-modeled as
        //   Instants to avoid machinery their phase did not need. Real names,
        //   wrong card types.
        // - Underground Sea, Cloudspire Mesa and Moonlit Steppe are Lands whose
        //   phases modeled them loosely; the ten real dual lands below are what
        //   `fuzz_games::random_deck` draws its nonbasic land slots from.
        registry.register("Cerulean Wisps", phase_lc_cards::cerulean_wisps);
        registry.register("Crimson Wisps", phase_lc_cards::crimson_wisps);
        registry.register("Moonlace", phase_lc_cards::moonlace);
        registry.register("Blood Moon", phase_ld_cards::blood_moon);
        registry.register("March of the Machines", phase_ld_cards::march_of_the_machines);
        registry.register("Tarmogoyf", phase_le_cards::tarmogoyf);
        registry.register("Culling Drone", phase_le_cards::culling_drone);
        registry.register("Humility", phase_lf_cards::humility);
        registry.register("Citanul Hierophants", phase_lf_cards::citanul_hierophants);
        registry.register("Act of Treason", phase_lg_cards::act_of_treason);

        // Layer 7b and 7d had no random-play coverage until these two: March of
        // the Machines was registered with no artifact in the crate to animate,
        // and nothing anywhere switched a creature's P/T. Sol Ring is colorless,
        // so every deck gets it; the Thaumaturgist rides in blue decks, which is
        // where March lives too.
        registry.register("Sol Ring", artifacts::sol_ring);
        registry.register("Darksteel Myr", artifacts::darksteel_myr);
        registry.register("Merfolk Thaumaturgist", utility_creatures::merfolk_thaumaturgist);

        // The ten original duals. Nonbasic lands with basic land types, which is
        // what gives `fuzz_games` a mana base Blood Moon can actually affect.
        registry.register("Tundra", dual_lands::tundra);
        registry.register("Underground Sea", dual_lands::underground_sea);
        registry.register("Badlands", dual_lands::badlands);
        registry.register("Taiga", dual_lands::taiga);
        registry.register("Savannah", dual_lands::savannah);
        registry.register("Scrubland", dual_lands::scrubland);
        registry.register("Volcanic Island", dual_lands::volcanic_island);
        registry.register("Bayou", dual_lands::bayou);
        registry.register("Plateau", dual_lands::plateau);
        registry.register("Tropical Island", dual_lands::tropical_island);

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
