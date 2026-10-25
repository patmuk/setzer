//! Typed utilities. Each helper accepts only the token kind that fits and
//! returns a [`Utility`]: a class name plus the declarations behind it.
//! The same helpers serve two purposes:
//!
//! * in markup: `class=(style([pad(Space::M), bg(Color::Surface)]))` renders
//!   `class="p-m bg-surface"` and records the CSS those classes need;
//! * in named rules: `Css::rule(".prose pre", [pad(Space::M)])`.
//!
//! A raw number cannot be written. Keyword-only properties go through
//! [`raw`]; the stylesheet lint rejects lengths or colours smuggled in there.

use crate::css::Css;
use crate::tokens::*;

/// When a utility applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Variant {
    Always,
    Hover,
    /// `:first-child`
    First,
    /// In dark mode: the system's, unless the visitor picked light, or
    /// theirs from the theme toggle.
    Dark,
    /// At and below a breakpoint.
    At(Breakpoint),
}

#[derive(Clone, Debug)]
pub struct Utility {
    /// Utilities in the same group set the same property; when two meet,
    /// the later one wins (see `Style::with`).
    pub(crate) group: String,
    pub(crate) class: String,
    pub(crate) decls: String,
    pub(crate) variant: Variant,
    /// A named class: its CSS is a whole rule set, not `decls`.
    pub(crate) named: Option<fn() -> Css>,
}

impl PartialEq for Utility {
    fn eq(&self, other: &Self) -> bool {
        self.class == other.class
            && self.variant == other.variant
            && self.decls == other.decls
            && self.named.is_some() == other.named.is_some()
    }
}
impl Eq for Utility {}

impl Utility {
    pub fn new(group: &str, class: impl Into<String>, decls: impl Into<String>) -> Utility {
        Utility {
            group: group.into(),
            class: class.into(),
            decls: decls.into(),
            variant: Variant::Always,
            named: None,
        }
    }
    /// The declarations, `property: value; …`.
    pub fn decls(&self) -> &str {
        &self.decls
    }
    /// The class name as written in HTML, variant prefix included.
    pub fn class(&self) -> String {
        match self.variant {
            Variant::Always => self.class.clone(),
            Variant::Hover => format!("hover:{}", self.class),
            Variant::First => format!("first:{}", self.class),
            Variant::Dark => format!("dark:{}", self.class),
            Variant::At(b) => format!("{}:{}", b.slug(), self.class),
        }
    }
    pub(crate) fn key(&self) -> (Variant, String) {
        (
            self.variant,
            if self.named.is_some() {
                self.class.clone()
            } else {
                self.group.clone()
            },
        )
    }
    /// The CSS rule for this utility (empty for named classes).
    pub(crate) fn rule(&self) -> String {
        if self.named.is_some() {
            return String::new();
        }
        let selector = format!(".{}", escape(&self.class()));
        match self.variant {
            Variant::Always => format!("{selector} {{ {} }}\n", self.decls),
            Variant::Hover => format!("{selector}:hover {{ {} }}\n", self.decls),
            Variant::First => format!("{selector}:first-child {{ {} }}\n", self.decls),
            Variant::Dark => format!(
                "@media (prefers-color-scheme: dark) {{ :root:not([data-theme=\"light\"]) {selector} {{ {d} }} }}\n\
                 :root[data-theme=\"dark\"] {selector} {{ {d} }}\n",
                d = self.decls
            ),
            Variant::At(b) => format!("{b} {{ {selector} {{ {} }} }}\n", self.decls),
        }
    }
}

fn escape(class: &str) -> String {
    class.replace(':', "\\:")
}

/// Keeps `[a-z0-9-]` so any value can become part of a class name.
fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

// ---- variants ---------------------------------------------------------------

