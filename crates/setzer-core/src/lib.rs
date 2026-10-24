//! Setzer core: the site builder, page definitions, the render state that
//! pages, links and images report to, and the writers for HTML, CSS,
//! images, feeds, sitemap and headers.

pub mod cli;
pub mod config;
pub mod content;
pub mod frame;
pub mod image;
pub mod lang;
pub mod page;
pub mod render;
pub mod script;
pub mod site;
pub mod xml;

pub use config::{base_url, site_name, url};
pub use content::{documents_in, Body, Doc, Document, Part, StaticFile};
pub use frame::Frame;
pub use image::ImageRef;
pub use lang::Lang;
pub use maud::{html, Markup, PreEscaped, Render, DOCTYPE};
pub use page::{ContentMeta, Feed, FeedItem, Meta, PageDef, RenderFn};
pub use render::{anchor, href, stylesheet, theme_script};
pub use site::Site;

/// Every page of the program. Each `setzer::page!` adds its `PAGE` here
/// when the program is linked, so a page that is compiled is part of the site.
#[linkme::distributed_slice]
pub static PAGES: [&'static PageDef];

#[doc(hidden)]
pub use linkme as __linkme;

/// Rendered, escaped HTML (Maud's `Markup`).
pub type Html = Markup;

/// Renders an iterator of fragments back to back: `(each(items.iter().map(card)))`.
pub fn each<I>(items: I) -> Each<I>
where
    I: IntoIterator + Clone,
    I::Item: Render,
{
    Each(items)
}

pub struct Each<I>(I);

impl<I> Render for Each<I>
where
    I: IntoIterator + Clone,
    I::Item: Render,
{
    fn render_to(&self, buffer: &mut String) {
        for item in self.0.clone() {
            item.render_to(buffer);
        }
    }
}

/// Marks a folder's `mod.rs` as a page and holds its options:
///
/// ```ignore
/// setzer::page!();
/// setzer::page!(documents: "posts", path: "/404.html", feed: feed, subpages: [a, b]);
/// ```
///
/// `setzer::site!()` (in `main.rs`) finds the folders whose `mod.rs` call
/// this macro, reads the options from this call, and generates the page
/// around the file: the text structs, `PAGE`, the route. Here the macro only
/// imports what was generated, so `render` can use `Content`, `content`,
/// `PAGE` and the sub-page modules directly.
#[macro_export]
macro_rules! page {
    ($($options:tt)*) => {
        #[allow(unused_imports)]
        use super::*;
    };
}

/// What a page or component usually needs, besides the design prelude.
pub mod prelude {
    pub use crate::{
        anchor, each, href, html, stylesheet, theme_script, Body, Doc, Document, Feed, FeedItem,
        Frame, Html, ImageRef, Lang, PreEscaped, Render, DOCTYPE,
    };
}
