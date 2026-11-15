/// The five colors of Magic (rule 105)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}

impl Color {
    /// The five colors, in the order CR 105.1 lists them: what "choose a
    /// color" offers (CR 105.4).
    pub const ALL: [Color; 5] = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];
}
