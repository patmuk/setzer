use setzer_core::{html, Html};
use setzer_design::prelude::*;

/// How a button looks.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Look {
    Primary,
    Secondary,
    /// Translucent, for use over a photo (secondary on narrow screens,
    /// where the photo becomes a strip above the text).
    OnPhoto,
}

pub fn button(label: &str, href: &str, look: Look) -> Html {
    let base = style([
        inline_block(),
        pad_xy(Space::S, Space::M),
        radius(Radius::Pill),
        border(Stroke::Hairline, Color::Rule),
        bg(Color::Surface),
        fg(Color::Fg),
        weight(600),
        hover(border_color(Color::Accent)),
        hover(no_underline()),
    ]);
    let look = match look {
        Look::Primary => base.with(style([
            bg(Color::Accent),
            border_color(Color::Accent),
            fg(Color::OnAccent),
        ])),
        Look::Secondary => base,
        Look::OnPhoto => base.with(style([
            bg(Color::GhostFill),
            border_color(Color::GhostBorder),
            fg(Color::OnPhoto),
            at(Breakpoint::Narrow, bg(Color::Surface)),
            at(Breakpoint::Narrow, border_color(Color::Rule)),
            at(Breakpoint::Narrow, fg(Color::Fg)),
        ])),
    };
    html! { a class=(look) href=(href) { (label) } }
}