pub fn hover(mut u: Utility) -> Utility {
    u.variant = Variant::Hover;
    u
}
pub fn first(mut u: Utility) -> Utility {
    u.variant = Variant::First;
    u
}
/// Applies in dark mode, following the theme toggle like the colours do;
/// e.g. a logo drawn for dark backgrounds: `[hidden(), dark(block())]`.
pub fn dark(mut u: Utility) -> Utility {
    u.variant = Variant::Dark;
    u
}
/// Applies at and below `b` (desktop first).
pub fn at(b: Breakpoint, mut u: Utility) -> Utility {
    u.variant = Variant::At(b);
    u
}

/// A named class whose CSS is a whole rule set: for styling that cannot sit
/// on one element (markdown output, pseudo-elements, parent-dependent
/// colours). `css` is written with `Css::rule` and recorded when used.
pub fn named(class: &'static str, css: fn() -> Css) -> Utility {
    Utility {
        group: class.into(),
        class: class.into(),
        decls: String::new(),
        variant: Variant::Always,
        named: Some(css),
    }
}

/// A bare class name, styled by a parent's named rule set
/// (e.g. `class("hero-lead")` inside `named("hero-banner", …)`).
pub fn class(name: &'static str) -> Utility {
    named(name, Css::new)
}

/// Keyword-only properties: `raw("object-fit", "cover")`.
pub fn raw(property: &str, keyword: &str) -> Utility {
    Utility::new(
        property,
        format!("{}-{}", slug(property), slug(keyword)),
        format!("{property}: {keyword}"),
    )
}

// ---- box ----------------------------------------------------------------------

pub fn pad(s: Space) -> Utility {
    Utility::new(
        "padding",
        format!("p-{}", s.slug()),
        format!("padding: {s}"),
    )
}
pub fn pad_x(s: Space) -> Utility {
    Utility::new(
        "padding-inline",
        format!("px-{}", s.slug()),
        format!("padding-inline: {s}"),
    )
}
pub fn pad_y(s: Space) -> Utility {
    Utility::new(
        "padding-block",
        format!("py-{}", s.slug()),
        format!("padding-block: {s}"),
    )
}
pub fn pad_xy(y: Space, x: Space) -> Utility {
    Utility::new(
        "padding",
        format!("p-{}-{}", y.slug(), x.slug()),
        format!("padding: {y} {x}"),
    )
}
pub fn pad_top(s: Space) -> Utility {
    Utility::new(
        "padding-top",
        format!("pt-{}", s.slug()),
        format!("padding-top: {s}"),
    )
}
pub fn pad_bottom(s: Space) -> Utility {
    Utility::new(
        "padding-bottom",
        format!("pb-{}", s.slug()),
        format!("padding-bottom: {s}"),
    )
}
pub fn pad_left(s: Space) -> Utility {
    Utility::new(
        "padding-left",
        format!("pl-{}", s.slug()),
        format!("padding-left: {s}"),
    )
}
pub fn pad_none() -> Utility {
    Utility::new("padding", "p-0", "padding: 0")
}
pub fn margin_none() -> Utility {
    Utility::new("margin", "m-0", "margin: 0")
}
pub fn margin_top(s: Space) -> Utility {
    Utility::new(
        "margin-top",
        format!("mt-{}", s.slug()),
        format!("margin-top: {s}"),
    )
}
pub fn margin_top_none() -> Utility {
    Utility::new("margin-top", "mt-0", "margin-top: 0")
}
/// Negative top margin, e.g. to overlap the band above.
pub fn margin_top_neg(s: Space) -> Utility {
    Utility::new(
        "margin-top",
        format!("-mt-{}", s.slug()),
        format!("margin-top: calc(-1 * {s})"),
    )
}
pub fn margin_bottom(s: Space) -> Utility {
    Utility::new(
        "margin-bottom",
        format!("mb-{}", s.slug()),
        format!("margin-bottom: {s}"),
    )
}
pub fn margin_bottom_none() -> Utility {
    Utility::new("margin-bottom", "mb-0", "margin-bottom: 0")
}
pub fn margin_y(s: Space) -> Utility {
    Utility::new(
        "margin-block",
        format!("my-{}", s.slug()),
        format!("margin-block: {s}"),
    )
}
pub fn margin_x_auto() -> Utility {
    Utility::new("margin-inline", "mx-auto", "margin-inline: auto")
}
pub fn gap(s: Space) -> Utility {
    Utility::new("gap", format!("gap-{}", s.slug()), format!("gap: {s}"))
}
pub fn gap_xy(row: Space, col: Space) -> Utility {
    Utility::new(
        "gap",
        format!("gap-{}-{}", row.slug(), col.slug()),
        format!("gap: {row} {col}"),
    )
}
pub fn gap_none() -> Utility {
    Utility::new("gap", "gap-0", "gap: 0")
}

