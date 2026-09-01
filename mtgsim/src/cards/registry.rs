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
use super::phase_rb_cards;
use super::phase_rc_cards;
use super::phase_rs_cards;
use super::phase_sba_cards;

/// The board an engine change is measured against — **representative, not
/// frozen** (revised 2026-09-01).
///
/// It was frozen for a year, to keep runs comparable against baselines recorded
/// in `plans/`. That rationale is gone: `CLAUDE.md` mandates an **interleaved
/// A/B in one sitting**, both arms of which use this pool by construction, so
/// stability across months buys the timing measurement nothing — and commit
/// `a926627` showed a stored ms/game number is machine drift, not a baseline.
///
/// What a freeze cost instead was *representativeness*. RS-1 added a gated
/// subsystem no card in the pool could open, so its A/B measured only the
/// closed path — the pool had begun measuring a shrinking fraction of the
/// engine. That is the same failure the two-pool split was created to fix one
/// level down, where a card was kept out of the *registry* to protect a number.
///
/// **The rule now: a phase that opens a new engine path adds one card here,
/// deliberately, and re-records the gameplay table in
/// `plans/engineering-practices.md` §3.** That table's seed-deterministic rows
/// — turns, spells cast, creatures died — are what an addition invalidates and
/// what still has to be re-measured. Registering a card is still not the same
/// act as adding one here.
const PERFORMANCE_POOL: [&str; 60] = [
    "Plains",
    "Island",
    "Swamp",
    "Mountain",
    "Forest",
    "Lightning Bolt",
    "Ancestral Recall",
    "Counterspell",
    "Burst of Energy",
    "Volcanic Upheaval",
    "Giant Growth",
    "Grizzly Bears",
    "Hill Giant",
    "Savannah Lions",
    "Earth Elemental",
    "Serra Angel",
    "Thornweald Archer",
    "Raging Cougar",
    "Wall of Stone",
    "Elvish Archers",
    "Ridgetop Raptor",
    "War Mammoth",
    "Knight of Meadowgrain",
    "Rhox War Monk",
    "Giant Spider",
    "Vampire Nighthawk",
    "Isamaru, Hound of Konda",
    "Night's Whisper",
    "Doom Blade",
    "Angel's Mercy",
    "Dark Ritual",
    "Glorious Anthem",
    "Cerulean Wisps",
    "Crimson Wisps",
    "Moonlace",
    "Blood Moon",
    "March of the Machines",
    "Tarmogoyf",
    "Culling Drone",
    "Humility",
    "Citanul Hierophants",
    "Act of Treason",
    "Sol Ring",
    "Darksteel Myr",
    "Merfolk Thaumaturgist",
    "Tundra",
    "Underground Sea",
    "Badlands",
    "Taiga",
    "Savannah",
    "Scrubland",
    "Volcanic Island",
    "Bayou",
    "Plateau",
    "Tropical Island",
    // RS-1 — the first cards that open the CR 101.2 restriction sweep. A pair,
    // because neither is measurable alone: Sigarda populates
    // `restriction_ability_sources` (and nothing else in the pool does), and
    // the edict is the only resolution that asks her anything.
    "Sigarda, Host of Herons",
    "Diabolic Edict",
    // RC-2 — the first cards that make entering the battlefield a replaceable
    // event. A pair, because CR 614.1c's two halves are two code paths: one
    // writes a status before anything can observe the permanent, the other
    // allocates a CR 613.7c timestamp per counter kind. The land is also the
    // pool's first nonbasic whose *own* ability CR 305.7 can strip, which is
    // what makes Blood Moon's effect on a tapland measurable here.
    "Idyllic Beachfront",
    "Chainbreaker",
    // The first card in the crate to propose a `GameAction::AddCounters`.
    // Chainbreaker is already here and enters with -1/-1 counters, so this is
    // also what makes CR 704.5q's annihilation sweep run in a measured game
    // rather than only in a fixture.
    "Battlegrowth",
];

