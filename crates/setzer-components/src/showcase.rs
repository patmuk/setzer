//! Product-page shapes: an intro with text beside a picture, and feature rows
//! that alternate sides. The picture is any `Html`: a screenshot, a logo, two
//! phones side by side.

use crate::layout::wrap;
use setzer_core::{each, html, Html};
use setzer_design::prelude::*;

/// Text on the left, `media` on the right; one column on tablets and phones.
pub struct Intro<'a> {
    pub eyebrow: &'a str,
    pub title: &'a str,
    pub lead: &'a str,
    pub pitch: &'a str,
    /// Buttons below the text, e.g. `button(…)`.
    pub actions: Html,
    pub media: Html,
}

pub fn intro(i: &Intro) -> Html {
    html! {
        section class=(wrap().with(style([grid(), columns_fr(&[1, 1]), items_center(), gap(Space::Hero), pad_y(Space::Divider),
                                          at(Breakpoint::Narrow, columns(1)), at(Breakpoint::Narrow, gap(Space::L)), at(Breakpoint::Narrow, pad_y(Space::Xl))]))) {
            div {
                p class=(style([text(TextRole::Overline), fg(Color::AccentFg), margin_bottom(Space::M)])) { (i.eyebrow) }
                h1 class=(style([text(TextRole::Display), margin_bottom(Space::M)])) { (i.title) }
                p class=(style([text(TextRole::Lead), weight(600), margin_bottom(Space::S)])) { (i.lead) }
                p class=(style([text(TextRole::Lead), fg(Color::Muted), max_width(Size::HeroText)])) { (i.pitch) }
                div class=(style([flex(), flex_wrap(), items_center(), gap(Space::S), margin_top(Space::L)])) { (i.actions) }
            }
            div { (i.media) }
        }
    }
}

pub struct Feature<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub text: Html,
    pub media: Html,
}

/// One row per feature, the picture alternating left and right; on tablets
/// and phones the picture comes above its text.
pub fn features(items: &[Feature]) -> Html {
    let row = |(n, f): (usize, &Feature)| {
        let flip = n % 2 == 1;
        let ratio: &[u8] = if flip { &[2, 3] } else { &[3, 2] };
        html! {
            section id=(f.id) class=(wrap().with(style([grid(), columns_fr(ratio), items_center(), gap(Space::Hero), pad_y(Space::Hero),
                                                        at(Breakpoint::Narrow, columns(1)), at(Breakpoint::Narrow, gap(Space::M)), at(Breakpoint::Narrow, pad_y(Space::Xl))]))) {
                @if flip {
                    div class=(style([at(Breakpoint::Narrow, order_last())])) { h2 { (f.title) } (f.text) }
                    div { (f.media) }
                } @else {
                    div { (f.media) }
                    div { h2 { (f.title) } (f.text) }
                }
            }
        }
    };
    html! { (each(items.iter().enumerate().map(row))) }
}
