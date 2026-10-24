//! Theme values. `Theme::default()` is a complete 8pt-grid theme with a
//! neutral palette; a site replaces what it cares about with struct update
//! syntax. Literals are allowed in this module and in theme crates only.

use crate::tokens::*;

/// Length in CSS pixels; emitted as rem (÷16) so user font scaling works.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Px(pub f32);

impl std::fmt::Display for Px {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == 0.0 {
            write!(f, "0")
        } else {
            write!(f, "{}rem", trim(self.0 / 16.0))
        }
    }
}

fn trim(v: f32) -> String {
    let s = format!("{v:.4}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Spacing scale. "Gap tells the story": closer = belongs together.
#[derive(Clone, Debug)]
pub struct SpaceScale {
    pub tight: Px,
    pub xs: Px,
    pub s: Px,
    pub sm: Px,
    pub m: Px,
    pub l: Px,
    pub xl: Px,
    pub hero: Px,
    pub divider: Px,
}

impl SpaceScale {
    /// A grid with the given base unit: s = 1×, m = 2×, l = 3×, xl = 4×, hero = 6×, divider = 8×.
    pub fn grid(unit: f32) -> SpaceScale {
        SpaceScale {
            tight: Px(unit / 4.0),
            xs: Px(unit / 2.0),
            s: Px(unit),
            sm: Px(unit * 1.5),
            m: Px(unit * 2.0),
            l: Px(unit * 3.0),
            xl: Px(unit * 4.0),
            hero: Px(unit * 6.0),
            divider: Px(unit * 8.0),
        }
    }
    pub fn get(&self, t: Space) -> Px {
        match t {
            Space::Tight => self.tight,
            Space::Xs => self.xs,
            Space::S => self.s,
            Space::Sm => self.sm,
            Space::M => self.m,
            Space::L => self.l,
            Space::Xl => self.xl,
            Space::Hero => self.hero,
            Space::Divider => self.divider,
        }
    }
}

impl Default for SpaceScale {
    fn default() -> Self {
        SpaceScale::grid(8.0)
    }
}

#[derive(Clone, Debug)]
pub struct RadiusScale {
    pub xs: Px,
    pub s: Px,
    pub m: Px,
    pub l: Px,
    pub hero: Px,
    pub pill: Px,
}

impl RadiusScale {
    pub fn get(&self, t: Radius) -> Px {
        match t {
            Radius::Xs => self.xs,
            Radius::S => self.s,
            Radius::M => self.m,
            Radius::L => self.l,
            Radius::Hero => self.hero,
            Radius::Pill => self.pill,
        }
    }
    /// Everything square except pills. For "Swiss" looks.
    pub fn sharp() -> Self {
        RadiusScale {
            xs: Px(0.0),
            s: Px(0.0),
            m: Px(0.0),
            l: Px(0.0),
            hero: Px(0.0),
            pill: Px(0.0),
        }
    }
}

impl Default for RadiusScale {
    fn default() -> Self {
        RadiusScale {
            xs: Px(4.0),
            s: Px(8.0),
            m: Px(12.0),
            l: Px(14.0),
            hero: Px(16.0),
            pill: Px(999.0),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StrokeScale {
    pub hairline: Px,
    pub medium: Px,
    pub emphasized: Px,
    pub bold: Px,
}

impl StrokeScale {
    pub fn get(&self, t: Stroke) -> Px {
        match t {
            Stroke::Hairline => self.hairline,
            Stroke::Medium => self.medium,
            Stroke::Emphasized => self.emphasized,
            Stroke::Bold => self.bold,
        }
    }
}

impl Default for StrokeScale {
    fn default() -> Self {
        StrokeScale {
            hairline: Px(1.0),
            medium: Px(2.0),
            emphasized: Px(3.0),
            bold: Px(4.0),
        }
    }
}

/// Fixed sizes with a role (see `AppSizes` in the apps).
#[derive(Clone, Debug)]
pub struct Sizes {
    pub header: Px,
    pub portrait: Px,
    pub portrait_sm: Px,
    pub banner_strip: Px,
    pub hero_text: Px,
    pub banner_max: Px,
    pub date_col: Px,
    pub label_col: Px,
    pub year_col: Px,
    pub blur: Px,
    pub touch_min: Px,
}

impl Sizes {
    pub fn get(&self, t: Size) -> Px {
        match t {
            Size::Header => self.header,
            Size::Portrait => self.portrait,
            Size::PortraitSm => self.portrait_sm,
            Size::BannerStrip => self.banner_strip,
            Size::HeroText => self.hero_text,
            Size::BannerMax => self.banner_max,
            Size::DateCol => self.date_col,
            Size::LabelCol => self.label_col,
            Size::YearCol => self.year_col,
            Size::Blur => self.blur,
            Size::TouchMin => self.touch_min,
        }
    }
}

impl Default for Sizes {
    fn default() -> Self {
        Sizes {
            header: Px(72.0),
            portrait: Px(240.0),
            portrait_sm: Px(144.0),
            banner_strip: Px(240.0),
            hero_text: Px(544.0),
            banner_max: Px(608.0),
            date_col: Px(144.0),
            label_col: Px(112.0),
            year_col: Px(88.0),
            blur: Px(8.0),
            touch_min: Px(44.0),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Widths {
    pub wrap: Px,
    pub narrow: Px,
}

impl Widths {
    pub fn get(&self, t: Width) -> Px {
        match t {
            Width::Wrap => self.wrap,
            Width::Narrow => self.narrow,
        }
    }
}

impl Default for Widths {
    fn default() -> Self {
        Widths {
            wrap: Px(992.0),
            narrow: Px(704.0),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Fonts {
    pub body: &'static str,
    pub heading: &'static str,
    pub mono: &'static str,
}

impl Fonts {
    pub fn get(&self, f: Font) -> &'static str {
        match f {
            Font::Body => self.body,
            Font::Heading => self.heading,
            Font::Mono => self.mono,
        }
    }
}

pub const SYSTEM_SANS: &str =
    r#"-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif"#;
pub const SYSTEM_MONO: &str = "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace";

impl Default for Fonts {
    fn default() -> Self {
        Fonts {
            body: SYSTEM_SANS,
            heading: SYSTEM_SANS,
            mono: SYSTEM_MONO,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextStyle {
    pub font: Font,
    /// `Px` rendered, or a free-form CSS size such as `clamp(...)`.
    pub size: String,
    pub weight: u16,
    pub leading: f32,
    /// em
    pub tracking: f32,
    pub uppercase: bool,
}

impl TextStyle {
    pub fn new(font: Font, size_px: f32, weight: u16, leading: f32, tracking: f32) -> Self {
        TextStyle {
            font,
            size: Px(size_px).to_string(),
            weight,
            leading,
            tracking,
            uppercase: false,
        }
    }
    /// `clamp(min, vw, max)`
    pub fn fluid(mut self, min_px: f32, vw: f32, max_px: f32) -> Self {
        self.size = format!("clamp({}, {}vw, {})", Px(min_px), trim(vw), Px(max_px));
        self
    }
    pub fn uppercase(mut self) -> Self {
        self.uppercase = true;
        self
    }
}

/// One `TextStyle` per [`TextRole`].
#[derive(Clone, Debug)]
pub struct TextScale {
    pub display: TextStyle,
    pub h1: TextStyle,
    pub h2: TextStyle,
    pub h3: TextStyle,
    pub lead: TextStyle,
    pub body: TextStyle,
    pub body_sm: TextStyle,
    pub small: TextStyle,
    pub caption: TextStyle,
    pub overline: TextStyle,
    pub mono: TextStyle,
}

impl TextScale {
    pub fn get(&self, r: TextRole) -> &TextStyle {
        match r {
            TextRole::Display => &self.display,
            TextRole::H1 => &self.h1,
            TextRole::H2 => &self.h2,
            TextRole::H3 => &self.h3,
            TextRole::Lead => &self.lead,
            TextRole::Body => &self.body,
            TextRole::BodySm => &self.body_sm,
            TextRole::Small => &self.small,
            TextRole::Caption => &self.caption,
            TextRole::Overline => &self.overline,
            TextRole::Mono => &self.mono,
        }
    }
}

impl Default for TextScale {
    fn default() -> Self {
        use Font::*;
        TextScale {
            display: TextStyle::new(Heading, 56.0, 700, 1.05, -0.02),
            h1: TextStyle::new(Heading, 48.0, 700, 1.15, -0.01).fluid(32.0, 5.0, 48.0),
            h2: TextStyle::new(Heading, 27.0, 700, 1.15, -0.01),
            h3: TextStyle::new(Heading, 18.0, 600, 1.2, 0.0),
            lead: TextStyle::new(Body, 20.0, 400, 1.45, 0.0),
            body: TextStyle::new(Body, 17.0, 400, 1.55, 0.0),
            body_sm: TextStyle::new(Body, 15.0, 400, 1.5, 0.0),
            small: TextStyle::new(Body, 14.0, 400, 1.45, 0.0),
            caption: TextStyle::new(Body, 13.0, 400, 1.4, 0.0),
            overline: TextStyle::new(Body, 13.0, 600, 1.3, 0.1).uppercase(),
            mono: TextStyle::new(Mono, 15.0, 400, 1.5, 0.0),
        }
    }
}

/// Semantic colours for one mode. Fields are CSS colour values.
#[derive(Clone, Debug)]
pub struct Colors {
    pub bg: &'static str,
    pub surface: &'static str,
    pub fg: &'static str,
    pub muted: &'static str,
    pub rule: &'static str,
    pub accent: &'static str,
    pub accent_fg: &'static str,
    pub accent_soft: &'static str,
    pub on_accent: &'static str,
    pub header_bg: &'static str,
    pub shadow_card: &'static str,
    /// Over photos; usually the same in both modes.
    pub on_photo: &'static str,
    pub on_photo_muted: &'static str,
    pub ghost_fill: &'static str,
    pub ghost_border: &'static str,
    pub scrim_weak: &'static str,
    pub scrim_strong: &'static str,
    pub scrim_heavy: &'static str,
    pub scrim_none: &'static str,
    pub photo_backdrop: &'static str,
}

impl Colors {
    pub fn get(&self, c: Color) -> &'static str {
        match c {
            Color::Bg => self.bg,
            Color::Surface => self.surface,
            Color::Fg => self.fg,
            Color::Muted => self.muted,
            Color::Rule => self.rule,
            Color::Accent => self.accent,
            Color::AccentFg => self.accent_fg,
            Color::AccentSoft => self.accent_soft,
            Color::OnAccent => self.on_accent,
            Color::HeaderBg => self.header_bg,
            Color::OnPhoto => self.on_photo,
            Color::OnPhotoMuted => self.on_photo_muted,
            Color::GhostFill => self.ghost_fill,
            Color::GhostBorder => self.ghost_border,
            Color::ScrimWeak => self.scrim_weak,
            Color::ScrimStrong => self.scrim_strong,
            Color::ScrimHeavy => self.scrim_heavy,
            Color::ScrimNone => self.scrim_none,
            Color::PhotoBackdrop => self.photo_backdrop,
        }
    }

    /// Neutral greys, light mode. A starting point for `..Colors::light()`.
    pub fn light() -> Colors {
        Colors {
            bg: "#fafafa",
            surface: "#ffffff",
            fg: "#1a1a1a",
            muted: "#5c5c5c",
            rule: "#e2e2e2",
            accent: "#2f5d8a",
            accent_fg: "#244a6e",
            accent_soft: "#e4edf5",
            on_accent: "#ffffff",
            header_bg: "rgba(250,250,250,.88)",
            shadow_card: "0 8px 24px -18px rgba(0,0,0,.4)",
            on_photo: "#ffffff",
            on_photo_muted: "rgba(255,255,255,.82)",
            ghost_fill: "rgba(255,255,255,.1)",
            ghost_border: "rgba(255,255,255,.35)",
            scrim_weak: "rgba(14,20,28,.15)",
            scrim_strong: "rgba(14,20,28,.78)",
            scrim_heavy: "rgba(14,20,28,.88)",
            scrim_none: "rgba(14,20,28,0)",
            photo_backdrop: "#1a2430",
        }
    }

    /// Neutral greys, dark mode.
    pub fn dark() -> Colors {
        Colors {
            bg: "#141414",
            surface: "#1e1e1e",
            fg: "#ebebeb",
            muted: "#a3a3a3",
            rule: "#2e2e2e",
            accent: "#7fa9d4",
            accent_fg: "#95b9dd",
            accent_soft: "#1d2834",
            on_accent: "#141414",
            header_bg: "rgba(20,20,20,.88)",
            shadow_card: "0 8px 24px -18px rgba(0,0,0,.7)",
            ..Colors::light()
        }
    }
}

/// Everything a stylesheet needs to know about look. All fields public, all
/// have defaults.
#[derive(Clone, Debug)]
pub struct Theme {
    pub fonts: Fonts,
    pub space: SpaceScale,
    pub radius: RadiusScale,
    pub stroke: StrokeScale,
    pub size: Sizes,
    pub width: Widths,
    pub text: TextScale,
    pub light: Colors,
    pub dark: Colors,
    pub shadow_text: &'static str,
    pub motion_fast_ms: u16,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            fonts: Fonts::default(),
            space: SpaceScale::default(),
            radius: RadiusScale::default(),
            stroke: StrokeScale::default(),
            size: Sizes::default(),
            width: Widths::default(),
            text: TextScale::default(),
            light: Colors::light(),
            dark: Colors::dark(),
            shadow_text: "0 2px 12px rgba(0,0,0,.45)",
            motion_fast_ms: 160,
        }
    }
}

/// Breakpoints are layout, not look, so they are not part of a theme.
pub fn breakpoint_rem(b: Breakpoint) -> &'static str {
    match b {
        Breakpoint::Narrow => "48rem",
        Breakpoint::Phone => "30rem",
    }
}
