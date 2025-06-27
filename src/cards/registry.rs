// src/cards/registry.rs

use std::collections::HashMap;

use crate::utils::constants::game_objects::Characteristics;

use super::red::instant::lightning_bolt::lightning_bolt_characteristics;
use super::green::creature::grizzly_bears::grizzly_bears_characteristics;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CARD_CHARACTERISTICS: HashMap<&'static str, fn() -> Characteristics> = {
        let mut m = HashMap::new();
        m.insert("Lightning Bolt", lightning_bolt_characteristics as fn() -> Characteristics);
        m.insert("Grizzly Bears", grizzly_bears_characteristics as fn() -> Characteristics);
        m
    };
}
