//! Named rule sets, for styling a utility on one element cannot express:
//! markdown output (`.prose pre`), pseudo-elements (`::before`), styling
//! that depends on a parent (`.banner h1`). Declarations are the same typed
//! utilities as in markup.
//!
//! ```ignore
//! rule(".prose pre", [bg(Color::Surface), pad(Space::M)])
//!     .rule(".prose blockquote", [border_left(Stroke::Emphasized, Color::Accent)])
//!     .at(Breakpoint::Phone, |c| c.rule(".prose pre", [pad(Space::S)]))
//! ```

use crate::tokens::Breakpoint;
use crate::utility::Utility;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Item {
    Rule {
        selector: String,
        decls: Vec<String>,
    },
    Media {
        prelude: String,
        rules: Css,
    },
    Raw(String),
}

/// An ordered list of rules.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Css(Vec<Item>);

/// Starts a rule set with its first rule.
pub fn rule(selector: &str, decls: impl IntoIterator<Item = Utility>) -> Css {
    Css::new().rule(selector, decls)
}

impl Css {
    pub fn new() -> Css {
        Css::default()
    }

    /// Adds a rule. Only the utilities' declarations are used; variants and
    /// named classes have no meaning inside a rule.
    pub fn rule(mut self, selector: &str, decls: impl IntoIterator<Item = Utility>) -> Css {
        let decls = decls
            .into_iter()
            .map(|u| u.decls().to_string())
            .filter(|d| !d.is_empty())
            .collect();
        self.0.push(Item::Rule {
            selector: selector.to_string(),
            decls,
        });
        self
    }

    /// Rules that apply at and below a breakpoint.
    pub fn at(mut self, bp: Breakpoint, rules: impl FnOnce(Css) -> Css) -> Css {
        self.0.push(Item::Media {
            prelude: bp.to_string(),
            rules: rules(Css::new()),
        });
        self
    }

    /// Appends another rule set.
    pub fn include(mut self, other: Css) -> Css {
        self.0.extend(other.0);
        self
    }

    /// Verbatim CSS, for the theme's token block.
    pub fn raw(text: impl Into<String>) -> Css {
        Css(vec![Item::Raw(text.into())])
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn write(&self, f: &mut fmt::Formatter<'_>, indent: &str) -> fmt::Result {
        for item in &self.0 {
            match item {
                Item::Rule { selector, decls } => {
                    writeln!(f, "{indent}{selector} {{ {} }}", decls.join("; "))?
                }
                Item::Media { prelude, rules } => {
                    writeln!(f, "{indent}{prelude} {{")?;
                    rules.write(f, &format!("{indent}  "))?;
                    writeln!(f, "{indent}}}")?;
                }
                Item::Raw(text) => writeln!(f, "{text}")?,
            }
        }
        Ok(())
    }
}

impl fmt::Display for Css {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(f, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::*;
    use crate::utility::*;

    #[test]
    fn renders_rules_and_media() {
        let css = rule(".card", [pad(Space::M), radius(Radius::M)])
            .at(Breakpoint::Narrow, |c| c.rule(".card", [pad(Space::S)]));
        let s = css.to_string();
        assert!(s.contains(".card { padding: var(--space-m); border-radius: var(--radius-m) }"));
        assert!(s.contains("@media (max-width: 48rem) {\n  .card { padding: var(--space-s) }"));
    }
}
