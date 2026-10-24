//! Reset and element defaults, shipped by every site (Tailwind's "base" layer).

use crate::css::{rule, Css};
use crate::tokens::*;
use crate::utility::*;

pub fn base_css() -> Css {
    rule("*, *::before, *::after", [raw("box-sizing", "border-box")])
        .rule(
            "html",
            [
                raw("font-size", "100%"),
                raw("scroll-behavior", "smooth"),
                scroll_padding_top(Size::Header),
            ],
        )
        .rule(
            "body",
            [
                margin_none(),
                bg(Color::Bg),
                fg(Color::Fg),
                text(TextRole::Body),
                raw("-webkit-font-smoothing", "antialiased"),
            ],
        )
        .rule("a", [fg(Color::AccentFg), no_underline()])
        .rule(
            "a:hover, a:focus-visible",
            [underline(), underline_offset(Stroke::Emphasized)],
        )
        .rule("img", [raw("max-width", "100%"), raw("height", "auto")])
        .rule("h1, h2, h3, p", [margin_top_none()])
        .rule("h1", [text(TextRole::H1), margin_bottom(Space::S)])
        .rule("h2", [text(TextRole::H2), margin_bottom(Space::M)])
        .rule("h3", [text(TextRole::H3), margin_bottom(Space::Sm)])
        .rule("p", [margin_bottom(Space::M)])
        .rule("button", [font_inherit()])
        .rule(
            ".skip",
            [
                raw("position", "absolute"),
                top(Space::M),
                left(Space::M),
                raw("transform", "translateY(-300%)"),
                bg(Color::Surface),
                pad_xy(Space::S, Space::M),
                layer(10),
            ],
        )
        .rule(".skip:focus", [raw("transform", "none")])
}
