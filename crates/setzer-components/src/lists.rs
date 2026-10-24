use setzer_core::{each, html, Doc, Html};
use setzer_design::prelude::*;

pub struct Card<'a> {
    pub kind: &'a str,
    pub title: &'a str,
    pub text: &'a str,
    pub href: &'a str,
}

/// Grid of linked cards: 3 columns, 2 on narrow screens, 1 on phones.
pub fn cards(items: &[Card]) -> Html {
    let card = |c: &Card| {
        html! {
            a href=(c.href) class=(style([flex(), flex_col(), gap(Space::Xs), bg(Color::Surface), border(Stroke::Hairline, Color::Rule), radius(Radius::M), pad(Space::M), fg(Color::Fg), hover(border_color(Color::Accent)), hover(no_underline())])) {
                span class=(style([text(TextRole::Overline), fg(Color::AccentFg)])) { (c.kind) }
                strong class=(style([text(TextRole::H3)])) { (c.title) }
                span class=(style([fg(Color::Muted)])) { (c.text) }
            }
        }
    };
    html! {
        div class=(style([grid(), columns(3), gap(Space::M), margin_bottom(Space::M), at(Breakpoint::Narrow, columns(2)), at(Breakpoint::Phone, columns(1))])) {
            (each(items.iter().map(card)))
        }
    }
}

pub struct Term<'a> {
    pub title: &'a str,
    pub text: &'a str,
}

/// Definition list: bold term, muted description.
pub fn terms(items: &[Term]) -> Html {
    let term = |t: &Term| html! { dt class=(style([weight(600), margin_top(Space::S)])) { (t.title) } dd class=(style([margin_none(), fg(Color::Muted)])) { (t.text) } };
    html! { dl class=(style([margin_none()])) { (each(items.iter().map(term))) } }
}

/// List of title + text pairs.
pub fn titled_list(items: &[Term]) -> Html {
    let item = |t: &Term| html! { li class=(style([margin_bottom(Space::Sm)])) { strong class=(style([block()])) { (t.title) } span class=(style([fg(Color::Muted)])) { (t.text) } } };
    html! { ul class=(style([list_none(), margin_none(), pad_none()])) { (each(items.iter().map(item))) } }
}

pub struct Period<'a> {
    pub dates: &'a str,
    pub role: &'a str,
    pub org: &'a str,
    pub text: &'a str,
}

/// CV-style timeline: dates left, role, organisation and text right.
pub fn timeline(items: &[Period]) -> Html {
    let period = |p: &Period| {
        html! {
            li class=(style([grid(), columns_fixed_first(Size::DateCol), gap(Space::M), pad_y(Space::Sm), border_top(Stroke::Hairline, Color::Rule), first(border_top_none()), at(Breakpoint::Narrow, columns(1)), at(Breakpoint::Narrow, gap(Space::Tight))])) {
                span class=(style([text(TextRole::BodySm), fg(Color::Muted), pad_top(Space::Tight)])) { (p.dates) }
                div {
                    strong class=(style([block()])) { (p.role) }
                    span class=(style([block(), fg(Color::Muted), margin_bottom(Space::Xs)])) { (p.org) }
                    p class=(style([margin_none()])) { (p.text) }
                }
            }
        }
    };
    html! { ol class=(style([list_none(), margin_none(), margin_bottom(Space::Xl), pad_none()])) { (each(items.iter().map(period))) } }
}

/// Compact list of documents (blog posts, releases, …): year and title.
/// `doc_teasers(&post::documents(lang).take(4).collect::<Vec<_>>())`
pub fn doc_teasers(docs: &[&Doc]) -> Html {
    let rows = docs
        .iter()
        .map(|p| teaser_row(&p.date("%Y"), p.title, &p.href()));
    html! { ul class=(style([list_none(), margin_none(), margin_bottom(Space::M), pad_none()])) { (each(rows)) } }
}

fn teaser_row(year: &str, title: &str, href: &str) -> Html {
    html! {
        li class=(style([grid(), columns_fixed_first(Size::YearCol), gap(Space::M), pad_y(Space::Xs)])) {
            time class=(style([fg(Color::Muted), tabular_nums()])) { (year) }
            a href=(href) { (title) }
        }
    }
}

