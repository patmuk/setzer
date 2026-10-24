use setzer_core::{html, Html, ImageRef, Render};
use setzer_design::prelude::*;

/// A responsive image: `srcset` of all variants, `sizes` as given.
pub fn picture(img: &'static ImageRef, alt: &str, sizes: &str) -> Html {
    html! { img src=(img.src()) srcset=(img.srcset()) sizes=(sizes) width=(img.width()) height=(img.height()) alt=(alt) loading="lazy" decoding="async"; }
}

/// An image with a dark-mode variant. The browser downloads only the
/// variant that matches; the theme script flips `media` on a user override.
pub fn themed_picture(
    light: &'static ImageRef,
    dark: &'static ImageRef,
    alt: &str,
    sizes: &str,
) -> Html {
    html! {
        picture {
            source media="(prefers-color-scheme: dark)" srcset=(dark.srcset()) sizes=(sizes) data-dark;
            (picture(light, alt, sizes))
        }
    }
}

/// Article text: markdown output, styled by element inside one named class.
pub fn prose(body: impl Render) -> Html {
    html! { div class=(style([named("prose", prose_css), max_width(Width::Narrow)])) { (body) } }
}

fn prose_css() -> Css {
    rule(".prose img", [radius(Radius::S)])
        .rule(".prose pre, .prose code", [text(TextRole::Mono)])
        .rule(
            ".prose pre",
            [
                bg(Color::Surface),
                border(Stroke::Hairline, Color::Rule),
                radius(Radius::S),
                pad_xy(Space::Sm, Space::M),
                raw("overflow-x", "auto"),
            ],
        )
        .rule(
            ".prose blockquote",
            [
                margin_y(Space::M),
                raw("margin-inline", "0"),
                pad_left(Space::M),
                border_left(Stroke::Emphasized, Color::Accent),
                fg(Color::Muted),
            ],
        )
        .rule(".prose table", [raw("border-collapse", "collapse")])
        .rule(
            ".prose td, .prose th",
            [
                border(Stroke::Hairline, Color::Rule),
                pad_xy(Space::Xs, Space::S),
            ],
        )
}
