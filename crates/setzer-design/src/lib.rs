//! Design system for Setzer sites. Same vocabulary as a Flutter `AppTheme`.
//!
//! * [`tokens`]: the *names* (`Space::M`, `TextRole::H2`, `Color::Surface`),
//!   each rendering as `var(--…)`. A typo is a compile error.
//! * [`theme`]: the *values* behind the names, light and dark.
//!   `Theme::default()` is an 8pt grid; every scale can be replaced.
//! * [`utility`]: typed utilities (`pad(Space::M)`), used in markup through
//!   [`style()`] and in named rule sets through [`rule()`].
//! * [`style::record`]: rendering a style records the CSS it needs.

pub mod base;
pub mod css;
pub mod emit;
pub mod lint;
pub mod style;
pub mod theme;
pub mod tokens;
pub mod utility;

pub use css::{Css, rule};
pub use style::{Style, style};
pub use theme::{Colors, Fonts, SYSTEM_MONO, SYSTEM_SANS, TextStyle, Theme};
pub use tokens::*;
pub use utility::Utility;

/// Tokens, utilities, `style` and `rule`: what a component needs.
pub mod prelude {
    pub use crate::css::{Css, rule};
    pub use crate::style::{Style, style};
    pub use crate::tokens::*;
    pub use crate::utility::*;
}
