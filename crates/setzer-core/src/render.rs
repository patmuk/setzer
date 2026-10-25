//! While `Site::write` renders a page, it keeps a little state here: the
//! language, which pages and sections were linked, which images and whether
//! the theme script were used. Links and images report to it; afterwards the
//! site checks the links and produces exactly the images used.

use crate::config;
use crate::image::ImageRef;
use crate::lang::Lang;
use crate::page::PageDef;
use crate::script::SCRIPT;
use maud::{html, Markup, PreEscaped};
use std::cell::RefCell;

/// Replaced by the hashed stylesheet path after all pages are rendered.
pub(crate) const STYLESHEET_PLACEHOLDER: &str = "/css/__setzer_stylesheet__.css";

#[derive(Default)]
pub(crate) struct State {
    pub lang: Option<Lang>,
    /// Lang-prefixed path of the page being rendered.
    pub path: String,
    /// (target path, anchor, linked from)
    pub links: Vec<(String, Option<String>, String)>,
    pub images: Vec<&'static ImageRef>,
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
