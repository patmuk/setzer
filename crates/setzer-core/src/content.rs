//! Content compiled from markdown by `page!`, `content!` and `documents`.

use crate::lang::Lang;
use crate::page::PageDef;
use crate::render;
use maud::Render;

/// Markdown rendered at compile time. Internal links stay typed until the
/// page is rendered, so their language prefix and target can be resolved
/// and checked.
#[derive(Clone, Copy)]
pub struct Body(pub &'static [Part]);

#[doc(hidden)]
#[derive(Clone, Copy)]
pub enum Part {
    Html(&'static str),
    /// `[[apps/ringio]]`, `[text](apps/ringio#faq)`. `text` is the link's
    /// inner HTML; `None` means the target page's title.
    Page {
        page: &'static PageDef,
        anchor: Option<&'static str>,
        text: Option<&'static str>,
    },
    /// `[text](#faq)`: a section of the page being rendered.
    Anchor {
        anchor: &'static str,
        text: &'static str,
    },
    /// A file from the site's `static/` folder.
    Static {
        path: &'static str,
        text: &'static str,
    },
}

impl Body {
    pub const EMPTY: Body = Body(&[]);
}

impl Render for Body {
    fn render_to(&self, out: &mut String) {
        let lang = render::lang();
        for part in self.0 {
            match *part {
                Part::Html(html) => out.push_str(html),
                Part::Page { page, anchor, text } => {
                    let href = match anchor {
                        Some(a) => render::anchor(page, a, lang),
                        None => render::href(page, lang),
                    };
                    let text = text
                        .map(str::to_string)
                        .unwrap_or_else(|| escape(page.title(lang)));
                    out.push_str(&format!("<a href=\"{href}\">{text}</a>"));
                }
                Part::Anchor { anchor, text } => {
                    render::record_anchor(None, anchor);
                    out.push_str(&format!("<a href=\"#{anchor}\">{text}</a>"));
                }
                Part::Static { path, text } => {
                    out.push_str(&format!("<a href=\"{path}\">{text}</a>"))
                }
            }
        }
    }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// One markdown file of a collection (a blog post) in one language. It
/// knows its page and language, so `doc.href()` needs no arguments.
pub struct Doc {
    pub title: &'static str,
    /// RFC 3339 or `YYYY-MM-DD`; from the front matter or the file name.
    pub date: Option<&'static str>,
    pub description: Option<&'static str>,
    pub body: Body,
    /// The file name without date and language: `2014-03-09-hello.de.md` → `hello`.
    pub slug: &'static str,
    lang: Option<Lang>,
    page: &'static PageDef,
}

impl Doc {
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub const fn __new(
        title: &'static str,
        date: Option<&'static str>,
        description: Option<&'static str>,
        body: Body,
        slug: &'static str,
        lang: Option<Lang>,
        page: &'static PageDef,
    ) -> Doc {
        Doc {
            title,
            date,
            description,
            body,
            slug,
            lang,
            page,
        }
    }

    /// The language of this version.
    pub fn lang(&self) -> Lang {
        self.lang.unwrap_or_else(crate::config::default_lang)
    }

    /// Where this version is published: `/blog/hello/`, `/de/blog/hello/`.
    pub fn href(&self) -> String {
        render::doc_path(self.page, self.slug, self.lang())
    }

    /// The same document in the site's other languages, if translated.
    pub fn translations(&self) -> Vec<&'static Doc> {
        let crate::page::RenderFn::Each(documents, _) = self.page.render else {
            return Vec::new();
        };
        let lang = self.lang();
        documents
            .iter()
            .filter(|d| d.slug == self.slug)
            .flat_map(|d| d.versions.iter().map(|(_, doc)| doc))
            .filter(|doc| doc.lang() != lang && crate::config::get().langs.contains(&doc.lang()))
            .collect()
    }
    /// The date formatted with chrono's `fmt` (`"%Y-%m-%d"`), or "".
    pub fn date(&self, fmt: &str) -> String {
        let Some(d) = self.date else {
            return String::new();
        };
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(d) {
            return dt.format(fmt).to_string();
        }
        chrono::NaiveDate::parse_from_str(&d[..d.len().min(10)], "%Y-%m-%d")
            .map(|n| n.format(fmt).to_string())
            .unwrap_or_default()
    }

    /// The date as a timestamp, for feeds.
    pub fn datetime(&self) -> Option<chrono::DateTime<chrono::FixedOffset>> {
        let d = self.date?;
        chrono::DateTime::parse_from_rfc3339(d).ok().or_else(|| {
            chrono::NaiveDate::parse_from_str(&d[..d.len().min(10)], "%Y-%m-%d")
                .ok()
                .map(|n| n.and_hms_opt(0, 0, 0).unwrap().and_utc().fixed_offset())
        })
    }
}

/// A collection entry: one slug, its versions per language. `None` is the
/// file without language suffix, which belongs to the site's default language.
pub struct Document {
    pub slug: &'static str,
    pub versions: &'static [(Option<Lang>, Doc)],
}

/// `post::documents(lang)`: the documents that exist in `lang`, newest first.
#[doc(hidden)]
pub fn documents_in(all: &'static [Document], lang: Lang) -> impl Iterator<Item = &'static Doc> {
    all.iter().filter_map(move |d| d.get(lang))
}

impl Document {
    /// The version in `lang`, if this document exists in it.
    pub fn get(&self, lang: Lang) -> Option<&Doc> {
        let default = crate::config::default_lang();
        self.versions
            .iter()
            .find(|(l, _)| *l == Some(lang) || (l.is_none() && lang == default))
            .map(|(_, d)| d)
    }

    /// The version in `lang`; panics if there is none (use inside its own page).
    pub fn doc(&self, lang: Lang) -> &Doc {
        self.get(lang)
            .unwrap_or_else(|| panic!("document {} has no {lang:?} version", self.slug))
    }

    /// Languages this document exists in, default language first.
    pub fn langs(&self) -> Vec<Lang> {
        let default = crate::config::default_lang();
        let mut langs: Vec<Lang> = self
            .versions
            .iter()
            .map(|(l, _)| l.unwrap_or(default))
            .collect();
        langs.sort_by_key(|l| *l != default);
        langs.dedup();
        langs
    }
}

/// A file copied verbatim to the site root, embedded by `static_dir!`.
pub struct StaticFile {
    pub path: &'static str,
    pub bytes: &'static [u8],
}
