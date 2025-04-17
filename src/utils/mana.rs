// src/utils/mana.rs
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ManaType {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless
}

#[derive(Debug, Clone)]
pub struct ManaPool {
    pub mana: HashMap<ManaType, u64>,
}

// Methods for ManaPool

// Add one or more mana of a specific type to the mana pool
impl ManaPool {
    // Create a new empty mana pool
    pub fn new() -> Self {
        ManaPool {
            mana: HashMap::new(),
        }
    }

    // Get all available mana (for displaying to player)
    pub fn get_available_mana(&self) -> HashMap<ManaType, u64> {
        self.mana.clone()
    }

    pub fn add_mana(&mut self, mana_type: ManaType, amount: u64) {
        *self.mana.entry(mana_type).or_insert(0) += amount;
    }

    // Remove mana from pool
    pub fn remove_mana(&mut self, mana_type: ManaType, amount: u64) -> Result<(), String> {
        if let Some(mana) = self.mana.get_mut(&mana_type) {
            if *mana >= amount {
                *mana -= amount;
                Ok(())
            } else {
                Err(format!("Not enough {:?} mana in the pool", mana_type))
            }
        } else {
            Err(format!("No {:?} mana in the pool", mana_type))
        }
    }

    // Check if pool has enough mana of a specific type
    pub fn has_mana(&self, mana_type: ManaType, amount: u64) -> bool {
        if let Some(mana) = self.mana.get(&mana_type) {
            *mana >= amount
        } else {
            false
        }
    }

    // empty the mana pool
    pub fn empty(&mut self) {
        self.mana.clear();
    }
}