//! The home page, "/". Sub-folders whose mod.rs calls `setzer::page!` are its sub-pages.

use setzer::prelude::*;

setzer::page!(); // this folder is a page: "/", its text from content.md as `Content`

pub fn render(c: &Content, lang: Lang) -> Html {
    html! {
        (DOCTYPE)
        html lang=(lang.code()) {
            head { meta charset="utf-8"; title { (c.title) } (stylesheet()) }
            body {
                h1 { (word(c.hello)) " " (word(c.world)) }
                (c.body) // the markdown below the front matter
            }
        }
    }
}

/// One component, used twice.
fn word(label: &str) -> Html {
    html! {
        span class=(style([inline_block(), pad(Space::M), radius(Radius::M), bg(Color::AccentSoft), fg(Color::AccentFg), text(TextRole::H1)])) {
            (label)
        }
    }
}
