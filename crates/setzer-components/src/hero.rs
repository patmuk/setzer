use crate::button::{button, Look};
use crate::layout::wrap;
use setzer_core::{each, html, Html, ImageRef};
use setzer_design::prelude::*;

pub struct Link<'a> {
    pub label: &'a str,
    pub href: &'a str,
}

/// Full-width photo band, CV style: subject on the right, text over a dark
/// fade on the left. On narrow screens the photo becomes a strip above.
pub struct Hero<'a> {
    pub eyebrow: &'a str,
    pub title: &'a str,
    pub lead: &'a str,
    pub pitch: &'a str,
    pub primary: Link<'a>,
    pub secondary: Option<Link<'a>>,
    pub banner: &'static ImageRef,
}

pub fn hero(h: &Hero) -> Html {
    html! {
        section class=(style([named("hero-banner", banner_css)])) {
            img src=(h.banner.src()) srcset=(h.banner.srcset()) sizes="100vw" width=(h.banner.width()) height=(h.banner.height()) alt="" fetchpriority="high";
            div class=(wrap().with(style([class("hero-inner")]))) {
                div {
                    p class=(style([class("hero-eyebrow")])) { (h.eyebrow) }
                    h1 { (h.title) }
                    p class=(style([class("hero-lead")])) { (h.lead) }
                    p class=(style([class("hero-pitch")])) { (h.pitch) }
                    p class=(style([flex(), flex_wrap(), gap(Space::S), margin_top(Space::L)])) {
                        (button(h.primary.label, h.primary.href, Look::Primary))
                        @if let Some(s) = &h.secondary { " " (button(s.label, s.href, Look::OnPhoto)) }
                    }
                }
            }
        }
    }
}

/// Everything here depends on the photo behind it, hence one named rule set.
fn banner_css() -> Css {
    rule(
        ".hero-banner",
        [
            relative(),
            raw("overflow", "hidden"),
            bg(Color::PhotoBackdrop),
            fg(Color::OnPhoto),
            raw("aspect-ratio", "2598 / 1100"),
            max_height(Size::BannerMax),
            grid(),
            items_center(),
            border_bottom(Stroke::Hairline, Color::Rule),
        ],
    )
    .rule(
        ".hero-banner > img",
        [
            raw("position", "absolute"),
            raw("inset", "0"),
            full_width(),
            full_height(),
            raw("object-fit", "cover"),
            raw("object-position", "90% 0"),
        ],
    )
    .rule(
        ".hero-banner::before",
        [
            raw("content", "\"\""),
            raw("position", "absolute"),
            raw("inset", "0"),
            layer(1),
            gradient(
                90,
                &[
                    (Color::ScrimHeavy, 0),
                    (Color::ScrimStrong, 40),
                    (Color::ScrimWeak, 68),
                    (Color::ScrimNone, 85),
                ],
            ),
        ],
    )
    .rule(
        ".hero-inner",
        [
            relative(),
            layer(2),
            grid(),
            columns_capped_first(Size::HeroText),
            pad_y(Space::Hero),
        ],
    )
    .rule(
        ".hero-banner h1",
        [fg(Color::OnPhoto), text_shadow(Shadow::Text)],
    )
    .rule(
        ".hero-eyebrow",
        [
            text(TextRole::BodySm),
            weight(600),
            margin_bottom(Space::Xs),
            fg(Color::Accent),
            raw("filter", "brightness(1.25)"),
        ],
    )
    .rule(
        ".hero-lead",
        [
            text(TextRole::Lead),
            fg(Color::OnPhotoMuted),
            margin_bottom(Space::M),
        ],
    )
    .rule(
        ".hero-pitch",
        [fg(Color::OnPhoto), max_width(Size::HeroText)],
    )
    .at(Breakpoint::Narrow, |c| {
        c.rule(
            ".hero-banner",
            [
                bg(Color::Bg),
                fg(Color::Fg),
                raw("aspect-ratio", "auto"),
                raw("max-height", "none"),
                block(),
                raw("overflow", "visible"),
            ],
        )
        .rule(
            ".hero-banner > img",
            [
                raw("position", "static"),
                block(),
                height(Size::BannerStrip),
                raw("object-position", "75% 30%"),
            ],
        )
        .rule(".hero-banner::before", [hidden()])
        .rule(".hero-inner", [columns(1), pad_top(Space::L)])
        .rule(".hero-banner h1", [fg(Color::Fg), no_text_shadow()])
        .rule(
            ".hero-eyebrow",
            [fg(Color::AccentFg), raw("filter", "none")],
        )
        .rule(".hero-lead", [fg(Color::Muted)])
        .rule(".hero-pitch", [fg(Color::Fg)])
    })
}

pub struct Fact<'a> {
    pub value: &'a str,
    pub label: &'a str,
}

/// Row of key numbers, e.g. under a hero.
pub fn facts(items: &[Fact], aria_label: &str) -> Html {
    let item = |f: &Fact| {
        html! {
            div class=(style([bg(Color::Surface), border(Stroke::Hairline, Color::Rule), radius(Radius::M), pad(Space::M), shadow(Shadow::Card)])) {
                strong class=(style([block(), text(TextRole::H2), fg(Color::AccentFg)])) { (f.value) }
                span class=(style([text(TextRole::BodySm), fg(Color::Muted)])) { (f.label) }
            }
        }
    };
    html! {
        section aria-label=(aria_label) class=(wrap().with(style([grid(), columns(4), gap(Space::M), margin_top(Space::L), at(Breakpoint::Narrow, columns(2))]))) {
            (each(items.iter().map(item)))
        }
    }
}
