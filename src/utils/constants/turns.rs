// src/utils/constants/turns.rs
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum PhaseType {
    Beginning,
    Precombat, // Main phase 1
    Combat,
    Postcombat, // Main phase 2 (and any additional main phases granted by effects per rule 505.1a)
    Ending,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum StepType {
    Untap,
    Upkeep,
    Draw,
    BeginCombat,
    DeclareAttackers,
    DeclareBlockers,
    FirstStrikeDamage,
    CombatDamage,
    EndCombat,
    End,
    Cleanup,
}
