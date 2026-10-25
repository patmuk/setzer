//! The site builder: settings, pages, then `write()`.

use crate::content::{Document, StaticFile};
use crate::frame::Frame;
use crate::lang::Lang;
use crate::page::{PageDef, RenderFn};
use crate::render::{self, STYLESHEET_PLACEHOLDER};
use crate::{cli, config, xml};
use anyhow::{bail, Context as _, Result};
use base64::Engine;
use maud::Markup;
use setzer_design::style::record;
use setzer_design::{base, emit, lint, Css, Theme};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::Path;

/// Wraps a page's body into the site's header, footer and `<head>`.
pub type FrameFn = fn(&Frame, Markup) -> Markup;

pub struct Site {
    langs: Vec<Lang>,
    theme: Theme,
    frame: Option<FrameFn>,
    css: Vec<Css>,
    statics: Vec<&'static [StaticFile]>,
    files: Vec<(String, String)>,
    headers: bool,
    header_rules: Vec<(String, String, String)>,
    orphans_fail: bool,
}

struct Entry {
    def: &'static PageDef,
    doc: Option<&'static Document>,
    /// Language independent.
    path: String,
    langs: Vec<Lang>,
}

impl Site {
    /// Starts a site. `langs[0]` is the default language and lives at `/`;
    /// the others under `/<code>/`.
    pub fn new(base_url: &str, name: &str, langs: &[Lang]) -> Site {
        assert!(!langs.is_empty(), "a site needs at least one language");
        config::set(config::Config {
            base_url: base_url.trim_end_matches('/').to_string(),
            name: name.to_string(),
            langs: langs.to_vec(),
        });
        Site {
            langs: langs.to_vec(),
            theme: Theme::default(),
            frame: None,
            css: Vec::new(),
            statics: Vec::new(),
            files: Vec::new(),
            headers: true,
            header_rules: Vec::new(),
            orphans_fail: false,
        }
    }

    /// The look. Default: `Theme::default()`.
    pub fn theme(mut self, theme: Theme) -> Site {
        self.theme = theme;
        self
    }

    /// Header, footer and `<head>` around every page. Without a frame, a
    /// page renders its whole document itself.
    pub fn frame(mut self, frame: FrameFn) -> Site {
        self.frame = Some(frame);
        self
    }

    /// Site-wide rules, e.g. `rule("button", [bg(Color::Accent)])`.
    pub fn css(mut self, css: Css) -> Site {
        self.css.push(css);
        self
    }

    /// Files copied as is to the site root: `static_files(setzer::static_dir!("static"))`.
    pub fn static_files(mut self, files: &'static [StaticFile]) -> Site {
        self.statics.push(files);
        self
    }

    /// A generated file at the site root.
    pub fn file(mut self, name: &str, content: impl Into<String>) -> Site {
        self.files.push((name.to_string(), content.into()));
        self
    }

    /// Treat a page that no other page links to as an error instead of a
    /// warning.
    pub fn orphans_fail(mut self) -> Site {
        self.orphans_fail = true;
        self
    }

    /// An extra rule for Cloudflare's `_headers` file, e.g.
    /// `.header("/.well-known/apple-app-site-association", "Content-Type", "application/json")`.
    pub fn header(mut self, path: &str, name: &str, value: &str) -> Site {
        self.header_rules
            .push((path.to_string(), name.to_string(), value.to_string()));
        self
    }

    /// Do not write Cloudflare's `_headers` file.
    pub fn without_headers(mut self) -> Site {
        self.headers = false;
        self
    }

    fn entries(&self) -> Result<Vec<Entry>> {
        let mut defs: Vec<&'static PageDef> = crate::PAGES.iter().copied().collect();
        if defs.is_empty() {
            bail!("no pages: call `setzer::site!()` in main.rs and `setzer::page!()` in src/www/mod.rs");
        }
        defs.sort_by_key(|d| d.module);
        let mut entries = Vec::new();
        for def in defs {
            match def.render {
                RenderFn::Single(_) => {
                    entries.push(Entry {
                        def,
                        doc: None,
                        path: def.route(),
                        langs: self.langs.clone(),
                    });
                }
                RenderFn::Each(docs, _) => {
                    for doc in docs {
                        let langs: Vec<Lang> = doc
                            .langs()
                            .into_iter()
                            .filter(|l| self.langs.contains(l))
                            .collect();
                        entries.push(Entry {
                            def,
                            doc: Some(doc),
                            path: format!("{}{}/", def.route(), doc.slug),
                            langs,
                        });
                    }
                }
            }
        }
        let mut seen = BTreeSet::new();
        for e in &entries {
            if !seen.insert(e.path.clone()) {
                bail!(
                    "two pages want the path {} (one is {})",
                    e.path,
                    e.def.module
                );
            }
        }
        Ok(entries)
    }

