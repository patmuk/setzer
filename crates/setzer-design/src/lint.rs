//! Guards the "no magic numbers in components" rule for rendered CSS.

/// Returns every offending fragment: a length with a unit, a hex colour,
/// or an rgb/hsl call, outside `@media (...)` preludes.
pub fn literals(css: &str) -> Vec<String> {
    let unit =
        regex::Regex::new(r"(?i)(?:^|[^\w.-])-?\d*\.?\d+(?:px|rem|em|vw|vh|dvh|pt|ch)\b").unwrap();
    let color = regex::Regex::new(r"(?i)#[0-9a-f]{3,8}\b|\b(?:rgba?|hsla?)\(").unwrap();
    let media = regex::Regex::new(r"@media[^{]*").unwrap();
    let cleaned = media.replace_all(css, "@media");
    let mut out: Vec<String> = unit
        .find_iter(&cleaned)
        .map(|m| m.as_str().trim().to_string())
        .collect();
    out.extend(color.find_iter(&cleaned).map(|m| m.as_str().to_string()));
    out
}

/// Panics with every offender; for component crates' tests.
pub fn assert_clean(name: &str, css: &str) {
    let found = literals(css);
    assert!(found.is_empty(), "{name}: literals in CSS: {found:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_literals() {
        assert_eq!(
            literals("a { padding: 4px; color: #fff }"),
            vec!["4px", "#fff"]
        );
        assert!(
            literals("a { padding: var(--space-m); width: 100%; z-index: 5; flex: 1 1 0; }")
                .is_empty()
        );
        assert!(literals("@media (max-width: 48rem) { a { gap: 0 } }").is_empty());
    }

    #[test]
    fn base_is_clean() {
        assert_clean("base", &crate::base::base_css().to_string());
    }

    #[test]
    fn every_utility_class_is_clean() {
        use crate::tokens::*;
        use crate::utility::*;
        let samples = [
            pad(Space::M),
            column(Space::M, Width::Wrap),
            border(Stroke::Hairline, Color::Rule),
            text(TextRole::H1),
            gradient(90, &[(Color::ScrimHeavy, 0)]),
        ];
        for u in samples {
            assert_clean(&u.class(), u.decls());
        }
    }
}