// ---- size ---------------------------------------------------------------------

pub fn width(v: impl Length) -> Utility {
    Utility::new("width", format!("w-{}", v.name()), format!("width: {v}"))
}
pub fn height(v: impl Length) -> Utility {
    Utility::new("height", format!("h-{}", v.name()), format!("height: {v}"))
}
pub fn square(v: impl Length) -> Utility {
    Utility::new(
        "size",
        format!("size-{}", v.name()),
        format!("width: {v}; height: {v}"),
    )
}
pub fn max_width(v: impl Length) -> Utility {
    Utility::new(
        "max-width",
        format!("max-w-{}", v.name()),
        format!("max-width: {v}"),
    )
}
pub fn max_height(v: impl Length) -> Utility {
    Utility::new(
        "max-height",
        format!("max-h-{}", v.name()),
        format!("max-height: {v}"),
    )
}
pub fn min_height(v: impl Length) -> Utility {
    Utility::new(
        "min-height",
        format!("min-h-{}", v.name()),
        format!("min-height: {v}"),
    )
}
pub fn full_width() -> Utility {
    Utility::new("width", "w-full", "width: 100%")
}
pub fn full_height() -> Utility {
    Utility::new("height", "h-full", "height: 100%")
}
/// A centred column: `min(100% - 2·gutter, max)` wide, auto side margins.
pub fn column(gutter: Space, max: Width) -> Utility {
    Utility::new(
        "column",
        format!("column-{}-{}", gutter.slug(), max.slug()),
        format!("width: min(100% - 2 * {gutter}, {max}); margin-inline: auto"),
    )
}

// ---- border, radius, shadow -----------------------------------------------------

pub fn border(w: Stroke, c: Color) -> Utility {
    Utility::new(
        "border",
        format!("border-{}-{}", w.slug(), c.slug()),
        format!("border: {w} solid {c}"),
    )
}
pub fn border_top(w: Stroke, c: Color) -> Utility {
    Utility::new(
        "border-top",
        format!("border-t-{}-{}", w.slug(), c.slug()),
        format!("border-top: {w} solid {c}"),
    )
}
pub fn border_bottom(w: Stroke, c: Color) -> Utility {
    Utility::new(
        "border-bottom",
        format!("border-b-{}-{}", w.slug(), c.slug()),
        format!("border-bottom: {w} solid {c}"),
    )
}
pub fn border_left(w: Stroke, c: Color) -> Utility {
    Utility::new(
        "border-left",
        format!("border-l-{}-{}", w.slug(), c.slug()),
        format!("border-left: {w} solid {c}"),
    )
}
pub fn border_none() -> Utility {
    Utility::new("border", "border-0", "border: 0")
}
pub fn border_top_none() -> Utility {
    Utility::new("border-top", "border-t-0", "border-top: 0")
}
pub fn border_color(c: Color) -> Utility {
    Utility::new(
        "border-color",
        format!("border-{}", c.slug()),
        format!("border-color: {c}"),
    )
}
pub fn radius(r: Radius) -> Utility {
    Utility::new(
        "border-radius",
        format!("rounded-{}", r.slug()),
        format!("border-radius: {r}"),
    )
}
pub fn shadow(s: Shadow) -> Utility {
    Utility::new(
        "box-shadow",
        format!("shadow-{}", s.slug()),
        format!("box-shadow: {s}"),
    )
}
pub fn text_shadow(s: Shadow) -> Utility {
    Utility::new(
        "text-shadow",
        format!("text-shadow-{}", s.slug()),
        format!("text-shadow: {s}"),
    )
}
pub fn no_text_shadow() -> Utility {
    Utility::new("text-shadow", "text-shadow-none", "text-shadow: none")
}
pub fn underline_offset(w: Stroke) -> Utility {
    Utility::new(
        "text-underline-offset",
        format!("underline-offset-{}", w.slug()),
        format!("text-underline-offset: {w}"),
    )
}

