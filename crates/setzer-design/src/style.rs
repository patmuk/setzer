//! `style([...])`: the value you put into a `class` attribute. Rendering it
//! writes the class names and records the CSS they need, so the site's
//! stylesheet contains exactly the utilities and named classes its pages use.

use crate::css::Css;
use crate::tokens::Breakpoint;
use crate::utility::{Utility, Variant};
use std::cell::RefCell;
use std::collections::BTreeMap;

/// A set of utilities for one element. Two utilities for the same property
/// and variant never coexist: the later one replaces the earlier.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Style(Vec<Utility>);

/// The class list of one element: `class=(style([pad(Space::M), bg(Color::Surface)]))`.
pub fn style(utilities: impl IntoIterator<Item = Utility>) -> Style {
    Style::default().with(Style(utilities.into_iter().collect()))
}

impl Style {
    /// Adds `other` on top: for the same property and variant, `other` wins.
    /// This is how a caller overrides a component's look.
    pub fn with(mut self, other: impl Into<Style>) -> Style {
        for u in other.into().0 {
            let key = u.key();
            self.0.retain(|e| e.key() != key);
            self.0.push(u);
        }
        self
    }
    pub fn classes(&self) -> String {
        self.0
            .iter()
            .map(Utility::class)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl From<Utility> for Style {
    fn from(u: Utility) -> Style {
        Style(vec![u])
    }
}

impl maud::Render for Style {
    fn render_to(&self, buffer: &mut String) {
        for u in &self.0 {
            record::utility(u);
        }
        buffer.push_str(&self.classes());
    }
}

/// Collects the CSS of what was rendered. A site calls `start` before
/// rendering its pages and `finish` afterwards.
pub mod record {
    use super::*;

    thread_local! {
        static RECORDED: RefCell<Option<Recorded>> = const { RefCell::new(None) };
    }

    #[derive(Default, Debug)]
    pub struct Recorded {
        utilities: BTreeMap<(Variant, String), Utility>,
        named: BTreeMap<String, String>,
    }

    pub fn start() {
        RECORDED.with(|r| *r.borrow_mut() = Some(Recorded::default()));
    }

    pub fn finish() -> Recorded {
        RECORDED.with(|r| r.borrow_mut().take()).unwrap_or_default()
    }

    /// Records a utility (or a named class's rule set). Outside of
    /// `start`/`finish` this does nothing.
    pub fn utility(u: &Utility) {
        RECORDED.with(|r| {
            if let Some(rec) = r.borrow_mut().as_mut() {
                match u.named {
                    Some(css) => {
                        rec.named
                            .entry(u.class.clone())
                            .or_insert_with(|| css().to_string());
                    }
                    None => {
                        rec.utilities
                            .entry((u.variant, u.class()))
                            .or_insert_with(|| u.clone());
                    }
                }
            }
        });
    }

    /// Records a rule set under `key` once, e.g. a component's named class.
    pub fn css(key: &str, css: impl FnOnce() -> Css) {
        RECORDED.with(|r| {
            if let Some(rec) = r.borrow_mut().as_mut() {
                rec.named
                    .entry(key.to_string())
                    .or_insert_with(|| css().to_string());
            }
        });
    }

    impl Recorded {
        /// In cascade order, like Tailwind's layers: named classes
        /// (components), then utilities, hover and first-child variants,
        /// then breakpoints from wide to narrow.
        pub fn css(&self) -> String {
            let mut out = String::new();
            for css in self.named.values() {
                out.push_str(css);
            }
            for v in [Variant::Always, Variant::Hover, Variant::First] {
                for ((variant, _), u) in &self.utilities {
                    if *variant == v {
                        out.push_str(&u.rule());
                    }
                }
            }
            for bp in [Breakpoint::Narrow, Breakpoint::Phone] {
                let rules: Vec<String> = self
                    .utilities
                    .iter()
                    .filter(|((v, _), _)| *v == Variant::At(bp))
                    .map(|(_, u)| {
                        let rule = u.rule();
                        // strip the per-rule @media wrapper; one block per breakpoint
                        let inner = rule
                            .split_once("{ ")
                            .map(|(_, r)| r.trim_end().trim_end_matches('}').trim_end().to_string())
                            .unwrap_or_default();
                        format!("  {inner}\n")
                    })
                    .collect();
                if !rules.is_empty() {
                    out.push_str(&format!("{bp} {{\n{}}}\n", rules.concat()));
                }
            }
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::*;
    use crate::utility::*;
    use maud::Render;

    #[test]
    fn later_utility_for_same_property_wins() {
        let s = style([pad(Space::M), bg(Color::Surface)]).with(style([pad(Space::S)]));
        assert_eq!(s.classes(), "bg-surface p-s");
    }

    #[test]
    fn rendering_records_css_once() {
        record::start();
        let s = style([pad(Space::M), at(Breakpoint::Narrow, columns(1))]);
        let _ = s.render();
        let _ = s.render();
        let css = record::finish().css();
        assert_eq!(css.matches(".p-m {").count(), 1);
        assert!(css.contains("@media (max-width: 48rem) {\n  .narrow\\:cols-1 { grid-template-columns: repeat(1, 1fr) }\n}"));
    }
}
