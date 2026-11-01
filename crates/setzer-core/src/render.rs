//! While `Site::write` renders a page, it keeps a little state here: the
//! language, which pages and sections were linked, which images, icons and
//! scripts were used. Links, images and icons report to it; afterwards the
//! site checks the links and produces exactly the images and icons used.

use crate::config;
use crate::content::StaticFile;
use crate::image::ImageRef;
use crate::lang::Lang;
use crate::page::PageDef;
use crate::script::SCRIPT;
use maud::{Markup, PreEscaped, html};
use std::cell::RefCell;

/// Replaced by the hashed stylesheet path after all pages are rendered.
pub(crate) const STYLESHEET_PLACEHOLDER: &str = "/assets/css/__setzer_stylesheet__.css";

#[derive(Default)]
pub(crate) struct State {
    pub lang: Option<Lang>,
    /// Lang-prefixed path of the page being rendered.
    pub path: String,
    /// (target path, anchor, linked from)
    pub links: Vec<(String, Option<String>, String)>,
    pub images: Vec<&'static ImageRef>,
    /// Icons the pages linked; written with the site, an SVG favicon with
    /// its PNG fallback.
    pub icons: Vec<&'static StaticFile>,
    /// Inline scripts the pages used; their hashes go into the CSP.
    pub scripts: std::collections::BTreeSet<&'static str>,
}

thread_local! {
    pub(crate) static STATE: RefCell<State> = RefCell::new(State::default());
}

pub(crate) fn begin_page(lang: Lang, path: &str) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.lang = Some(lang);
        s.path = path.to_string();
    });
}

/// The language of the page being rendered.
pub fn lang() -> Lang {
    STATE
        .with(|s| s.borrow().lang)
        .unwrap_or_else(config::default_lang)
}

fn record_link(path: &str, anchor: Option<&str>) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let from = s.path.clone();
        s.links
            .push((path.to_string(), anchor.map(str::to_string), from));
    });
}

pub(crate) fn record_anchor(path: Option<&str>, anchor: &str) {
    let path = path
        .map(str::to_string)
        .unwrap_or_else(|| STATE.with(|s| s.borrow().path.clone()));
    record_link(&path, Some(anchor));
}

pub(crate) fn record_image(img: &'static ImageRef) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if !s.images.iter().any(|i| std::ptr::eq(*i, img)) {
            s.images.push(img);
        }
    });
}

fn record_icon(file: &'static StaticFile) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if !s.icons.iter().any(|f| f.path == file.path) {
            s.icons.push(file);
        }
    });
}

/// Size of the PNG that stands in for an SVG favicon.
pub(crate) const FAVICON_PNG_SIZE: u32 = 32;

/// Where the PNG fallback of an SVG favicon is served: `favicon-3f9a1c2b.svg`
/// → `favicon-3f9a1c2b-32.png`.
pub(crate) fn favicon_png(svg: &StaticFile) -> Option<String> {
    svg.path
        .strip_suffix(".svg")
        .map(|stem| format!("{stem}-{FAVICON_PNG_SIZE}.png"))
}

/// `<link rel="icon">` for the browser tab, from an SVG or a PNG of the
/// crate (`static_file!`). The site publishes it; for an SVG it also renders
/// a 32×32 PNG, for browsers without SVG favicons (older Safari).
pub fn favicon(file: &'static StaticFile) -> Markup {
    record_icon(file);
    match favicon_png(file) {
        // The PNG first, the SVG last: browsers that can show the SVG take it.
        Some(png) => html! {
            link rel="icon" href={ "/" (png) } type="image/png" sizes=(format!("{FAVICON_PNG_SIZE}x{FAVICON_PNG_SIZE}"));
            link rel="icon" href=(file.url_path()) type="image/svg+xml";
        },
        None => html! { link rel="icon" href=(file.url_path()); },
    }
}

/// `<link rel="apple-touch-icon">`: a 180×180 PNG for the iOS home screen.
/// The site publishes it.
pub fn apple_touch_icon(file: &'static StaticFile) -> Markup {
    record_icon(file);
    html! { link rel="apple-touch-icon" href=(file.url_path()); }
}

/// Root-relative path of `page` in `lang`. The link is checked after
/// rendering: its target must be one of the site's pages.
pub fn href(page: &'static PageDef, lang: Lang) -> String {
    let path = config::in_lang(&page.route(), lang);
    record_link(&path, None);
    path
}

/// Path of one document of a collection page, e.g. a blog post: `Doc::href`.
pub(crate) fn doc_path(page: &'static PageDef, slug: &str, lang: Lang) -> String {
    let path = config::in_lang(&format!("{}{slug}/", page.route()), lang);
    record_link(&path, None);
    path
}

/// `href(page, lang)#id`. The section is checked after rendering: the
/// target page must contain an element with `id="<id>"`.
pub fn anchor(page: &'static PageDef, id: &str, lang: Lang) -> String {
    let path = config::in_lang(&page.route(), lang);
    record_link(&path, Some(id));
    format!("{path}#{id}")
}

/// `<link rel="stylesheet">` for the site's stylesheet.
pub fn stylesheet() -> Markup {
    html! { link rel="stylesheet" href=(STYLESHEET_PLACEHOLDER); }
}

/// The light/dark script (Auto → Light → Dark toggle). Using it adds its
/// hash to the Content-Security-Policy; sites without it run no script.
pub fn theme_script() -> Markup {
    script(SCRIPT)
}

/// An inline `<script>`, for the rare page that needs one. Its hash goes into
/// the Content-Security-Policy, so exactly these scripts run and no others.
/// Write the code into the page (e.g. `script(include_str!("open.js"))`), not
/// text from outside.
pub fn script(js: &'static str) -> Markup {
    STATE.with(|s| s.borrow_mut().scripts.insert(js));
    html! { script { (PreEscaped(js)) } }
}

#[cfg(test)]
mod tests {
    use super::*;

    static SVG: StaticFile = StaticFile {
        path: "assets/img/favicon-3f9a1c2b.svg",
        bytes: b"<svg/>",
    };
    static PNG: StaticFile = StaticFile {
        path: "assets/img/favicon-3f9a1c2b.png",
        bytes: b"",
    };

    #[test]
    fn svg_favicon_has_a_png_fallback() {
        let html = favicon(&SVG).into_string();
        assert_eq!(
            html,
            r#"<link rel="icon" href="/assets/img/favicon-3f9a1c2b-32.png" type="image/png" sizes="32x32"><link rel="icon" href="/assets/img/favicon-3f9a1c2b.svg" type="image/svg+xml">"#
        );
        assert!(STATE.with(|s| s.borrow().icons.iter().any(|f| f.path == SVG.path)));
    }

    #[test]
    fn png_favicon_stands_alone() {
        assert_eq!(
            favicon(&PNG).into_string(),
            r#"<link rel="icon" href="/assets/img/favicon-3f9a1c2b.png">"#
        );
    }
}
