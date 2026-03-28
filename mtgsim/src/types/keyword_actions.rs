/// Keyword actions (rule 701)
///
/// These are verbs that spells and effects perform. They are mechanically
/// distinct from keyword abilities (rule 702), which are static properties
/// on permanents. Keyword actions are things that *happen* during resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeywordAction {
    Activate,
    Attach,
    Cast,
    Counter,
    Create,
    Destroy,
    Discard,
    Draw,
    Exchange,
    Exile,
    Fight,
    Mill,
    Play,
    Reveal,
    Sacrifice,
    Scry,
    Search,
    Shuffle,
    Tap,
    Untap,
}