// ---- colour -------------------------------------------------------------------

pub fn bg(c: Color) -> Utility {
    Utility::new(
        "background",
        format!("bg-{}", c.slug()),
        format!("background: {c}"),
    )
}
pub fn bg_none() -> Utility {
    Utility::new("background", "bg-none", "background: none")
}
pub fn fg(c: Color) -> Utility {
    Utility::new("color", format!("text-{}", c.slug()), format!("color: {c}"))
}
/// Linear gradient between colour tokens, stops in percent.
pub fn gradient(angle_deg: u16, stops: &[(Color, u8)]) -> Utility {
    let css: Vec<String> = stops.iter().map(|(c, p)| format!("{c} {p}%")).collect();
    let name: Vec<String> = stops
        .iter()
        .map(|(c, p)| format!("{}-{p}", c.slug()))
        .collect();
    Utility::new(
        "background",
        format!("bg-grad-{angle_deg}-{}", name.join("-")),
        format!(
            "background: linear-gradient({angle_deg}deg, {})",
            css.join(", ")
        ),
    )
}

// ---- text ---------------------------------------------------------------------

pub fn text(r: TextRole) -> Utility {
    Utility::new(
        "font",
        format!("text-{}", r.slug()),
        r.to_string().trim_end_matches("; ").to_string(),
    )
}
pub fn weight(w: u16) -> Utility {
    Utility::new(
        "font-weight",
        format!("font-{w}"),
        format!("font-weight: {w}"),
    )
}
pub fn font(f: Font) -> Utility {
    Utility::new(
        "font-family",
        format!("font-{}", f.slug()),
        format!("font-family: {f}"),
    )
}
pub fn font_inherit() -> Utility {
    Utility::new("font", "font-inherit", "font: inherit")
}
pub fn no_underline() -> Utility {
    Utility::new("text-decoration", "no-underline", "text-decoration: none")
}
pub fn underline() -> Utility {
    Utility::new("text-decoration", "underline", "text-decoration: underline")
}
pub fn tabular_nums() -> Utility {
    Utility::new(
        "font-variant-numeric",
        "tabular-nums",
        "font-variant-numeric: tabular-nums",
    )
}

// ---- layout -------------------------------------------------------------------

