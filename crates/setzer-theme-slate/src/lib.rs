//! "Slate": cool blue-grey, pairs with photos that have a cold bokeh.
//! Everything not set here comes from `Theme::default()` (8pt grid, system fonts).

use setzer_design::{Colors, Theme};

pub fn theme() -> Theme {
    Theme {
        light: Colors {
            bg: "#f5f7fa",
            surface: "#ffffff",
            fg: "#15202b",
            muted: "#5b6774",
            rule: "#dde3ea",
            accent: "#3b6ea5",
            accent_fg: "#2d5a8a",
            accent_soft: "#e3edf7",
            on_accent: "#ffffff",
            header_bg: "rgba(245,247,250,.88)",
            shadow_card: "0 8px 24px -18px rgba(0,0,0,.4)",
            ..Colors::light()
        },
        dark: Colors {
            bg: "#0f1620",
            surface: "#182130",
            fg: "#e6edf3",
            muted: "#9fb0c0",
            rule: "#243041",
            accent: "#6ea3d8",
            accent_fg: "#8ab8e6",
            accent_soft: "#1b2a3d",
            on_accent: "#0f1620",
            header_bg: "rgba(15,22,32,.88)",
            shadow_card: "0 8px 24px -18px rgba(0,0,0,.7)",
            ..Colors::dark()
        },
        ..Theme::default()
    }
}
