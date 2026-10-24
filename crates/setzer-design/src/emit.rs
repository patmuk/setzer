//! Theme → CSS custom properties.

use crate::theme::{Colors, Theme};
use crate::tokens::*;

/// `:root` block with light values, dark values under `prefers-color-scheme`
/// and `[data-theme]` overrides for a user toggle.
pub fn tokens_css(t: &Theme) -> String {
    let mut s = String::from(":root {\n");
    for &f in [Font::Body, Font::Heading, Font::Mono].iter() {
        s.push_str(&format!("  {}: {};\n", f.prop(), t.fonts.get(f)));
    }
    for &k in Space::ALL {
        s.push_str(&format!("  {}: {};\n", k.prop(), t.space.get(k)));
    }
    for &k in Radius::ALL {
        s.push_str(&format!("  {}: {};\n", k.prop(), t.radius.get(k)));
    }
    for &k in Stroke::ALL {
        s.push_str(&format!("  {}: {};\n", k.prop(), t.stroke.get(k)));
    }
    for &k in Size::ALL {
        s.push_str(&format!("  {}: {};\n", k.prop(), t.size.get(k)));
    }
    for &k in Width::ALL {
        s.push_str(&format!("  {}: {};\n", k.prop(), t.width.get(k)));
    }
    for &role in TextRole::ALL {
        let st = t.text.get(role);
        s.push_str(&format!(
            "  {}: {}; {}: {}; {}: {}; {}: {}; {}: {}em; {}: {};\n",
            role.prop("family"),
            st.font,
            role.prop("size"),
            st.size,
            role.prop("weight"),
            st.weight,
            role.prop("leading"),
            st.leading,
            role.prop("tracking"),
            st.tracking,
            role.prop("transform"),
            if st.uppercase { "uppercase" } else { "none" },
        ));
    }
    s.push_str(&format!("  {}: {};\n", Shadow::Text.prop(), t.shadow_text));
    s.push_str(&format!(
        "  {}: {}ms;\n",
        Motion::Fast.prop(),
        t.motion_fast_ms
    ));
    s.push_str(&colors(&t.light));
    s.push_str("  color-scheme: light dark;\n}\n");
    s.push_str("@media (prefers-color-scheme: dark) {\n  :root:not([data-theme=\"light\"]) {\n");
    s.push_str(&indent(&colors(&t.dark)));
    s.push_str("  }\n}\n:root[data-theme=\"dark\"] {\n");
    s.push_str(&colors(&t.dark));
    s.push_str("  color-scheme: dark;\n}\n:root[data-theme=\"light\"] { color-scheme: light; }\n");
    s
}

fn colors(set: &Colors) -> String {
    let mut s = String::new();
    for &c in Color::ALL {
        s.push_str(&format!("  {}: {};\n", c.prop(), set.get(c)));
    }
    s.push_str(&format!(
        "  {}: {};\n",
        Shadow::Card.prop(),
        set.shadow_card
    ));
    s
}

fn indent(s: &str) -> String {
    s.lines().map(|l| format!("  {l}\n")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_token_gets_a_value() {
        let css = tokens_css(&Theme::default());
        for p in Space::ALL.iter().map(|t| t.prop()) {
            assert!(css.contains(&format!("{p}:")), "{p}");
        }
        for c in Color::ALL {
            assert!(css.contains(&format!("{}:", c.prop())), "{c:?}");
        }
        for r in TextRole::ALL {
            assert!(css.contains(&r.prop("size")), "{r:?}");
        }
        for s in Size::ALL {
            assert!(css.contains(&format!("{}:", s.prop())), "{s:?}");
        }
    }

    #[test]
    fn grid_is_configurable() {
        let t = Theme {
            space: crate::theme::SpaceScale::grid(10.0),
            ..Theme::default()
        };
        assert!(tokens_css(&t).contains("--space-m: 1.25rem;"));
    }
}
