//! Every card the crate builds is the card it built when `card-layer-cleanup`
//! began (`plans/handoffs/card-layer-cleanup.md`).
//!
//! The cleanup rebuilds every ability and instruction from shared words and
//! moves every card file. Neither may change a card, and `CardData` compares
//! by value, so each card's value is fingerprinted and the fingerprints are
//! held to `card_fingerprints.txt`, which the cleanup's base wrote. A
//! mismatch names the card; `UPDATE_CARD_FINGERPRINTS=1 cargo test --test
//! card_data_fingerprint_test` rewrites the file, which only a change that
//! means to change a card may do.
//!
//! The fingerprint is the value's pretty `Debug` with each hash set's entries
//! sorted, since a `HashSet` iterates in an order its process chooses. The
//! file is the cleanup's scaffolding: it goes when the cleanup's last
//! mechanical PR closes.

use std::sync::Arc;

use mtgsim::cards;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::objects::card_data::CardData;
use mtgsim::types::effects::TokenDef;

/// The card functions no registry lists: each is reached here by name.
const UNREGISTERED_CARDS: [(&str, fn() -> Arc<CardData>); 51] = [
    ("zhalfirin_shapecraft", cards::phase5_pre_cards::zhalfirin_shapecraft),
    ("inside_out", cards::phase5_pre_cards::inside_out),
    ("bull_rush", cards::phase5_pre_cards::bull_rush),
    ("locked_sphere", cards::phase_cm_cards::locked_sphere),
    ("generic_reducer", cards::phase_cm_cards::generic_reducer),
    ("green_reducer", cards::phase_cm_cards::green_reducer),
    ("red_reducer", cards::phase_cm_cards::red_reducer),
    ("power_reducer", cards::phase_cm_cards::power_reducer),
    ("ember_lesson", cards::phase_cm_cards::ember_lesson),
    ("crimson_lesson", cards::phase_cm_cards::crimson_lesson),
    ("verdant_lesson", cards::phase_cm_cards::verdant_lesson),
    ("kicked_lesson", cards::phase_cm_cards::kicked_lesson),
    ("bargain_lesson", cards::phase_cm_cards::bargain_lesson),
    ("twin_offering", cards::phase_cm_cards::twin_offering),
    ("self_eating_engine", cards::phase_cm_cards::self_eating_engine),
    ("fixture_curtain", cards::phase_history_and_frame_records_cards::fixture_curtain),
    ("fixture_token_tally", cards::phase_history_and_frame_records_cards::fixture_token_tally),
    ("fixture_mourning_ink", cards::phase_history_and_frame_records_cards::fixture_mourning_ink),
    ("fixture_mourners_tithe", cards::phase_history_and_frame_records_cards::fixture_mourners_tithe),
    ("fixture_first_creature_tithe", cards::phase_history_and_frame_records_cards::fixture_first_creature_tithe),
    ("chromatic_ward", cards::phase_lc_cards::chromatic_ward),
    ("liquimetal_coating_spell", cards::phase_ld_cards::liquimetal_coating_spell),
    ("ensoul_artifact_spell", cards::phase_ld_cards::ensoul_artifact_spell),
    ("call_to_serve_spell", cards::phase_ld_cards::call_to_serve_spell),
    ("on_serras_wings_spell", cards::phase_ld_cards::on_serras_wings_spell),
    ("urborg_effect", cards::phase_ld_cards::urborg_effect),
    ("dual_land_ub", cards::phase_ld_cards::dual_land_ub),
    ("lands_have_flying", cards::phase_ld_cards::lands_have_flying),
    ("land_creatures_have_flying", cards::phase_ld_cards::land_creatures_have_flying),
    ("self_stripping_land", cards::phase_ld_cards::self_stripping_land),
    ("purifier_clause", cards::phase_li_cards::purifier_clause),
    ("flight_clause", cards::phase_li_cards::flight_clause),
    ("simian_clause", cards::phase_li_cards::simian_clause),
    ("graveyard_painter", cards::phase_lj_cards::graveyard_painter),
    ("graveyard_reveler", cards::phase_lj_cards::graveyard_reveler),
    ("teferi_flash_clause", cards::phase_lj_cards::teferi_flash_clause),
    ("lattice_colorless_clause", cards::phase_lj_cards::lattice_colorless_clause),
    ("exiled_ancestor", cards::phase_lk_cards::exiled_ancestor),
    ("grist_insect_clause", cards::phase_ll_cards::grist_insect_clause),
    ("titanias_song_clause", cards::phase_ll_cards::titanias_song_clause),
    ("pocket_griffin", cards::phase_ll_cards::pocket_griffin),
    ("arcane_adaptation_elf_clause", cards::phase_ll_cards::arcane_adaptation_elf_clause),
    ("library_artificer", cards::phase_ll_cards::library_artificer),
    ("library_assassins", cards::phase_ll_cards::library_assassins),
    ("timid_golem", cards::phase_rf_cards::timid_golem),
    ("hollow_hands", cards::phase_rf_cards::hollow_hands),
    ("sealing_ward", cards::phase_rf_cards::sealing_ward),
    ("immortal_coils_state_trigger", cards::phase_tr6_cards::immortal_coils_state_trigger),
    ("draws_on_an_empty_hand", cards::phase_tr6_cards::draws_on_an_empty_hand),
    ("discard_your_hand_then_draw_that_many", cards::phase_tr6_cards::discard_your_hand_then_draw_that_many),
    ("destroy_your_others_then_a_token", cards::phase_tr6_cards::destroy_your_others_then_a_token),
];