/// Full list of documents (blog posts, releases, …): date, title, description.
/// `doc_list(&post::documents(lang).collect::<Vec<_>>())`
pub fn doc_list(docs: &[&Doc]) -> Html {
    let rows = docs
        .iter()
        .map(|p| list_row(&p.date("%Y-%m-%d"), p.title, &p.href(), p.description));
    html! { ul class=(style([list_none(), margin_none(), pad_none()])) { (each(rows)) } }
}

fn list_row(date: &str, title: &str, href: &str, description: Option<&str>) -> Html {
    html! {
        li class=(style([grid(), columns_fixed_first(Size::YearCol), gap(Space::M), pad_y(Space::Sm), border_top(Stroke::Hairline, Color::Rule), at(Breakpoint::Narrow, columns(1)), at(Breakpoint::Narrow, gap(Space::Tight))])) {
            time class=(style([fg(Color::Muted), tabular_nums(), at(Breakpoint::Narrow, text(TextRole::Small))])) { (date) }
            div {
                a href=(href) class=(style([weight(600)])) { (title) }
                @if let Some(d) = description { p class=(style([text(TextRole::BodySm), margin_top(Space::Tight), margin_bottom_none(), fg(Color::Muted)])) { (truncate(d, 160)) } }
            }
        }
    }
}

/// " · Also available in Deutsch": links to the post's other languages,
/// or nothing if it has none. `translations(post, c.also_in)`
pub fn translations(post: &Doc, label: &str) -> Html {
    let others = post.translations();
    let link = |(i, other): (usize, &&Doc)| {
        let code = other.lang().code();
        html! { @if i > 0 { ", " } a href=(other.href()) hreflang=(code) lang=(code) { (other.lang().native_name()) } }
    };
    if others.is_empty() {
        return html! {};
    }
    html! { " · " (label) " " (each(others.iter().enumerate().map(link))) }
}

#[cfg(test)]
pub(crate) fn rows_for_lint() -> Html {
    html! { (teaser_row("d", "t", "/")) (list_row("d", "t", "/", Some("x"))) }
}

pub struct ContactRow<'a> {
    pub label: &'a str,
    pub value: Html,
}

/// Label/value rows (email, profiles, location).
pub fn contact(rows: &[ContactRow]) -> Html {
    let row = |r: &ContactRow| {
        html! {
            li class=(style([grid(), columns_fixed_first(Size::LabelCol), gap(Space::M), pad_y(Space::Xs), at(Breakpoint::Narrow, columns(1)), at(Breakpoint::Narrow, gap_none())])) {
                span class=(style([fg(Color::Muted)])) { (r.label) }
                (r.value)
            }
        }
    };
    html! { ul class=(style([list_none(), margin_none(), pad_none()])) { (each(rows.iter().map(row))) } }
}

/// Muted line of metadata under a title.
pub fn meta_line(content: Html) -> Html {
    html! { p class=(style([text(TextRole::BodySm), fg(Color::Muted)])) { (content) } }
}

/// Highlighted note.
pub fn notice(message: &str) -> Html {
    html! { p class=(style([text(TextRole::BodySm), bg(Color::AccentSoft), radius(Radius::S), pad_xy(Space::S, Space::Sm)])) { (message) } }
}

/// Native accordion, no JavaScript.
pub fn accordion(items: &[(Html, Html)]) -> Html {
    let item = |(summary, body): &(Html, Html)| {
        html! {
            details class=(style([border_top(Stroke::Hairline, Color::Rule), pad_y(Space::Sm)])) {
                summary class=(style([pointer(), weight(600)])) { (summary) }
                div class=(style([pad_top(Space::S), fg(Color::Muted)])) { (body) }
            }
        }
    };
    html! { div { (each(items.iter().map(item))) } }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let cut: String = s.chars().take(max).collect();
    let cut = cut.rsplit_once(' ').map(|(a, _)| a).unwrap_or(&cut);
    format!("{cut}…")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_cuts_at_word() {
        assert_eq!(truncate("one two three", 8), "one two…");
        assert_eq!(truncate("short", 8), "short");
    }

    #[test]
    fn text_is_escaped() {
        let m = terms(&[Term {
            title: "<b>",
            text: "t",
        }]);
        assert!(m.into_string().contains("&lt;b&gt;"));
    }
}