    /// Renders every page in every language and writes the site to
    /// `--out <dir>` (default `target/site`): HTML, the stylesheet of what
    /// was used, images, static files, feeds, sitemap, `_headers`.
    /// Fails, writing nothing, if a link points to a page or section that
    /// does not exist.
    pub fn write(self) -> Result<()> {
        let options = cli::options()?;
        let out = options.out.clone();
        let entries = self.entries()?;
        record::start();
        render::STATE.with(|s| *s.borrow_mut() = Default::default());

        // Feeds first, so every page can link to its language's feed.
        let mut feeds: Vec<(Lang, String, String)> = Vec::new();
        for e in &entries {
            if let (Some(feed), None) = (e.def.feed, e.doc) {
                for &lang in &e.langs {
                    let page_path = config::in_lang(&e.path, lang);
                    render::begin_page(lang, &page_path);
                    let path = format!("{page_path}rss.xml");
                    feeds.push((lang, path.clone(), rss(&page_path, &path, lang, feed(lang))));
                }
            }
        }

        let mut pages: Vec<(String, String)> = Vec::new();
        for e in &entries {
            for &lang in &e.langs {
                let path = config::in_lang(&e.path, lang);
                render::begin_page(lang, &path);
                let body = match (&e.def.render, e.doc) {
                    (RenderFn::Single(f), _) => f(lang),
                    (RenderFn::Each(_, f), Some(doc)) => f(doc, lang),
                    (RenderFn::Each(..), None) => unreachable!(),
                };
                let html = match self.frame {
                    Some(frame) => {
                        let (title, description) = match e.doc {
                            Some(doc) => doc
                                .get(lang)
                                .map(|d| (Some(d.title), d.description))
                                .unwrap_or_default(),
                            None => {
                                let m = (e.def.meta)(lang);
                                (m.title, m.description)
                            }
                        };
                        let f = Frame {
                            lang,
                            path: path.clone(),
                            title,
                            description,
                            canonical: config::url(&path),
                            alternates: e
                                .langs
                                .iter()
                                .map(|&l| (l, config::url(&config::in_lang(&e.path, l))))
                                .collect(),
                            switches: self
                                .langs
                                .iter()
                                .filter(|&&l| l != lang)
                                .map(|&l| {
                                    (
                                        l,
                                        config::in_lang(
                                            if e.langs.contains(&l) { &e.path } else { "/" },
                                            l,
                                        ),
                                    )
                                })
                                .collect(),
                            feed: feeds
                                .iter()
                                .find(|(l, _, _)| *l == lang)
                                .map(|(_, p, _)| p.clone()),
                            document: e.doc.is_some(),
                        };
                        frame(&f, body)
                    }
                    None => body,
                };
                pages.push((path, html.into_string()));
            }
        }
        let state = render::STATE.with(|s| std::mem::take(&mut *s.borrow_mut()));
        let recorded = record::finish().css();

        // Checks: CSS without literals, links to existing pages and sections.
        let mut problems = Vec::new();
        let mut component_css: String = self.css.iter().map(|c| c.to_string()).collect();
        component_css.push_str(&recorded);
        for literal in lint::literals(&component_css) {
            problems.push(format!(
                "a style contains the literal `{literal}`; use a token"
            ));
        }
        let statics: BTreeSet<String> = self
            .statics
            .iter()
            .flat_map(|s| s.iter())
            .map(|f| format!("/{}", f.path))
            .collect();
        for (target, anchor, from) in &state.links {
            match pages.iter().find(|(p, _)| p == target) {
                None if statics.contains(target) => {}
                None => problems.push(format!(
                    "{from} links to {target}, which is not a page of this site"
                )),
                Some((_, html)) => {
                    if let Some(id) = anchor {
                        if !html.contains(&format!("id=\"{id}\"")) {
                            problems.push(format!("{from} links to {target}#{id}, but that page has no element with id=\"{id}\""));
                        }
                    }
                }
            }
        }
        // Reachability: from each language's home and the fixed-path pages
        // (404), following every recorded link (markdown, components, frame).
        let mut reached: BTreeSet<String> = BTreeSet::new();
        let mut queue: Vec<String> = self
            .langs
            .iter()
            .map(|&l| config::in_lang("/", l))
            .collect();
        queue.extend(
            entries
                .iter()
                .filter(|e| e.def.path.is_some())
                .flat_map(|e| e.langs.iter().map(|&l| config::in_lang(&e.path, l))),
        );
        while let Some(path) = queue.pop() {
            if reached.insert(path.clone()) {
                queue.extend(
                    state
                        .links
                        .iter()
                        .filter(|(_, _, from)| *from == path)
                        .map(|(target, _, _)| target.clone()),
                );
            }
        }
        let orphans: Vec<&String> = pages
            .iter()
            .map(|(p, _)| p)
            .filter(|p| !reached.contains(*p))
            .collect();
        for orphan in &orphans {
            let message = format!(
                "{orphan} is not linked from any page that is reachable from the home page"
            );
            if self.orphans_fail {
                problems.push(message);
            } else {
                eprintln!("warning: {message}");
            }
        }

        if !problems.is_empty() {
            problems.sort();
            problems.dedup();
            bail!(
                "the site has {} problem(s):\n  {}",
                problems.len(),
                problems.join("\n  ")
            );
        }

        // Write.
        clear_dir(&out)?;
        let mut css = emit::tokens_css(&self.theme);
        css.push_str(&base::base_css().to_string());
        css.push_str(&component_css);
        let digest = Sha256::digest(css.as_bytes());
        let href = format!(
            "/css/site-{}.css",
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&digest[..9])
        );
        write_file(&out, &href, css.as_bytes())?;
        for (path, html) in &pages {
            write_file(
                &out,
                path,
                html.replace(STYLESHEET_PLACEHOLDER, &href).as_bytes(),
            )?;
        }
        for (_, path, xml) in &feeds {
            write_file(&out, path, xml.as_bytes())?;
        }
        let cache = cli::target_dir().map(|t| t.join("setzer-cache").join("img"));
        for img in &state.images {
            img.write(&out.join("img"), cache.as_deref())?;
        }
        for file in self.statics.iter().flat_map(|s| s.iter()) {
            write_file(&out, &format!("/{}", file.path), file.bytes)?;
        }
        let default_lang = self.langs[0];
        let sitemap: Vec<xml::SitemapEntry> = entries
            .iter()
            .filter(|e| e.path.ends_with('/'))
            .flat_map(|e| {
                e.langs.iter().map(move |&l| xml::SitemapEntry {
                    loc: config::url(&config::in_lang(&e.path, l)),
                    alternates: e
                        .langs
                        .iter()
                        .map(|&a| (a, config::url(&config::in_lang(&e.path, a))))
                        .collect(),
                    default_alternate: e
                        .langs
                        .contains(&default_lang)
                        .then(|| config::url(&e.path)),
                })
            })
            .collect();
        write_file(&out, "/sitemap.xml", xml::sitemap(&sitemap).as_bytes())?;
        for (name, content) in &self.files {
            write_file(&out, &format!("/{name}"), content.as_bytes())?;
        }
        if self.headers && !self.files.iter().any(|(n, _)| n == "_headers") {
            write_file(
                &out,
                "/_headers",
                crate::script::cloudflare_headers(
                    state.scripts.iter().copied(),
                    &self.header_rules,
                )
                .as_bytes(),
            )?;
        }