/// Card registry: maps card names to factory functions that produce CardData.
///
/// Contributors add new cards by:
/// 1. Creating a function that returns CardData (using CardDataBuilder)
/// 2. Registering it here with `register()`
///
/// This keeps card definitions purely data-driven — no engine code needed.
///
/// Two pools are built from these definitions — `default_registry` (everything)
/// and `performance_pool` (representative). `plans/engineering-practices.md` §
/// "Two card pools" says which to reach for.
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

    /// Every card the crate can build. `fuzz_games --pool stress` plays this
    /// one, and it grows with each phase.
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

        // Phase RB — the first replacement effect with printed card text.
        registry.register("Kalitas, Traitor of Ghet", phase_rb_cards::kalitas_traitor_of_ghet);
        // The second and third replacement sources. Before these, CR 616.1's
        // "two or more" branch was unreachable in any game — see
        // `engineering-practices.md` §3.3.
        registry.register("Rest in Peace", phase_rb_cards::rest_in_peace);
        registry.register("Leyline of the Void", phase_rb_cards::leyline_of_the_void);

        // Phase RS-1 — the first CR 101.2 "can't" with printed card text, and
        // the resolution that makes its *absence of a prompt* observable.
        registry.register("Sigarda, Host of Herons", phase_rs_cards::sigarda_host_of_herons);
        registry.register("Diabolic Edict", phase_rs_cards::diabolic_edict);

        // Phase RC-2 — the first cards that modify how a permanent enters
        // (CR 614.1c/d). One per half of `EnterMods`: status and counters.
        registry.register("Idyllic Beachfront", phase_rc_cards::idyllic_beachfront);
        registry.register("Chainbreaker", phase_rc_cards::chainbreaker);
        // The third RC-2 card, and it was written but never registered — its
        // own doc comment says "this one grows the stress pool", which was
        // false for as long as this line was missing.
        registry.register("Adaptive Shimmerer", phase_rc_cards::adaptive_shimmerer);

        // The +1/+1 half of CR 704.5q. Its -1/-1 half is Chainbreaker above,
        // and until this card the annihilation sweep had never run in a fuzz
        // game — `engineering-practices.md` §3.3.
        registry.register("Battlegrowth", phase_sba_cards::battlegrowth);

        registry
    }

    /// The subset an engine change is A/B'd against — see [`PERFORMANCE_POOL`]
    /// for why it grows rather than stays frozen.
    ///
    /// Panics if a name in `PERFORMANCE_POOL` is no longer registered. That is
    /// the point: a rename that silently shrank the pool would leave two runs
    /// incomparable without saying so, which is the one failure a *deliberate*
    /// addition does not have.
    pub fn performance_pool() -> Self {
        let all = Self::default_registry();
        let mut registry = CardRegistry::new();
        for name in PERFORMANCE_POOL {
            let factory = all.cards.get(name).unwrap_or_else(|| {
                panic!("performance pool names {name:?}, which is no longer registered")
            });
            registry.register(name, *factory);
        }
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

    #[test]
    fn test_every_performance_pool_name_still_resolves() {
        // `performance_pool` panics on a name it cannot find, so building it is
        // the check. The count is asserted against the array's own length
        // rather than a literal: the pool grows by deliberate act now, and a
        // hardcoded number would fail the *addition* instead of failing the
        // thing worth catching, which is a name that stopped resolving.
        let pool = CardRegistry::performance_pool();
        assert_eq!(pool.card_names().len(), PERFORMANCE_POOL.len());
    }

    #[test]
    fn test_stress_pool_is_a_superset_of_the_performance_pool() {
        // The two pools share definitions rather than duplicating them, so the
        // only way to fail this is to unregister a card the performance list
        // names.
        let all = CardRegistry::default_registry();
        for name in CardRegistry::performance_pool().card_names() {
            assert!(all.create(name).is_ok(), "{name:?} left the full registry");
        }
        assert!(
            all.card_names().len() > PERFORMANCE_POOL.len(),
            "the stress pool has stopped being a strict superset"
        );
    }
}
