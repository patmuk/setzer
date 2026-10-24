//! What a frame (the site's shared header, footer and `<head>`) is told
//! about the page it wraps.

use crate::lang::Lang;

pub struct Frame {
    pub lang: Lang,
    /// Root-relative, language prefixed.
    pub path: String,
    pub title: Option<&'static str>,
    pub description: Option<&'static str>,
    /// Absolute URL of this page.
    pub canonical: String,
    /// Every language this page exists in, with its absolute URL.
    pub alternates: Vec<(Lang, String)>,
    /// The other languages: this page there if it exists, else that language's home.
    pub switches: Vec<(Lang, String)>,
    /// Root-relative path of the site's RSS feed in this language, if any.
    pub feed: Option<String>,
    /// `true` for a page of a collection (a blog post).
    pub document: bool,
}
