//! Minimal XML writers for sitemap and RSS. Small enough to not need a crate.

use crate::Lang;
use chrono::{DateTime, FixedOffset};

pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

pub struct SitemapEntry {
    pub loc: String,
    /// All language versions of this page, including itself.
    pub alternates: Vec<(Lang, String)>,
    pub default_alternate: Option<String>,
}

pub fn sitemap(entries: &[SitemapEntry]) -> String {
    let mut s = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\" xmlns:xhtml=\"http://www.w3.org/1999/xhtml\">\n",
    );
    for e in entries {
        s.push_str("  <url>\n    <loc>");
        s.push_str(&escape(&e.loc));
        s.push_str("</loc>\n");
        for (lang, href) in &e.alternates {
            s.push_str(&format!(
                "    <xhtml:link rel=\"alternate\" hreflang=\"{}\" href=\"{}\"/>\n",
                lang.code(),
                escape(href)
            ));
        }
        if let Some(d) = &e.default_alternate {
            s.push_str(&format!(
                "    <xhtml:link rel=\"alternate\" hreflang=\"x-default\" href=\"{}\"/>\n",
                escape(d)
            ));
        }
        s.push_str("  </url>\n");
    }
    s.push_str("</urlset>\n");
    s
}

pub struct RssChannel<'a> {
    pub title: &'a str,
    pub link: &'a str,
    pub description: &'a str,
    pub lang: Lang,
    pub self_url: &'a str,
}

pub struct RssItem<'a> {
    pub title: &'a str,
    pub link: &'a str,
    pub description: Option<&'a str>,
    pub date: DateTime<FixedOffset>,
}

pub fn rss(ch: &RssChannel, items: &[RssItem]) -> String {
    let mut s = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">\n<channel>\n",
    );
    s.push_str(&format!(
        "  <title>{}</title>\n  <link>{}</link>\n  <description>{}</description>\n  <language>{}</language>\n  <atom:link href=\"{}\" rel=\"self\" type=\"application/rss+xml\"/>\n",
        escape(ch.title), escape(ch.link), escape(ch.description), ch.lang.code(), escape(ch.self_url)
    ));
    if let Some(latest) = items.iter().map(|i| i.date).max() {
        s.push_str(&format!(
            "  <lastBuildDate>{}</lastBuildDate>\n",
            latest.to_rfc2822()
        ));
    }
    for i in items {
        s.push_str("  <item>\n");
        s.push_str(&format!("    <title>{}</title>\n", escape(i.title)));
        s.push_str(&format!(
            "    <link>{}</link>\n    <guid>{}</guid>\n",
            escape(i.link),
            escape(i.link)
        ));
        s.push_str(&format!("    <pubDate>{}</pubDate>\n", i.date.to_rfc2822()));
        if let Some(d) = i.description {
            s.push_str(&format!("    <description>{}</description>\n", escape(d)));
        }
        s.push_str("  </item>\n");
    }
    s.push_str("</channel>\n</rss>\n");
    s
}
