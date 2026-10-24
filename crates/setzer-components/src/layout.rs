//! Layout presets: styles to put on your own elements, merge with `.with()`.

use setzer_design::prelude::*;

/// Centred column of the site's width with side gutters.
pub fn wrap() -> Style {
    style([column(Space::M, Width::Wrap)])
}

/// A page section: wrapped, spaced, a hairline above (none on the first).
pub fn section() -> Style {
    wrap().with(style([
        pad_top(Space::Hero),
        pad_bottom(Space::M),
        margin_top(Space::Xl),
        border_top(Stroke::Hairline, Color::Rule),
        first(border_top_none()),
        first(margin_top_none()),
        first(pad_top(Space::Xl)),
    ]))
}

/// A section in the narrow reading width.
pub fn narrow_section() -> Style {
    section().with(style([column(Space::M, Width::Narrow)]))
}

/// Two columns 3:2, stacked on narrow screens.
pub fn split() -> Style {
    style([
        grid(),
        columns_fr(&[3, 2]),
        gap(Space::Hero),
        at(Breakpoint::Narrow, columns(1)),
        at(Breakpoint::Narrow, gap(Space::L)),
    ])
}

/// Two equal columns, stacked on narrow screens.
pub fn two_col() -> Style {
    style([
        grid(),
        columns(2),
        gap(Space::Xl),
        at(Breakpoint::Narrow, columns(1)),
        at(Breakpoint::Narrow, gap(Space::L)),
    ])
}

/// Secondary text colour.
pub fn muted() -> Style {
    style([fg(Color::Muted)])
}
