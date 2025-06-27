// src/utils/constants/costs.rs

// Represents costs for activating abilities
#[derive(Debug, Clone, PartialEq)]
pub enum Cost {
    Tap,
    Mana(ManaCost),
    // Add other costs later (sacrifice, etc.)
}

// Mana costs

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct ManaCost {
    pub white: u8,
    pub blue: u8,
    pub black: u8,
    pub red: u8,
    pub green: u8,
    pub colorless: u8,
    pub generic: u8,
}

impl ManaCost {
    // Helpers for mana costs of each color (vast majority of cards have generic costs as well)
    pub fn white(white_amount: u8, generic_amount: u8) -> Self {
        ManaCost { white: white_amount, blue: 0, black: 0, red: 0, green: 0, colorless: 0, generic: generic_amount }
    }
    pub fn blue(blue_amount: u8, generic_amount: u8) -> Self {
        ManaCost { white: 0, blue: blue_amount, black: 0, red: 0, green: 0, colorless: 0, generic: generic_amount }
    }
    pub fn black(black_amount: u8, generic_amount: u8) -> Self {
        ManaCost { white: 0, blue: 0, black: black_amount, red: 0, green: 0, colorless: 0, generic: generic_amount }
    }
    pub fn red(red_amount: u8, generic_amount: u8) -> Self {
        ManaCost { white: 0, blue: 0, black: 0, red: red_amount, green: 0, colorless: 0, generic: generic_amount }
    }
    pub fn green(green_amount: u8, generic_amount: u8) -> Self {
        ManaCost { white: 0, blue: 0, black: 0, red: 0, green: green_amount, colorless: 0, generic: generic_amount }
    }
    pub fn colorless(colorless_amount: u8, generic_amount: u8) -> Self {
        ManaCost { white: 0, blue: 0, black: 0, red: 0, green: 0, colorless: colorless_amount, generic: generic_amount }
    }
    
}