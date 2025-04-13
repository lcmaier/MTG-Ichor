// src/utils/constants/turns.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Beginning,
    Precombat, // Main phase 1
    Combat,
    Postcombat, // Main phase 2
    Ending,
}

#[derive(Debug, Clone)]
pub enum Step {
    Untap,
    Upkeep,
    Draw,
    BeginCombat,
    DeclareAttackers,
    DeclareBlockers,
    CombatDamage,
    EndCombat,
    End,
    Cleanup,
}
