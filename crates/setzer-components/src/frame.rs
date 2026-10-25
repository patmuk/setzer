//! The usual frame: `<head>` with metadata, sticky header with navigation,
//! language switch and theme toggle, footer. Call it from your site's frame
//! function: `fn frame(page: &Frame, body: Html) -> Html { standard_frame(page, &parts(page.lang), body) }`.

use crate::layout::wrap;
use setzer_core::{each, html, site_name, stylesheet, theme_script, Frame, Html, DOCTYPE};
use setzer_design::prelude::*;

pub struct NavItem {
    pub label: String,
    pub href: String,
}

/// Labels of the light/dark toggle, in the page's language.
pub struct ThemeLabels {
    pub label: String,
    pub auto: String,
    pub light: String,
    pub dark: String,
}

/// What `standard_frame` needs from your site; everything else comes from the page.
pub struct FrameParts {
    pub home: String,
    pub nav: Vec<NavItem>,
    pub skip: String,
    pub favicon: String,
    /// Image before the site name in the header (e.g. `/logo.svg`); the name stays as text.
    pub logo: Option<String>,
    /// The logo for dark mode, if it needs other colours (`None`: `logo` in both).
    pub logo_dark: Option<String>,
    /// Description for pages that have none.
    pub description: String,
    /// Open Graph share image (root-relative or absolute).
    pub og_image: Option<String>,
    pub footer: Html,
    /// `None`: system light/dark only, no script on the site.
    pub theme: Option<ThemeLabels>,
}

/// A complete, ready-to-use frame: `<head>`, sticky header, `<main>`, footer.
pub fn standard_frame(page: &Frame, parts: &FrameParts, body: Html) -> Html {
    let name = site_name();
    let title = match page.title {
        Some(t) if t != name => format!("{t} · {name}"),
        _ => name.to_string(),
    };
    let description = page.description.unwrap_or(&parts.description);
    let og_type = if page.document { "article" } else { "website" };
    let x_default = page.alternates.first().map(|(_, u)| u.clone());
    let nav_link =
        |n: &NavItem| html! { a href=(n.href) class=(style([fg(Color::Fg)])) { (n.label) } };
    let switch = |(lang, href): &(setzer_core::Lang, String)| html! { a href=(href) hreflang=(lang.code()) lang=(lang.code()) class=(style([fg(Color::Muted)])) { (lang.native_name()) } };
    html! {
        (DOCTYPE)
        html lang=(page.lang.code()) {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }
                meta name="description" content=(description);
                link rel="canonical" href=(page.canonical);
                @for (lang, url) in &page.alternates { link rel="alternate" hreflang=(lang.code()) href=(url); }
                @if let Some(d) = &x_default { link rel="alternate" hreflang="x-default" href=(d); }
                @if let Some(feed) = &page.feed { link rel="alternate" type="application/rss+xml" title=(name) href=(setzer_core::url(feed)); }
                meta property="og:type" content=(og_type);
                meta property="og:title" content=(title);
                meta property="og:description" content=(description);
                meta property="og:url" content=(page.canonical);
                meta property="og:site_name" content=(name);
                meta property="og:locale" content=(page.lang.locale());
                @if let Some(img) = &parts.og_image { meta property="og:image" content=(absolute(img)); }
                link rel="icon" href=(parts.favicon) type="image/svg+xml";
                (stylesheet())
                @if parts.theme.is_some() { (theme_script()) }
            }
            body {
                a.skip href="#main" { (parts.skip) }
                header class=(style([sticky_top(), layer(5), bg(Color::HeaderBg), blur(Size::Blur), border_bottom(Stroke::Hairline, Color::Rule)])) {
                    div class=(wrap().with(style([flex(), flex_wrap(), items_center(), justify_between(), gap_xy(Space::Xs, Space::M), pad_y(Space::Sm)]))) {
                        a href=(parts.home) class=(style([class("brand"), flex(), items_center(), gap(Space::S), weight(700), fg(Color::Fg)])) {
                            @if let Some(logo) = &parts.logo {
                                @if let Some(logo_dark) = &parts.logo_dark {
                                    img src=(logo) alt="" height="32" class=(style([height(Space::Xl), raw("width", "auto"), dark(hidden())]));
                                    img src=(logo_dark) alt="" height="32" class=(style([height(Space::Xl), raw("width", "auto"), hidden(), dark(block())]));
                                } @else {
                                    img src=(logo) alt="" height="32" class=(style([height(Space::Xl), raw("width", "auto")]));
                                }
                            }
                            span { (name) }
                        }
                        nav aria-label="Main" class=(style([flex(), flex_wrap(), items_center(), gap_xy(Space::Tight, Space::M), text(TextRole::BodySm)])) {
                            (each(parts.nav.iter().map(nav_link)))
                            (each(page.switches.iter().map(switch)))
                            @if let Some(t) = &parts.theme {
                                button #theme-toggle type="button" hidden
                                    data-label=(t.label) data-auto=(t.auto) data-light=(t.light) data-dark=(t.dark)
                                    class=(style([text(TextRole::Small), fg(Color::Muted), bg_none(), border(Stroke::Hairline, Color::Rule), radius(Radius::Pill), pad_xy(Space::Tight, Space::Sm), pointer(), hover(fg(Color::Fg)), hover(border_color(Color::Accent))]))
                                    { (t.auto) }
                            }
                        }
                    }
                }
                main #main { (body) }
                footer class=(style([margin_top(Space::Divider), pad_top(Space::Xl), pad_bottom(Space::Hero), border_top(Stroke::Hairline, Color::Rule), text(TextRole::Small)])) {
                    div class=(wrap()) { (parts.footer) }
                }
            }
        }
    }
}

fn absolute(path: &str) -> String {
    if path.starts_with("http") {
        path.to_string()
    } else {
        setzer_core::url(path)
    }
}
