//! Why a scenario was not built (`setup-architecture.md` §4.1).

/// Which of §4.1's classes a refusal is. A state the rules would correct
/// (0 toughness, an Aura attached to nothing) is no refusal: it is built, and
/// CR 117.5's check performs it before the first priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusalKind {
    /// A line the grammar does not read.
    Syntax,
    /// A name no registered card, and no card in development, has.
    NotACard,
    /// A reference that names nothing, or two things.
    Reference,
    /// A state no sequence of events reaches, named by the rule that says so.
    Unreachable,
}

/// The line, what is wrong, and what to change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub kind: RefusalKind,
    /// 1-based; `None` for a fact about the file as a whole.
    pub line: Option<usize>,
    pub message: String,
}

impl Refusal {
    pub(crate) fn at(kind: RefusalKind, line: usize, message: impl Into<String>) -> Refusal {
        Refusal { kind, line: Some(line), message: message.into() }
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.line {
            Some(line) => write!(f, "line {line}: {}", self.message),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for Refusal {}