pub fn flex() -> Utility {
    Utility::new("display", "flex", "display: flex")
}
pub fn grid() -> Utility {
    Utility::new("display", "grid", "display: grid")
}
pub fn block() -> Utility {
    Utility::new("display", "block", "display: block")
}
pub fn inline_block() -> Utility {
    Utility::new("display", "inline-block", "display: inline-block")
}
pub fn hidden() -> Utility {
    Utility::new("display", "hidden", "display: none")
}
pub fn flex_col() -> Utility {
    Utility::new("flex-direction", "flex-col", "flex-direction: column")
}
pub fn flex_wrap() -> Utility {
    Utility::new("flex-wrap", "flex-wrap", "flex-wrap: wrap")
}
pub fn items_center() -> Utility {
    Utility::new("align-items", "items-center", "align-items: center")
}
/// Moves a flex or grid item after its siblings.
pub fn order_last() -> Utility {
    Utility::new("order", "order-last", "order: 1")
}
pub fn items_start() -> Utility {
    Utility::new("align-items", "items-start", "align-items: start")
}
pub fn justify_between() -> Utility {
    Utility::new(
        "justify-content",
        "justify-between",
        "justify-content: space-between",
    )
}
pub fn list_none() -> Utility {
    Utility::new("list-style", "list-none", "list-style: none")
}
/// Equal columns.
pub fn columns(n: u8) -> Utility {
    Utility::new(
        "grid-template-columns",
        format!("cols-{n}"),
        format!("grid-template-columns: repeat({n}, 1fr)"),
    )
}
/// `fr` ratios, e.g. `&[3, 2]`.
pub fn columns_fr(ratios: &[u8]) -> Utility {
    let css: Vec<String> = ratios.iter().map(|r| format!("{r}fr")).collect();
    let name: Vec<String> = ratios.iter().map(|r| r.to_string()).collect();
    Utility::new(
        "grid-template-columns",
        format!("cols-fr-{}", name.join("-")),
        format!("grid-template-columns: {}", css.join(" ")),
    )
}
/// A fixed first column, the rest flexible.
pub fn columns_fixed_first(first: impl Length) -> Utility {
    Utility::new(
        "grid-template-columns",
        format!("cols-first-{}", first.name()),
        format!("grid-template-columns: {first} 1fr"),
    )
}
/// A capped first column, the rest flexible.
pub fn columns_capped_first(max: impl Length) -> Utility {
    Utility::new(
        "grid-template-columns",
        format!("cols-capped-{}", max.name()),
        format!("grid-template-columns: minmax(0, {max}) 1fr"),
    )
}
/// Flexible first column, the last sized by its content.
pub fn columns_content_first() -> Utility {
    Utility::new(
        "grid-template-columns",
        "cols-content-first",
        "grid-template-columns: auto minmax(0, 1fr)",
    )
}
pub fn columns_content_last() -> Utility {
    Utility::new(
        "grid-template-columns",
        "cols-content-last",
        "grid-template-columns: minmax(0, 1fr) auto",
    )
}
pub fn top(s: Space) -> Utility {
    Utility::new("top", format!("top-{}", s.slug()), format!("top: {s}"))
}
pub fn left(s: Space) -> Utility {
    Utility::new("left", format!("left-{}", s.slug()), format!("left: {s}"))
}
pub fn sticky_top() -> Utility {
    Utility::new("position", "sticky-top", "position: sticky; top: 0")
}
pub fn relative() -> Utility {
    Utility::new("position", "relative", "position: relative")
}
pub fn layer(z: u8) -> Utility {
    Utility::new("z-index", format!("z-{z}"), format!("z-index: {z}"))
}
pub fn blur(v: Size) -> Utility {
    Utility::new(
        "backdrop-filter",
        format!("backdrop-blur-{}", v.slug()),
        format!("backdrop-filter: blur({v})"),
    )
}
pub fn pointer() -> Utility {
    Utility::new("cursor", "cursor-pointer", "cursor: pointer")
}
pub fn transition(m: Motion) -> Utility {
    Utility::new(
        "transition",
        format!("transition-{}", m.slug()),
        format!("transition: all {m} ease"),
    )
}
pub fn scroll_padding_top(v: Size) -> Utility {
    Utility::new(
        "scroll-padding-top",
        format!("scroll-pt-{}", v.slug()),
        format!("scroll-padding-top: {v}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_names_and_rules() {
        assert_eq!(pad(Space::M).class(), "p-m");
        assert_eq!(at(Breakpoint::Narrow, columns(1)).class(), "narrow:cols-1");
        assert_eq!(
            hover(border_color(Color::Accent)).rule(),
            ".hover\\:border-accent:hover { border-color: var(--color-accent) }\n"
        );
        assert_eq!(raw("object-fit", "cover").class(), "object-fit-cover");
        assert_eq!(
            dark(hidden()).rule(),
            "@media (prefers-color-scheme: dark) { :root:not([data-theme=\"light\"]) .dark\\:hidden { display: none } }\n\
             :root[data-theme=\"dark\"] .dark\\:hidden { display: none }\n"
        );
    }
}