        let shown = std::env::current_dir()
            .ok()
            .and_then(|cwd| out.strip_prefix(cwd).ok().map(Path::to_path_buf))
            .unwrap_or_else(|| out.clone());
        let count =
            |n: usize, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
        println!(
            "setzer: wrote {} in {} to {}",
            count(pages.len(), "page", "pages"),
            count(self.langs.len(), "language", "languages"),
            shown.display()
        );
        match options.serve {
            Some(port) => crate::serve::run(&out, port),
            None => {
                println!("        preview: cargo run -- --serve");
                Ok(())
            }
        }
    }
}

fn rss(page_path: &str, self_path: &str, lang: Lang, feed: crate::page::Feed) -> String {
    let links: Vec<String> = feed.items.iter().map(|i| config::url(&i.href)).collect();
    let items: Vec<xml::RssItem> = feed
        .items
        .iter()
        .zip(&links)
        .filter_map(|(i, link)| {
            Some(xml::RssItem {
                title: &i.title,
                link,
                description: i.description.as_deref(),
                date: i.date?,
            })
        })
        .collect();
    let channel = xml::RssChannel {
        title: &feed.title,
        link: &config::url(page_path),
        description: &feed.description,
        lang,
        self_url: &config::url(self_path),
    };
    xml::rss(&channel, &items)
}

fn write_file(out: &Path, path: &str, bytes: &[u8]) -> Result<()> {
    let rel = path.trim_start_matches('/');
    let target = if path.ends_with('/') {
        out.join(rel).join("index.html")
    } else {
        out.join(rel)
    };
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&target, bytes).with_context(|| format!("writing {}", target.display()))
}

/// Empties `dir`, keeping the directory itself so a server watching it
/// (`just dev`) keeps serving. A link there (from `nix build --out-link`)
/// is replaced by a real directory.
fn clear_dir(dir: &Path) -> Result<()> {
    if dir.is_symlink() {
        std::fs::remove_file(dir).with_context(|| format!("removing link {}", dir.display()))?;
    }
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() && !path.is_symlink() {
                std::fs::remove_dir_all(&path)
            } else {
                std::fs::remove_file(&path)
            }
            .with_context(|| format!("clearing {}", path.display()))?;
        }
    }
    std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))
}
