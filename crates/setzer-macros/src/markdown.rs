//! Markdown → HTML at compile time, with internal links kept as typed parts.
//!
//! A link without scheme is internal and checked:
//! * `[[apps/ringio]]`, `[text](apps/ringio)`, `[text](/apps/ringio#faq)` →
//!   the page module `crate::www::apps::ringio` (compile error if missing);
//!   `[text](/)` or `[text](/#contact)` → the home page, `crate::www`;
//! * `[text](#faq)` → a section of the page being rendered;
//! * a path that is a file in the crate's `static/` folder → that file.
//!
//! Links with a scheme (`https:`, `mailto:`) stay as they are.

use proc_macro2::{Span, TokenStream as Ts};
use pulldown_cmark::{html, Event, LinkType, Options, Parser, Tag, TagEnd};
use quote::quote;
use std::path::Path;
use syn::Ident;

pub fn split_front_matter(raw: &str) -> Result<(&str, &str), String> {
    let Some(rest) = raw.strip_prefix("+++") else {
        return Ok(("", raw));
    };
    let end = rest.find("\n+++").ok_or("unterminated +++ front matter")?;
    Ok((&rest[..end], rest[end + 4..].trim_start_matches('\n')))
}

#[derive(Clone, Debug)]
pub enum PartSpec {
    Html(String),
    Page {
        segments: Vec<String>,
        anchor: Option<String>,
        text: Option<String>,
    },
    Anchor {
        anchor: String,
        text: String,
    },
    Static {
        path: String,
        text: String,
    },
}

pub fn body_parts(md: &str, manifest: &Path) -> Result<Vec<PartSpec>, String> {
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_WIKILINKS;
    let events: Vec<Event> = Parser::new_ext(md, opts).collect();
    let mut parts: Vec<PartSpec> = Vec::new();
    let mut pending: Vec<Event> = Vec::new();
    let mut i = 0;
    while i < events.len() {
        if let Event::Start(Tag::Link {
            link_type,
            dest_url,
            ..
        }) = &events[i]
        {
            if is_internal(dest_url) {
                let end = (i + 1..events.len())
                    .find(|&j| matches!(events[j], Event::End(TagEnd::Link)))
                    .ok_or("unterminated link")?;
                flush(&mut pending, &mut parts);
                let mut text = String::new();
                html::push_html(&mut text, events[i + 1..end].iter().cloned());
                let plain_wiki = matches!(link_type, LinkType::WikiLink { has_pothole: false });
                parts.push(classify(dest_url, text, plain_wiki, manifest)?);
                i = end + 1;
                continue;
            }
        }
        pending.push(events[i].clone());
        i += 1;
    }
    flush(&mut pending, &mut parts);
    Ok(parts)
}

fn flush(pending: &mut Vec<Event>, parts: &mut Vec<PartSpec>) {
    if pending.is_empty() {
        return;
    }
    let mut out = String::new();
    html::push_html(&mut out, pending.drain(..));
    match parts.last_mut() {
        Some(PartSpec::Html(prev)) => prev.push_str(&out),
        _ => parts.push(PartSpec::Html(out)),
    }
}

fn is_internal(dest: &str) -> bool {
    let head = dest.split(['/', '#', '?']).next().unwrap_or("");
    !dest.is_empty() && !head.contains(':')
}

fn classify(
    dest: &str,
    text: String,
    plain_wiki: bool,
    manifest: &Path,
) -> Result<PartSpec, String> {
    if let Some(anchor) = dest.strip_prefix('#') {
        let text = if plain_wiki { anchor.to_string() } else { text };
        return Ok(PartSpec::Anchor {
            anchor: anchor.to_string(),
            text,
        });
    }
    let (path, anchor) = match dest.split_once('#') {
        Some((p, a)) => (p, Some(a.to_string())),
        None => (dest, None),
    };
    let path = path.trim_matches('/');
    if !path.is_empty() && manifest.join("static").join(path).is_file() {
        return Ok(PartSpec::Static {
            path: format!("/{path}"),
            text,
        });
    }
    let segments: Vec<String> = path
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.replace('-', "_"))
        .collect();
    for s in &segments {
        if syn::parse_str::<Ident>(s).is_err() {
            return Err(format!(
                "link `{dest}`: `{s}` is not a page module name (and no static/{path} exists)"
            ));
        }
    }
    Ok(PartSpec::Page {
        segments,
        anchor,
        text: (!plain_wiki).then_some(text),
    })
}

fn opt(s: &Option<String>) -> Ts {
    match s {
        Some(s) => quote!(Some(#s)),
        None => quote!(None),
    }
}

/// `::setzer::Body(&[...])`. Page links become references to the target's
/// `PAGE` static below `crate::www`, so a missing page is a compile error.
pub fn emit_parts(parts: &[PartSpec]) -> Ts {
    let items = parts.iter().map(|p| match p {
        PartSpec::Html(h) => quote!(::setzer::Part::Html(#h)),
        PartSpec::Page {
            segments,
            anchor,
            text,
        } => {
            let segs = segments.iter().map(|s| Ident::new(s, Span::call_site()));
            let anchor = opt(anchor);
            let text = opt(text);
            {
                // the home page is `crate::www` itself
                let page = if segments.is_empty() {
                    quote!(crate::www::PAGE)
                } else {
                    quote!(crate::www::#(#segs)::*::PAGE)
                };
                quote!(::setzer::Part::Page { page: &#page, anchor: #anchor, text: #text })
            }
        }
        PartSpec::Anchor { anchor, text } => {
            quote!(::setzer::Part::Anchor { anchor: #anchor, text: #text })
        }
        PartSpec::Static { path, text } => {
            quote!(::setzer::Part::Static { path: #path, text: #text })
        }
    });
    quote!(::setzer::Body(&[#(#items),*]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts(md: &str) -> Vec<PartSpec> {
        body_parts(md, Path::new("/nonexistent")).unwrap()
    }

    #[test]
    fn links_are_classified() {
        let p = parts("See [[apps/ringio]], [the app](apps/ringio#faq), [below](#faq) and [web](https://example.com).");
        assert!(
            matches!(&p[1], PartSpec::Page { segments, anchor: None, text: None } if segments == &["apps", "ringio"])
        );
        assert!(
            matches!(&p[3], PartSpec::Page { anchor: Some(a), text: Some(t), .. } if a == "faq" && t == "the app")
        );
        assert!(matches!(&p[5], PartSpec::Anchor { anchor, .. } if anchor == "faq"));
        assert!(matches!(&p[6], PartSpec::Html(h) if h.contains("href=\"https://example.com\"")));
    }

    #[test]
    fn root_and_dashes() {
        let p = parts("[home](/) [x](not-found)");
        assert!(matches!(&p[1], PartSpec::Page { segments, .. } if segments.is_empty()));
        assert!(matches!(&p[3], PartSpec::Page { segments, .. } if segments == &["not_found"]));
    }
}
