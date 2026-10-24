//! Components: functions from props to HTML. Styling is utilities on the
//! elements (`style([...])`); a named class only where a style depends on
//! something outside the element (markdown output, photo-dependent colours).
//! Every component's CSS is recorded when it renders, so a site ships only
//! the CSS of components it uses.

pub mod button;
pub mod frame;
pub mod hero;
pub mod layout;
pub mod lists;
pub mod media;

pub use button::{button, Look};
pub use frame::{standard_frame, FrameParts, NavItem, ThemeLabels};
pub use hero::{facts, hero, Fact, Hero, Link};
pub use layout::{muted, narrow_section, section, split, two_col, wrap};
pub use lists::{
    accordion, cards, contact, doc_list, doc_teasers, meta_line, notice, terms, timeline,
    titled_list, translations, Card, ContactRow, Period, Term,
};
pub use media::{picture, prose, themed_picture};

#[cfg(test)]
mod lint {
    use super::*;
    use setzer_core::html;
    use setzer_design::lint::assert_clean;
    use setzer_design::style::record;

    /// Every component's CSS, as recorded when it renders, uses tokens only.
    #[test]
    fn components_use_tokens_only() {
        record::start();
        let _ = button("b", "/", Look::Primary);
        let _ = button("b", "/", Look::OnPhoto);
        let _ = facts(
            &[Fact {
                value: "1",
                label: "l",
            }],
            "facts",
        );
        let _ = cards(&[Card {
            kind: "k",
            title: "t",
            text: "x",
            href: "/",
        }]);
        let _ = terms(&[Term {
            title: "t",
            text: "x",
        }]);
        let _ = titled_list(&[Term {
            title: "t",
            text: "x",
        }]);
        let _ = timeline(&[Period {
            dates: "d",
            role: "r",
            org: "o",
            text: "x",
        }]);
        let _ = lists::rows_for_lint(); // doc_teasers, doc_list
        let _ = contact(&[ContactRow {
            label: "l",
            value: html! { "v" },
        }]);
        let _ = meta_line(html! { "m" });
        let _ = notice("n");
        let _ = accordion(&[(html! { "s" }, html! { "b" })]);
        let _ = prose(html! { p { "p" } });
        let _ = html! { div class=(section()) {} div class=(narrow_section()) {} div class=(split()) {} div class=(two_col()) {} div class=(muted()) {} };
        let css = record::finish().css();
        assert!(css.contains(".prose pre"), "named rule sets are recorded");
        assert_clean("components", &css);
    }
}
