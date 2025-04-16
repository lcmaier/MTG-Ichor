// src/utils/traits/zonestate.rs

use crate::utils::constants::zones::Zone;
use crate::utils::constants::game_objects::{BattlefieldState, CommandState, ExileState, GraveyardState, HandState, LibraryState, StackState};

pub trait ZoneState {
    // Zone-specific behaviors that all zone states need
    fn zone() -> Zone; // returns current zone
}

impl ZoneState for LibraryState {
    fn zone() -> Zone { Zone::Library }
}

impl ZoneState for HandState {
    fn zone() -> Zone { Zone::Hand }
}

impl ZoneState for BattlefieldState {
    fn zone() -> Zone { Zone::Battlefield }
}

impl ZoneState for StackState {
    fn zone() -> Zone { Zone::Stack }
}

impl ZoneState for GraveyardState {
    fn zone() -> Zone { Zone::Graveyard }
}

impl ZoneState for ExileState {
    fn zone() -> Zone { Zone::Exile }
}

impl ZoneState for CommandState {
    fn zone() -> Zone { Zone::Command }
}
