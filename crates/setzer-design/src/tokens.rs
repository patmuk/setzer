//! Token vocabulary. Everything renders to a `var(--…)` reference, so
//! component CSS never carries a value.

use std::fmt;

macro_rules! token_enum {
    ($(#[$doc:meta])* $name:ident, $prefix:literal, { $($variant:ident => $slug:literal),+ $(,)? }) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum $name { $($variant),+ }
        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),+];
            /// CSS custom property name, e.g. `--space-m`.
            pub fn prop(self) -> String {
                format!("--{}-{}", $prefix, self.slug())
            }
            pub fn slug(self) -> &'static str {
                match self { $($name::$variant => $slug),+ }
            }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "var({})", self.prop())
            }
        }
    };
}

token_enum!(
    /// 8pt spacing grid. "Gap tells the story": closer = belongs together.
    Space, "space", {
        Tight => "tight", Xs => "xs", S => "s", Sm => "sm", M => "m", L => "l", Xl => "xl", Hero => "hero", Divider => "divider",
    }
);

token_enum!(
    /// Corner radii. One family per site; mixing looks accidental.
    Radius, "radius", { Xs => "xs", S => "s", M => "m", L => "l", Hero => "hero", Pill => "pill" }
);

token_enum!(
    /// Border / outline widths.
    Stroke, "stroke", { Hairline => "hairline", Medium => "medium", Emphasized => "emphasized", Bold => "bold" }
);

token_enum!(
    /// Semantic colours. Values differ between light and dark.
    Color, "color", {
        Bg => "bg", Surface => "surface", Fg => "fg", Muted => "muted", Rule => "rule",
        Accent => "accent", AccentFg => "accent-fg", AccentSoft => "accent-soft", OnAccent => "on-accent",
        // Text over photos; same in both modes.
        OnPhoto => "on-photo", OnPhotoMuted => "on-photo-muted",
        // Translucent fills over photos.
        GhostFill => "ghost-fill", GhostBorder => "ghost-border",
        // Dark overlays for text legibility over photos.
        ScrimWeak => "scrim-weak", ScrimStrong => "scrim-strong", ScrimHeavy => "scrim-heavy", ScrimNone => "scrim-none",
        // Fallback behind a photo while it loads.
        PhotoBackdrop => "photo-backdrop",
        // Header background, translucent over content.
        HeaderBg => "header-bg",
    }
);

token_enum!(
    /// Font stacks.
    Font, "font", { Body => "body", Heading => "heading", Mono => "mono" }
);

token_enum!(
    /// Fixed sizes with a role, like `AppSizes` in the apps.
    Size, "size", {
        Header => "header", Portrait => "portrait", PortraitSm => "portrait-sm", BannerStrip => "banner-strip",
        HeroText => "hero-text", BannerMax => "banner-max", DateCol => "date-col", LabelCol => "label-col", YearCol => "year-col",
        Blur => "blur", TouchMin => "touch-min",
    }
);

token_enum!(
    /// Container widths.
    Width, "width", { Wrap => "wrap", Narrow => "narrow" }
);

token_enum!(
    Shadow, "shadow", { Card => "card", Text => "text" }
);

token_enum!(
    Motion, "motion", { Fast => "fast" }
);

/// Named text roles. Each expands to the full set of declarations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextRole {
    Display,
    H1,
    H2,
    H3,
    Lead,
    Body,
    BodySm,
    Small,
    Caption,
    Overline,
    Mono,
}

impl TextRole {
    pub const ALL: &'static [TextRole] = &[
        TextRole::Display,
        TextRole::H1,
        TextRole::H2,
        TextRole::H3,
        TextRole::Lead,
        TextRole::Body,
        TextRole::BodySm,
        TextRole::Small,
        TextRole::Caption,
        TextRole::Overline,
        TextRole::Mono,
    ];
    pub fn slug(self) -> &'static str {
        match self {
            TextRole::Display => "display",
            TextRole::H1 => "h1",
            TextRole::H2 => "h2",
            TextRole::H3 => "h3",
            TextRole::Lead => "lead",
            TextRole::Body => "body",
            TextRole::BodySm => "body-sm",
            TextRole::Small => "small",
            TextRole::Caption => "caption",
            TextRole::Overline => "overline",
            TextRole::Mono => "mono",
        }
    }
    pub fn prop(self, part: &str) -> String {
        format!("--text-{}-{part}", self.slug())
    }
}

/// Expands to `font-family; font-size; font-weight; line-height; letter-spacing; text-transform`.
impl fmt::Display for TextRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for part in [
            "family",
            "size",
            "weight",
            "leading",
            "tracking",
            "transform",
        ] {
            let decl = match part {
                "family" => "font-family",
                "size" => "font-size",
                "weight" => "font-weight",
                "leading" => "line-height",
                "tracking" => "letter-spacing",
                _ => "text-transform",
            };
            write!(f, "{decl}: var({}); ", self.prop(part))?;
        }
        Ok(())
    }
}

/// Media-query preludes. CSS variables cannot be used inside `@media`, so
/// breakpoints are the one place a literal is allowed. Desktop first: a
/// breakpoint applies at and below its width; later ones win.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Breakpoint {
    /// Tablets and below: single column layouts.
    Narrow,
    /// Phones.
    Phone,
}

impl Breakpoint {
    pub fn slug(self) -> &'static str {
        match self {
            Breakpoint::Narrow => "narrow",
            Breakpoint::Phone => "phone",
        }
    }
}

/// Resolved by [`super::theme::Theme::media`]; components call `t.media(Breakpoint::Narrow)`
/// only through the `MEDIA_*` constants exported by the theme at emit time.
impl fmt::Display for Breakpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "@media (max-width: {})",
            super::theme::breakpoint_rem(*self)
        )
    }
}

/// Tokens that are lengths and may be used wherever CSS wants a length.
pub trait Length: fmt::Display + Copy {
    /// Short name used in utility class names (`m`, `portrait`, `wrap`).
    fn name(self) -> &'static str;
}
impl Length for Space {
    fn name(self) -> &'static str {
        self.slug()
    }
}
impl Length for Size {
    fn name(self) -> &'static str {
        self.slug()
    }
}
impl Length for Width {
    fn name(self) -> &'static str {
        self.slug()
    }
}
