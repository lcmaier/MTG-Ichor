/// Declares an enum of unit variants with a `COUNT` counted off the same
/// list, so a new variant moves the count with no second edit. Stable Rust
/// has no variant count (`std::mem::variant_count` is nightly-only).
///
/// `counted_enum! { pub enum Light { Red, Amber, Green, } }` expands to that
/// enum as written, plus `impl Light { pub const COUNT: usize = [Light::Red,
/// Light::Amber, Light::Green].len(); }`, which the compiler evaluates to 3.
/// Each variant needs its trailing comma.
///
/// Defined here, above the `mod` lines, so every module in `types` has it in
/// scope: a `macro_rules!` is visible only after its definition.
macro_rules! counted_enum {
    ($(#[$attr:meta])* $vis:vis enum $name:ident { $($variant:ident,)+ }) => {
        $(#[$attr])*
        $vis enum $name {
            $($variant,)+
        }

        impl $name {
            /// How many variants there are.
            pub const COUNT: usize = [$($name::$variant),+].len();
        }
    };
}

pub mod ids;
pub mod colors;
pub mod mana;
pub mod card_types;
pub mod zones;
pub mod keywords;
pub mod keyword_actions;
pub mod costs;
pub mod effects;
pub mod replacement;
pub mod restriction;
pub mod cost_modification;
pub mod triggers;
pub mod history;
