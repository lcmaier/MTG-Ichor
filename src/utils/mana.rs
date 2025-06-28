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
        if amount == 0 {
            Ok(()) // nothing to do if need to remove 0 mana
        } else if let Some(mana) = self.mana.get_mut(&mana_type) {
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

    pub fn get_generic_mana(&self) -> u64 {
        self.mana.iter().fold(0, |acc, (_, &amount)| acc + amount)
    }

    // Check if pool has enough mana of a specific type
    pub fn has_mana(&self, mana_type: ManaType, amount: u64) -> bool {
        if amount == 0 {
            true // you always have at least 0 mana of a given type in your mana pool
        } else if let Some(mana) = self.mana.get(&mana_type) {
            // println!("Has {} {:?} mana", mana, mana_type);
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


// UNIT TESTS
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mana_pool_creation() {
        let pool = ManaPool::new();
        assert_eq!(pool.get_generic_mana(), 0);
        assert!(pool.get_available_mana().is_empty());
    }
    
    #[test]
    fn test_add_mana() {
        let mut pool = ManaPool::new();
        
        pool.add_mana(ManaType::Red, 3);
        pool.add_mana(ManaType::Blue, 2);
        pool.add_mana(ManaType::Red, 1); // Add more red
        
        assert_eq!(pool.get_available_mana().get(&ManaType::Red), Some(&4));
        assert_eq!(pool.get_available_mana().get(&ManaType::Blue), Some(&2));
        assert_eq!(pool.get_generic_mana(), 6);
    }
    
    #[test]
    fn test_remove_mana_success() {
        let mut pool = ManaPool::new();
        pool.add_mana(ManaType::Green, 5);
        
        let result = pool.remove_mana(ManaType::Green, 3);
        assert!(result.is_ok());
        assert_eq!(pool.get_available_mana().get(&ManaType::Green), Some(&2));
    }
    
    #[test]
    fn test_remove_mana_exact_amount() {
        let mut pool = ManaPool::new();
        pool.add_mana(ManaType::White, 2);
        
        let result = pool.remove_mana(ManaType::White, 2);
        assert!(result.is_ok());
        assert_eq!(pool.get_available_mana().get(&ManaType::White), Some(&0));
    }
    
    #[test]
    fn test_remove_mana_insufficient() {
        let mut pool = ManaPool::new();
        pool.add_mana(ManaType::Black, 2);
        
        let result = pool.remove_mana(ManaType::Black, 3);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not enough Black mana in the pool");
        assert_eq!(pool.get_available_mana().get(&ManaType::Black), Some(&2)); // Unchanged
    }
    
    #[test]
    fn test_remove_mana_not_present() {
        let mut pool = ManaPool::new();
        
        let result = pool.remove_mana(ManaType::Colorless, 1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No Colorless mana in the pool");
    }
    
    #[test]
    fn test_remove_zero_mana() {
        let mut pool = ManaPool::new();
        
        // Should succeed even with no mana
        let result = pool.remove_mana(ManaType::Red, 0);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_has_mana() {
        let mut pool = ManaPool::new();
        pool.add_mana(ManaType::Red, 3);
        
        assert!(pool.has_mana(ManaType::Red, 0)); // Always have 0
        assert!(pool.has_mana(ManaType::Red, 1));
        assert!(pool.has_mana(ManaType::Red, 3));
        assert!(!pool.has_mana(ManaType::Red, 4));
        assert!(!pool.has_mana(ManaType::Blue, 1)); // Don't have blue
    }
    
    #[test]
    fn test_empty_mana_pool() {
        let mut pool = ManaPool::new();
        pool.add_mana(ManaType::Red, 3);
        pool.add_mana(ManaType::Blue, 2);
        pool.add_mana(ManaType::Green, 1);
        
        assert_eq!(pool.get_generic_mana(), 6);
        
        pool.empty();
        
        assert_eq!(pool.get_generic_mana(), 0);
        assert!(pool.get_available_mana().is_empty());
        assert!(!pool.has_mana(ManaType::Red, 1));
    }
    
    #[test]
    fn test_all_mana_types() {
        let mut pool = ManaPool::new();
        
        // Test all mana types
        pool.add_mana(ManaType::White, 1);
        pool.add_mana(ManaType::Blue, 1);
        pool.add_mana(ManaType::Black, 1);
        pool.add_mana(ManaType::Red, 1);
        pool.add_mana(ManaType::Green, 1);
        pool.add_mana(ManaType::Colorless, 2);
        
        assert_eq!(pool.get_generic_mana(), 7);
        
        let available = pool.get_available_mana();
        assert_eq!(available.len(), 6);
        assert_eq!(available.get(&ManaType::Colorless), Some(&2));
    }
}