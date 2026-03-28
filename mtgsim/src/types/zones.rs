/// Game zones (rule 400.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Zone {
    Library,
    Hand,
    Battlefield,
    Graveyard,
    Stack,
    Exile,
    Command,
}

impl Zone {
    /// Whether objects in this zone are public information
    pub fn is_public(&self) -> bool {
        matches!(self, Zone::Battlefield | Zone::Graveyard | Zone::Stack | Zone::Exile | Zone::Command)
    }
}
