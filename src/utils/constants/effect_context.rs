// src/utils/constants/effect_context.rs
#[derive(Debug, Clone)]
pub struct EffectContext {
    // Track state needed for conditional effects
    pub cards_drawn_this_turn: u32,
    // Add more as needed
}

impl EffectContext {
    pub fn new() -> Self {
        EffectContext { cards_drawn_this_turn: 0 }
    }

    pub fn reset_turn_tracking(&mut self) {
        self.cards_drawn_this_turn = 0;
    }
}