/// The token definitions a printed card's effect makes, which no card's
/// value holds unless that card is built.
const TOKENS: [(&str, fn() -> TokenDef); 6] = [
    ("soldier_token", cards::phase_re_cards::soldier_token),
    ("goblin_token", cards::phase_re_cards::goblin_token),
    ("angel_token", cards::phase_re_cards::angel_token),
    ("saproling_token", cards::phase_tr1_cards::saproling_token),
    ("elf_warrior_token", cards::phase_tr2a_cards::elf_warrior_token),
    ("attacking_human_token", cards::phase_tr5f_cards::attacking_human_token),
];

/// Every card and token, keyed by the registered name or the function name,
/// each with its value's pretty `Debug`.
fn every_value() -> Vec<(String, String)> {
    let registry = CardRegistry::default_registry();
    let mut values: Vec<(String, String)> = registry
        .card_names()
        .into_iter()
        .map(|name| {
            let card = registry.create(name).unwrap_or_else(|missing| panic!("{missing}"));
            (name.to_string(), format!("{card:#?}"))
        })
        .collect();
    values.extend(UNREGISTERED_CARDS.iter().map(|(name, build)| (format!("fn {name}"), format!("{:#?}", build()))));
    values.extend(TOKENS.iter().map(|(name, build)| (format!("fn {name}"), format!("{:#?}", build()))));
    values.sort();
    values
}

/// `text`, a pretty `Debug`, with the entries of every set or map block
/// sorted. A block is a set's or a map's when its `{` follows no type name:
/// `colors: {` opens one, `AbilityText {` a struct. Each entry starts four
/// spaces in from the block's line and runs until the next such line, so an
/// entry that is itself a struct sorts whole, after its own sets are sorted.
fn sorted_sets(lines: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    let mut at = 0;
    while at < lines.len() {
        let line = lines[at];
        let opened = line.trim_end();
        let before = opened.strip_suffix('{').map(str::trim_end);
        let opens_a_set = before.is_some_and(|before| {
            before.is_empty() || before.ends_with(':') || before.ends_with('(') || before.ends_with('[') || before.ends_with(',')
        });
        if !opens_a_set {
            out.push(line.to_string());
            at += 1;
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let close = (at + 1..lines.len())
            .find(|&i| {
                let l = lines[i];
                l.len() - l.trim_start().len() == indent && l.trim_start().starts_with('}')
            })
            .unwrap_or_else(|| panic!("an unclosed set at line {at}"));
        let mut entries: Vec<Vec<&str>> = Vec::new();
        for &inner in &lines[at + 1..close] {
            if inner.len() - inner.trim_start().len() == indent + 4 || entries.is_empty() {
                entries.push(Vec::new());
            }
            if let Some(entry) = entries.last_mut() {
                entry.push(inner);
            }
        }
        let mut sorted: Vec<String> = entries.iter().map(|entry| sorted_sets(entry).join("\n")).collect();
        sorted.sort();
        out.push(line.to_string());
        out.extend(sorted);
        out.push(lines[close].to_string());
        at = close + 1;
    }
    out
}

/// FNV-1a, 64 bits: a fixed function, so the file reads the same in every
/// process and on every machine.
fn fingerprint(debug: &str) -> String {
    let lines: Vec<&str> = debug.lines().collect();
    let canonical = sorted_sets(&lines).join("\n");
    let hash = canonical.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    });
    format!("{hash:016x}")
}

#[test]
fn every_card_builds_as_it_did_at_the_cleanups_base() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("card_fingerprints.txt");
    let current: Vec<String> = every_value().iter().map(|(key, debug)| format!("{key}\t{}", fingerprint(debug))).collect();
    if std::env::var_os("UPDATE_CARD_FINGERPRINTS").is_some() {
        std::fs::write(&path, current.join("\n") + "\n").unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        return;
    }
    let recorded = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let recorded: Vec<&str> = recorded.lines().collect();
    let changed: Vec<&str> = current
        .iter()
        .map(String::as_str)
        .filter(|line| !recorded.contains(line))
        .chain(recorded.iter().copied().filter(|line| !current.iter().any(|now| now == line)))
        .collect();
    assert!(changed.is_empty(), "cards whose value is not the recorded one:\n{}", changed.join("\n"));
}

/// The canonical form is what makes the file one file: two sets holding the
/// same entries print alike whatever order each iterates in.
#[test]
fn a_sets_order_does_not_change_its_fingerprint() {
    let one = "Card {\n    colors: {\n        Red,\n        Blue,\n    },\n    name: \"x\",\n}";
    let other = "Card {\n    colors: {\n        Blue,\n        Red,\n    },\n    name: \"x\",\n}";
    assert_eq!(fingerprint(one), fingerprint(other));
    let named = "Card {\n    colors: {\n        Blue,\n        Red,\n    },\n    name: \"y\",\n}";
    assert_ne!(fingerprint(one), fingerprint(named));
}
