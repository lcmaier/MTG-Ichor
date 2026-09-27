/// Declares an enum of unit variants with a `COUNT` counted off the same
/// list, so a new variant moves the count with no second edit. Stable Rust
/// has no variant count (`std::mem::variant_count` is nightly-only).
///
/// `counted_enum! { pub enum Light { Red, Amber, Green, } }` expands to that
/// enum as written, plus `impl Light { pub const COUNT: usize = [Light::Red,
/// Light::Amber, Light::Green].len(); }`, which the compiler evaluates to 3.
/// Each variant needs its trailing comma.
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
