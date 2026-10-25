# Setzer - SSC (Static Site Compiler)

**Static sites that the Rust compiler checks before your visitors do.**

*Setzer* (German for typesetter) builds a static website from an ordinary Rust program. Your
pages are Rust modules. Your text is Markdown that turns into typed structs at compile time.
Links between pages are checked, translations are checked, and styling is a typed utility system
in the spirit of Tailwind. There is no template language to learn and no JavaScript framework to
ship.

The result is a folder of plain HTML with one hashed stylesheet that holds only the rules your
pages use. Images are resized to the widths you ask for, and Setzer also writes a sitemap, RSS
feeds and security headers. You can host that folder anywhere.

```rust
setzer::page!();

pub fn render(c: &Content, lang: Lang) -> Html {
    html! {
        h1 class=(style([text(TextRole::H1), fg(Color::Accent)])) { (c.title) }
        (c.body)
        a href=(anchor(&crate::www::PAGE, "contact", lang)) { (c.cta) }
    }
}
```

Every name in that snippet is checked. If `content.md` has no `cta` key, if `content.de.md`
forgot to translate it, if `Color::Accent` were spelled wrong, or if the home page were deleted,
`cargo build` fails and your editor underlines the line. If the home page loses its `#contact`
section, the build stops with an error before anything is written.

## Contents

- [Why Setzer](#why-setzer)
- [Drawbacks, honestly](#drawbacks-honestly)
- [What you get](#what-you-get)
- [Crates](#crates)
- [Hello world](#hello-world)
- [How it works: compile, then run](#how-it-works-compile-then-run)
- [Pages](#pages)
  - [The module tree is the site](#the-module-tree-is-the-site)
  - [`page!` options](#page-options)
  - [Text and translations](#text-and-translations)
  - [Sections: a page built from parts](#sections-a-page-built-from-parts)
  - [Fixed paths: the 404 page](#fixed-paths-the-404-page)
  - [Blogs and other collections](#blogs-and-other-collections)
  - [Links](#links)
  - [Images and static files](#images-and-static-files)
  - [The frame](#the-frame)
- [Styling](#styling)
- [Components](#components)
  - [Built-in components](#built-in-components)
  - [Your own components](#your-own-components)
- [Embedding app code (WASM)](#embedding-app-code-wasm)
- [Deploy](#deploy)
- [Troubleshooting](#troubleshooting)
- [Status](#status)
- [License](#license)

## Why Setzer

**You already know the stack.** No Tera, Liquid, MDX or Astro components. Rust, HTML, CSS and
Markdown are all you need, and the best documentation for each of them is already on the web.

**A full language, on purpose.** Template languages keep growing ad-hoc features until they turn
into a bad programming language. Setzer starts with a good one. Gradle made the same choice when
it moved from a Groovy DSL to Kotlin.

**The compiler reviews your site.** Your text, links and styles become Rust types, so mistakes
show up in the editor while you type, not in production:

| Mistake | Caught |
|---|---|
| typo in a text field (`c.titel`) | compile time, in the editor |
| translation missing a key | compile time, naming both files |
| link to a removed, renamed or `_`-disabled page | compile time |
| wrong token kind (`pad(Color::Fg)`), or a raw value like `4px` | compile time: it can't be written |
| link to a section `id` that does not exist | before anything is written |
| a length or colour in a `raw(…)` style | before anything is written |
| page that no link reaches | warning, or an error with `.orphans_fail()` |

**Small, safe output.** Plain HTML, one CSS file with only the rules your pages render, images in
the sizes you use, and no JavaScript unless you ask for it: the theme toggle, or a page script
via `setzer::script(…)`. Setzer writes a Content-Security-Policy to match that lets exactly those
scripts run, so a site without any allows no script at all.

**Your app's Rust can run on your product page.** If your product has a Rust core, you can compile
it to WebAssembly and mount it in a component, so the live demo runs the same code as the app.

**Customisable, with nothing hidden.** Components are functions you own, themes are structs with
defaults, and the pipeline is a few hundred lines. Two things happen quietly: pages register
themselves with the site, and rendering records the CSS, images and links it needs. That is the
whole list.

## Drawbacks, honestly

Compared with Astro, Zola, Hugo or Eleventy:

- **It is small.** Most widgets a website might want don't exist yet, so you write them. A blog
  works, with collections, a list page, an RSS feed per language and translations per post.
  Pagination, tags, search and comments are not built in. You can write pagination and tags as
  more dynamic pages; search and comments need an external service.
- **You need to know HTML and CSS.** The utilities are typed CSS, not an abstraction that hides
  it.
- **There are no designs to grab.** No theme gallery and no starter templates yet.
- **There is no dedicated community.** No forum, no Discord. But every problem you will hit is an
  HTML, CSS or Rust problem, and those have the deepest knowledge base on the internet.
- **There is a compile step.** `cargo watch` rebuilds in a second or two, which is close to
  `zola serve` but not quite as fast.
- **You build where Rust is installed.** Build locally or in CI, then upload the folder.

## What you get

- **Pages are folders.** A folder with a `mod.rs` is a page, and the folder tree is the URL tree.
  There is no routes table, page list or config file to keep in sync.
- **Text as structs.** Each Markdown file becomes a struct: front-matter keys become fields, and
  the Markdown below becomes `body`. Long pages split into [sections](#sections-a-page-built-from-parts),
  each with its own text.
- **Links between pages.** `[[apps/ringio]]` or `[text](/#contact)` in Markdown, `href(&PAGE, lang)`
  in Rust; each one leads to the same language version of the target.
- **Tailwind-style utilities with types.** `style([pad(Space::M), bg(Color::Accent)])` on the
  element, design tokens from a theme struct, hover and breakpoint variants.
- **Images resized when used.** `image!` embeds a photo with an optional crop and target widths;
  only images that a page uses get resized, and the results are cached.
- **Multilingual from day one.** `content.md` holds the default language and `content.de.md` the
  German one. Every page is rendered in every language, with `hreflang` links, a language switch
  and an RSS feed per language.
- **Light and dark built in.** Colours are tokens with a light and a dark value. An optional
  700-byte toggle covers Auto, Light and Dark, and themed images load only the variant shown.
- **Blogs and feeds.** A folder of Markdown posts becomes one page per post, with dates, a list
  component and RSS.
- **A ready-made frame.** `standard_frame` writes the `<head>` (meta and Open Graph tags,
  canonical URL), a header with navigation and the footer; you fill in your site's parts.
- **Sitemap, headers, 404.** Setzer writes `sitemap.xml` and a `_headers` file for Cloudflare
  Pages, and a page with `path: "/404.html"` becomes a [404 page in every language](#fixed-paths-the-404-page).

## Crates

A site depends on `setzer` and on `maud` (maud's `html!` refers to its own crate by name):

```toml
[dependencies]
setzer = { git = "https://github.com/patmuk/setzer" }
setzer-theme-slate = { git = "https://github.com/patmuk/setzer" }  # optional: a ready theme
maud = "0.27"
```

`Cargo.lock` pins the exact Setzer commit; `cargo update -p setzer` moves to the newest one.

| Crate | Contents |
|---|---|
| `setzer` | the facade: everything below under one name, plus `setzer::prelude` |
| `setzer-core` | `Site`, `PageDef` and the `PAGES` registry, `Frame`, `href`/`anchor`, `Body`, `Doc`/`Document`, `Feed`, `ImageRef`, render state, writers |
| `setzer-macros` | `site!`, `image!`, `static_dir!` (`page!` is a plain `macro_rules!` in core) |
| `setzer-design` | tokens, `Theme`, utilities, `style`, `rule`, recorder, lint |
| `setzer-components` | `standard_frame`, `hero`, `facts`, `cards`, `timeline`, lists, `prose`, `picture`, layout presets |
| `setzer-theme-slate` | one palette; each theme is its own crate |

## Hello world

The example has one page, its text with front matter and a Markdown body, and one component,
which it uses twice. The complete crate is
[`examples/hello-world`](examples/hello-world); run it with `cargo run -p hello-world`.

```
hello-world/
  Cargo.toml              setzer, maud
  build.rs                println!("cargo::rerun-if-changed=src/www");
  src/main.rs             setzer::site!();  Site::new(...).write()
  src/www/mod.rs          the home page: setzer::page!() and fn render
  src/www/content.md      its text
```

```markdown
<!-- src/www/content.md -->
+++
title = "Hello World"
hello = "Hello"
world = "World"
+++

Everything above the closing `+++` is front matter: each key becomes a field of
`Content`, so `render` reads `c.title`, `c.hello` and `c.world`.

Everything below it is the **body**: ordinary Markdown that arrives as `c.body`
and renders as HTML wherever you place it with `(c.body)`.

## What the body can hold

- paragraphs, *emphasis* and **strong** text
- headings, lists and `inline code`
- links, such as [the Setzer repository](https://github.com/patmuk/setzer)

Links without a scheme point to pages of this site and are checked when you
build, so a link to a page that does not exist will not compile.
```

```rust
// src/main.rs
setzer::site!(); // the pages in src/www/ → module `www`

fn main() -> setzer::Result {
    setzer::Site::new("https://example.com", "Hello", &[setzer::Lang::En]).write()
}
```

```rust
// src/www/mod.rs
use setzer::prelude::*;

setzer::page!(); // this folder is a page: "/", its text from content.md as `Content`

pub fn render(c: &Content, lang: Lang) -> Html {
    html! {
        (DOCTYPE)
        html lang=(lang.code()) {
            head { meta charset="utf-8"; title { (c.title) } (stylesheet()) }
            body {
                h1 { (word(c.hello)) " " (word(c.world)) }
                (c.body) // the markdown below the front matter
            }
        }
    }
}

/// One component, used twice.
fn word(label: &str) -> Html {
    html! {
        span class=(style([inline_block(), pad(Space::M), radius(Radius::M), bg(Color::AccentSoft), fg(Color::AccentFg), text(TextRole::H1)])) {
            (label)
        }
    }
}
```

This is the output in `target/site/`:

```html
<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><title>Hello World</title>
<link rel="stylesheet" href="/css/site-<hash>.css"></head><body>
<h1><span class="inline-block p-m rounded-m bg-accent-soft text-accent-fg text-h1">Hello</span>
<span class="inline-block p-m rounded-m bg-accent-soft text-accent-fg text-h1">World</span></h1>
<p>Everything above the closing <code>+++</code> is front matter: …</p>
<p>Everything below it is the <strong>body</strong>: …</p>
<h2>What the body can hold</h2>
<ul><li>paragraphs, <em>emphasis</em> and <strong>strong</strong> text</li> …</ul>
…</body></html>
```

The stylesheet contains the theme's tokens, the base rules, and those six utilities, each once.
This page writes its own `<head>`, and with no frame registered Setzer adds nothing behind your
back. A real site registers a frame (`.frame(frame)`) and uses `setzer::components::standard_frame` for
the `<head>`, header and footer.

## How it works: compile, then run

A Setzer site is an ordinary Rust program, so you build and preview it with cargo:

| Command | What it does |
|---|---|
| `cargo run` | compiles the site and writes it to `target/site/` |
| `cargo run -- --serve` | the same, then serves it at <http://127.0.0.1:1111/> until Ctrl-C |
| `cargo run -- --serve 8080` | the same on another port |
| `cargo watch -x "run -- --serve"` | rebuilds on every change, and the open pages reload themselves |
| `cargo run --release -- --out dist` | an optimised build into another folder, e.g. for publishing |

Every run ends with a summary and a hint:

```
setzer: wrote 39 pages in 2 languages to target/site
        preview: cargo run -- --serve
```

With `--serve` it prints the link to open instead:

```
setzer: wrote 39 pages in 2 languages to target/site
setzer: serving http://127.0.0.1:1111/  (Ctrl-C to stop; pages reload on rebuild)
```

The server is built in and needs nothing else installed. It behaves like the static host will:
`/blog/` serves `blog/index.html`, `/blog` redirects to `/blog/`, and an unknown address gets
the closest `404.html`, so `/de/nope` shows the German 404 page. It only listens on your own
machine.

**Live reload.** The pages the server sends contain a small script; the files in `target/site/`
stay script-free. The script keeps a connection open to the server. When `cargo watch` restarts
the server after a change, the script reconnects, sees the new build and reloads the page. So:
save a file, and the browser shows the result a moment later. `cargo watch` comes from `cargo install cargo-watch`. (Opening `target/site/index.html`
straight from disk doesn't work well, because links like `/blog/` point at the root of a
server, not at a folder on disk.)

What happens inside `cargo run`:

1. **Compile.** `site!()` reads `src/www/` and turns it into typed Rust:
   - each page folder's Markdown files become structs (`content.md` → `struct Content`);
   - a collection folder becomes typed documents (`documents: "posts"` → `post::documents(lang)`);
   - `image!` turns a photo into an `ImageRef`.

   Mistakes stop the build here and show up in your editor: a translation missing a key, a typo
   like `c.titel`, or a link to a page that does not exist.
2. **Run.** `write()` renders every page in every language into `target/site/` (or into
   `--out <dir>`). While the pages render, each `style([...])`, image and link records what it
   needs. Afterwards Setzer writes exactly that CSS and exactly those image sizes. It also checks
   every link to a section: if a link points at `#contact` and no element has `id="contact"`,
   the run stops before anything is written.

`site!`, `image!` and `static_dir!` are procedural macros, which means Rust code that runs inside
the compiler. They also run inside rust-analyzer, so your editor knows every generated type and
can autocomplete `c.` to the keys of your Markdown file. The macros read files relative to the
crate root, because that is the one location the editor can tell them.

Cargo cannot know that a new Markdown file appeared, so each site's `build.rs` tells it which
folders to watch: `println!("cargo::rerun-if-changed=src/www")`. Cargo reads this build-script
output as an instruction; it is not a log message.

## Pages

### The module tree is the site

```
src/
  main.rs            setzer::site!();   Site::new(...).theme(...).frame(...).write()
  frame.rs           header, footer, <head>: fn frame(page: &Frame, body: Html) -> Html
  www/               the home page, /           mod.rs  content.md  content.de.md  images/
    apps/            /apps/                    mod.rs  content.md
      ringio/        /apps/ringio/             mod.rs  content.md  faq.md  faq.rs
    blog/            /blog/                    mod.rs  content.md
      post/          /blog/<slug>/             mod.rs  content.md  posts/*.md
    privacy/         /privacy/                 mod.rs  privacy.md
    not_found/       /404.html, /de/404.html   mod.rs  content.md  content.de.md
```

`crate::www` is `/`, and `www::about::impressum` is `/about/impressum/` (an underscore in a
folder name becomes a dash in the URL). Folders that don't call `page!`, such as `images/` or
`posts/`, are left alone.

Every page folder's `mod.rs` contains two things:

```rust
setzer::page!();                                       // this folder is a page (+ options)
pub fn render(c: &Content, lang: Lang) -> Html { … }   // what it shows
```

`main.rs` doesn't list any pages. `site!()` takes the place of `mod www;`: it reads `src/www/`
and every sub-folder whose `mod.rs` calls `setzer::page!`, and it generates module `www` with one
module per page. Each folder's `mod.rs` becomes a child of its generated module. `page!` imports
everything that was generated (`Content`, `content`, `PAGE`, the sections and the sub-page
modules), and the page's `pub` items are re-exported, so other code can use `www::PORTRAIT`.
For that reason, `render` and anything another page uses must be `pub`.

**Turning a page off takes one character.** A folder whose name starts with `_` is skipped.
Rename `ringio/` to `_ringio/` and that page is gone, together with its text and everything
below it, and every link that still points to it stops compiling.

### `page!` options

| Option | Effect |
|---|---|
| *(none)* | an ordinary page; its text is passed to `render` as `c: &Content` |
| `documents: "posts"` | one page per Markdown file in `posts/`; `render(post: &'static Doc, c: &Content, lang: Lang)` |
| `path: "/404.html"` | a fixed output path instead of the folder's route, in every language (`/de/404.html`) |
| `feed: feed` | an RSS feed next to the page, built by the page's own `pub fn feed(lang: Lang) -> Feed` |
| `subpages: [a, b]` | lists the sub-pages by hand instead of finding them |

You can combine options: `setzer::page! { documents: "posts", feed: feed }`.

### Text and translations

The page's text is called `Content`, whatever its Markdown file is called:

| Markdown files in the folder | The page's `Content` |
|---|---|
| `content.md` | `content.md` |
| exactly one, any name (`privacy.md`) | that file |
| several | `content.md` if present; every other file needs a `.rs` file of the same name, which makes it a [section](#sections-a-page-built-from-parts) |
| none | an empty `Content`, for pages that are all code |

So a folder with a single `privacy.md` needs no configuration at all. Once a folder holds more
than one text, each one belongs to the `.rs` file with its name, and a text that belongs to
nothing is a compile error rather than a file that is silently ignored:

```
src/www/apps/ringio/faq.md: content.md is this page's text; for faq.md add faq.rs with
`pub fn render(c: &Content, lang: Lang) -> Html` (a section)
```

`site!()` turns the text into `struct Content` and `fn content(lang) -> &'static Content`.
Front-matter keys become `&'static str` fields, and the Markdown below the front matter becomes
`body`, which renders as HTML. The file without a language suffix is the fallback. The
languages are `en`, `de`, `fr`, `it` and `gsw` (Swiss German, which falls back to `de` before the
fallback). Every
`name.<lang>.md` must have the same keys as its fallback, or the build fails with an error like
this one:

```
src/www/content.de.md is missing the key `pitch` (compared with src/www/content.md)
```

The page's `title` and `description` also fill its `<head>`, and `title` becomes the link text of
`[[page]]` links that point to it.

### Sections: a page built from parts

A long page is easier to manage in parts. A **section** is a `.rs` file next to a `.md` file of
the same name in the page folder. It has no URL of its own; the page embeds it. Just like a page,
a section gets its own text as `c: &Content` in its `render` function.

```
www/apps/ringio/
  mod.rs            the page, /apps/ringio/
  content.md        the page's text: its <head> title and link text
  about_me.rs       a section …
  about_me.md       … and its text
  faq.rs            a section …
  faq.md            … and its text
  faq.de.md         … in German
```

```rust
// mod.rs
use setzer::prelude::*;

setzer::page!();

pub fn render(c: &Content, lang: Lang) -> Html {
    html! { h1 { (c.title) } (about_me(lang)) (faq(lang)) }
}
```

```rust
// faq.rs
use super::*;
use setzer::prelude::*;

pub fn render(c: &Content, _lang: Lang) -> Html {
    html! { section { h2 { (c.title) } dl { dt { (c.question) } dd { (c.answer) } } } }
}
```

```markdown
<!-- faq.md -->
+++
title = "FAQ"
question = "Why Rust?"
answer = "Types."
+++
```

`about_me.rs` looks the same and shows `c.title` and `c.body`. For every pair, `site!()`
generates two things in the page's module:

- `mod faq`, holding the section's own `Content` and `content(lang)`, with `faq.rs` attached
  inside it. That is why `use super::*` in `faq.rs` finds *its* text, not the page's.
- `fn faq(lang) -> Html`, which renders the section with its text in `lang`. The page calls
  `(faq(lang))` wherever the section belongs.

Don't write `mod faq;` yourself; `site!()` already does, and it tells you so if you try. Sections
get the same checks as pages: `c.questoin` is a compile error in `faq.rs`, and a `faq.de.md`
without an `answer` stops the build. To reuse a section on several pages, move it into a
component function in a shared module and pass the text in.

To summarise: **a folder with a `mod.rs` that calls `page!()` is a page with a URL, and a
`.rs` + `.md` pair inside it is a section that the page embeds.**

### Fixed paths: the 404 page

Most pages get their URL from their folder. A page that a host expects at a particular path, such
as the 404 page, sets the path itself:

```rust
// src/www/not_found/mod.rs
setzer::page! { path: "/404.html" }
```

The folder can sit anywhere under `src/www/`, and `path` decides where the file is written. Like
every page, it is rendered in each language: `content.md` gives `/404.html` and `content.de.md`
gives `/de/404.html`. Cloudflare Pages, for example, serves the closest `404.html` up the directory tree, so a
visitor who mistypes a German URL gets the German page. The folder needs a name that is a valid
Rust module name, so `not_found/` works and `404/` does not.

### Blogs and other collections

A blog is two pages: a list and the posts. The folder names are yours to choose; here the list is
`blog/` and the posts live one level down in `blog/post/`:

```
www/blog/                 /blog/           mod.rs  content.md          ← the list, the RSS feed
www/blog/post/            /blog/<slug>/    mod.rs  content.md          ← one page per post
www/blog/post/posts/                       2014-03-09-hello.md  2014-03-09-hello.de.md  …
```

`setzer::page! { documents: "posts" }` in `blog/post/mod.rs` turns every Markdown file in its
`posts/` folder into a page. The date comes from the file name or from the front matter, and
`title` is required. The slug is the file name without date and language, and it takes the place
of the module's name in the URL: `2014-03-09-hello.md` is served at `/blog/hello/`, and its
German version at `/de/blog/hello/`. Posts without a German file simply don't exist in German.

`blog/post/mod.rs` is one page *per post*: Setzer calls its `render` once for every post in
every language. It gets two texts:

- `post`: this post, from its file in `posts/`, already in the right language;
- `c`: what every post page shares, from `blog/post/content.md` ("Published", "Also available
  in", "Back to all articles").

```rust
// src/www/blog/post/mod.rs
setzer::page! { documents: "posts" }

pub fn render(post: &'static Doc, c: &Content, lang: Lang) -> Html {
    html! {
        h1 { (post.title) }
        p { (c.posted_on) " " (post.date("%Y-%m-%d")) (translations(post, c.also_in)) }
        (post.body)
    }
}
```

`translations` is a component: " · Also available in Deutsch", or nothing for a post that exists
in one language only.

Besides the usual `PAGE`, `Content` and `content(lang)`, the module `post` has two more items:

| Item | What it is |
|---|---|
| `post::documents(lang)` | the posts that exist in `lang`, newest first, as `&'static Doc` |
| `post::DOCUMENTS` | every post with all its languages, for the rare case that you need them all |

A `Doc` has `title`, `date(fmt)`, `description`, `body` and `slug`, and it knows its page and
language: `post.href()` is its URL and `post.translations()` are its other languages. That makes
the list page short:

```rust
// src/www/blog/mod.rs
setzer::page! { feed: feed }

pub fn render(c: &Content, lang: Lang) -> Html {
    let posts: Vec<&Doc> = post::documents(lang).collect();
    html! { h1 { (c.title) } (doc_list(&posts)) }
}

pub fn feed(lang: Lang) -> Feed {
    Feed {
        title: content(lang).title.into(),
        description: content(lang).intro.into(),
        items: post::documents(lang).map(FeedItem::from).collect(),
    }
}
```

Note that `doc_list(&posts)` is a component predefined in Setzer
(`setzer::components::doc_list`). See [Components](#components) below for the built-in ones and
for [writing your own](#your-own-components), for example to show the posts as tiles instead.

`feed: feed` adds an RSS feed for each language (`/blog/rss.xml`, `/de/blog/rss.xml`) and links
it from the page's `<head>`. The page writes `feed` itself because only the page knows what
belongs in it: here the posts, but it could as well be a changelog or a list of releases.
`FeedItem::from(post)` turns a post into an item. The components `doc_list` (date, title,
description) and `doc_teasers` (year and title, e.g. the latest four on the home page) take the
documents directly; they are named after `Doc`, not after blogs, because a collection can just as
well hold releases, talks or case studies.

### Links

You can write links in Markdown. Any link without a scheme is internal, and Setzer checks it.

| Markdown | Links to | Checked |
|---|---|---|
| `[[apps/ringio]]` | that page, using its `title` as the text | compile time |
| `[text](apps/ringio)`, `[text](/)` | that page | compile time |
| `[text](/#contact)`, `[text](#faq)` | a section of a page | before writing |
| `[text](cv.pdf)` | a file in `static/` | compile time |
| `[text](https://…)`, `[text](mailto:…)` | left untouched | no |

The same links can be written in Rust: `href(&crate::www::apps::PAGE, lang)`,
`anchor(&crate::www::PAGE, "contact", lang)` and `post.href()` for a post. Links always lead
to the same language version of the target page.

**Orphan pages are reported.** After rendering, Setzer follows every link (in Markdown, in
components and in the frame), starting from each language's home page and from fixed-path pages
such as `/404.html`. Any page that no link reaches is printed as a warning. With
`.orphans_fail()` it stops the run instead.

### Images and static files

```rust
pub static PHOTO: ImageRef =
    setzer::image!("src/www/images/photo.jpg", crop: (x, y, w, h), widths: [640, 1280]);
```

The path starts at the crate root. `image!` embeds the file, `picture(&PHOTO, alt, sizes)` renders a
lazy-loading `<img>` with a `srcset` and fixed dimensions, and `themed_picture` picks a separate
light or dark variant without loading both. Only images that some page uses are resized, and the resized versions are
cached between builds.

`.static_files(setzer::static_dir!("static"))` embeds a folder (favicon, PDFs, `robots.txt`)
and copies it to the site root.

### The frame

The frame is what every page shares: the `<head>`, the header and the footer. Pages only render
their own content, and the frame wraps it. A frame is a plain function that you register once in
`main`:

```rust
setzer::Site::new("https://example.com", "Example", &[Lang::En, Lang::De])
    .frame(frame::frame)                 // fn frame(page: &Frame, body: Html) -> Html
    .write()
```

Setzer calls it for every page in every language. `body` is what the page's `render` returned,
and `page: &Frame` describes the page being wrapped:

| `Frame` field | Example |
|---|---|
| `lang` | `Lang::De` |
| `path` | `/de/blog/` |
| `title`, `description` | from the page's text, if it has them |
| `canonical` | `https://example.com/de/blog/` |
| `alternates` | this page in every language it exists in, for `hreflang` |
| `switches` | the other languages: this page there, or that language's home if it doesn't exist there |
| `feed` | the page's RSS feed, if it has one |
| `document` | `true` for a blog post, so the frame can mark it as an article |

You can define the frame in three ways.

**1. No frame.** Without `.frame(…)`, each page writes its complete HTML itself, `<head>`
included, like the [hello world](#hello-world) above. That suits a single page, but every
additional page would repeat the head.

**2. The ready-made `standard_frame`.** `setzer::components::standard_frame` is a complete,
ready-to-use frame. You fill in `FrameParts` with what is specific to your site, and
`standard_frame` does the rest:

```rust
// src/frame.rs
use crate::www;
use setzer::components::{standard_frame, FrameParts, NavItem, ThemeLabels};
use setzer::prelude::*;

pub fn frame(page: &Frame, body: Html) -> Html {
    let lang = page.lang;
    let parts = FrameParts {
        home: href(&www::PAGE, lang),
        nav: vec![
            NavItem { label: "Blog".into(), href: href(&www::blog::PAGE, lang) },
            NavItem { label: "Contact".into(), href: anchor(&www::PAGE, "contact", lang) },
        ],
        skip: "Skip to content".into(),
        favicon: "/favicon.svg".into(),
        logo: Some("/logo.svg".into()),                     // shown before the site name; None for text only
        description: "Websites built with Setzer.".into(), // for pages without a description
        og_image: Some("/og.png".into()),                   // preview image when the link is shared
        footer: html! { p { "© 2026 Example" } },
        theme: Some(ThemeLabels { label: "Theme".into(), auto: "Auto".into(), light: "Light".into(), dark: "Dark".into() }),
    };
    standard_frame(page, &parts, body)
}
```

This gives every page:

- a `<head>` with title (`Blog · Example`), description, canonical URL, `hreflang` alternates,
  the RSS link, Open Graph tags for link previews, the favicon and the stylesheet;
- a sticky header with the logo and site name, the navigation, the language switch
  (`English · Deutsch`) and the Auto/Light/Dark toggle (with `theme: None` the site follows the
  system setting and ships no script at all);
- a skip link for keyboard users, the page in `<main>`, and your footer.

For a multilingual site, take the labels from your text instead of writing them in Rust, for
example `skip: www::content(lang).skip.into()`. A missing translation is then a compile error,
like everywhere else.

**3. Your own frame.** When the standard frame's layout doesn't fit, write the frame yourself. The
`Frame` fields give you everything that depends on the page:

```rust
// src/frame.rs
use crate::www;
use setzer::prelude::*;

pub fn frame(page: &Frame, body: Html) -> Html {
    let alternate = |(l, url): &(Lang, String)| html! { link rel="alternate" hreflang=(l.code()) href=(url); };
    html! {
        (DOCTYPE)
        html lang=(page.lang.code()) {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (page.title.unwrap_or(setzer::site_name())) }
                link rel="canonical" href=(page.canonical);
                (each(page.alternates.iter().map(alternate)))
                (stylesheet())
            }
            body {
                header { a href=(href(&www::PAGE, page.lang)) { (setzer::site_name()) } }
                main { (body) }
                footer { "© 2026 Example" }
            }
        }
    }
}
```

In between, you can keep `standard_frame` and wrap the page yourself, for example to put a banner above
every page: `standard_frame(page, &parts, html! { (notice("We are moving!")) (body) })`. You can also
copy `standard_frame` from [`crates/setzer-components/src/frame.rs`](crates/setzer-components/src/frame.rs)
and change it, as with any other component.

## Styling

Styling follows the same model as Tailwind v4, with types:

| | Tailwind | Setzer |
|---|---|---|
| Design tokens | `@theme { --color-accent: … }` | `Theme`: a Rust struct with defaults, emitted as CSS variables |
| Per project | override the theme | `Theme { light, dark, ..Theme::default() }`, or a theme crate |
| On elements | `class="p-4 bg-accent"` | `class=(style([pad(Space::M), bg(Color::Accent)]))` |
| Variants | `hover:` `md:` | `hover(…)`, `at(Breakpoint::Narrow, …)`, `first(…)` |
| Override a component | pass more classes (and `tailwind-merge`) | `Style::with`: same property and variant, later wins |
| Global rules | `@layer base { button { … } }` | `.css(rule("button", [bg(Color::Accent)]))` in `main` |
| Generated CSS | only classes found in sources | only utilities rendered by pages |

`pad(Color::Fg)` doesn't compile, and `pad(4px)` can't be written. Utilities cover styling that
sits on a single element. When styling depends on something else (Markdown output, `::before`, or
colours that depend on a photo behind them), a component uses one **named** class with a typed
rule set: `style([named("prose", prose_css)])` with `rule(".prose pre", [pad(Space::M)])`. The
stylesheet is ordered like Tailwind's layers: base, named classes, utilities, variants,
breakpoints.

The tokens are `Space` (an 8pt grid by default, `SpaceScale::grid(8.0)`, and configurable),
`Radius`, `Stroke`, `Size`, `Width`, `Color` (with light and dark values), `Font`, `TextRole`,
`Shadow` and `Breakpoint`. The vocabulary mirrors a Flutter `AppTheme`, so a company's apps and
its websites can share one source of truth. A raw keyword property can be set with
`raw("object-fit", "cover")`, and the run fails if a length or colour sneaks in there.

Web fonts are files of the site (e.g. in `static/fonts/`), declared once and named in the theme:

```rust
Site::new(…)
    .theme(Theme { fonts: Fonts { heading: "\"Fraunces\", Georgia, serif", ..Fonts::default() }, ..Theme::default() })
    .css(Css::font_face("Fraunces", "/fonts/fraunces.woff2", "100 900", false))
```

The generated Content-Security-Policy allows fonts from the site itself only, so there are no
requests to font services.

Light and dark follow the system setting. `theme_script()` adds a toggle that cycles through
Auto, Light and Dark. It is about 700 bytes and stores the choice in `localStorage`. Its hash goes
into the generated Content-Security-Policy.

A page that needs a script of its own includes it with `setzer::script`:

```rust
html! {
    a #open href="#" class=(Look::Primary.style()) { "Open in the app" }
    (setzer::script(include_str!("open.js")))   // open.js next to the page's mod.rs
}
```

Its hash joins the Content-Security-Policy, so exactly the scripts your pages use can run. Extra
rules for the `_headers` file come from `main`, for example
`.header("/.well-known/apple-app-site-association", "Content-Type", "application/json")`.
Files in `static/` are copied as they are, `.well-known/` and Cloudflare's `_redirects`
included.

## Components

A component is a plain Rust function that returns `Html`. There is no trait to implement, no
registry and no plugin system. A function takes whatever data it needs and styles its markup with
`style([...])`.

### Built-in components

`setzer::components` comes with `standard_frame`, `button`, `hero`, `facts`, `intro` (text beside
a picture), `features` (rows that alternate sides, for product pages), `cards`, `terms`,
`titled_list`, `timeline`, `doc_list`, `doc_teasers`, `translations`, `contact`, `meta_line`,
`notice`, `accordion`, `picture`, `themed_picture`, `prose` and a few layout presets
(`section`, `narrow_section`, `split`, `two_col`). They live in
[`crates/setzer-components`](crates/setzer-components), and each one is short enough to read in
a minute.

### Your own components

Say the blog should show its posts as a plain list in one place and as tiles in another. Both are
functions in your site:

```rust
// src/components.rs   (declared with `mod components;` in main.rs)
use setzer::prelude::*;

/// A plain list: one link per document.
pub fn plain_list(docs: &[&Doc]) -> Html {
    let item = |d: &&Doc| html! { li { a href=(d.href()) { (d.title) } } };
    html! { ul { (each(docs.iter().map(item))) } }
}

/// Tiles: three per row, two on tablets, one on phones.
pub fn tiles(docs: &[&Doc]) -> Html {
    let tile = |d: &&Doc| html! {
        a href=(d.href()) class=(style([flex(), flex_col(), gap(Space::Xs), pad(Space::M), radius(Radius::M),
                                        bg(Color::Surface), border(Stroke::Hairline, Color::Rule),
                                        hover(border_color(Color::Accent)), hover(no_underline())])) {
            span class=(style([text(TextRole::Small), fg(Color::Muted)])) { (d.date("%Y")) }
            strong class=(style([text(TextRole::H3), fg(Color::Fg)])) { (d.title) }
            @if let Some(text) = d.description { span class=(style([fg(Color::Muted)])) { (text) } }
        }
    };
    html! {
        div class=(style([grid(), columns(3), gap(Space::M),
                          at(Breakpoint::Narrow, columns(2)), at(Breakpoint::Phone, columns(1))])) {
            (each(docs.iter().map(tile)))
        }
    }
}
```

A page uses them like any other function:

```rust
// src/www/blog/mod.rs
use crate::components::tiles;

pub fn render(c: &Content, lang: Lang) -> Html {
    let posts: Vec<&Doc> = post::documents(lang).collect();
    html! { h1 { (c.title) } (tiles(&posts)) }
}
```

Things to know:

- **The input is up to you.** These two take `&[&Doc]`, so they work for every collection. Others
  take a struct of their own, like the built-in `cards(&[Card])`, or just a `&str`.
- **The styling is checked.** Spacing, colours, radii and type sizes come from the theme, so a
  tile can't use a colour the theme doesn't have. `at(Breakpoint::Phone, …)` is the responsive
  variant, `hover(…)` the hover state. Only the rules that actually render reach the stylesheet,
  so a component that no page uses costs nothing.
- **Put it where it is used.** A component for one page can sit in that page's `mod.rs`, like
  `word()` in hello-world. Shared ones go in a module such as `src/components.rs`, and ones worth
  sharing between sites go in a crate of their own. Avoid a `components.md` next to a
  `components.rs` inside `src/www/`, because that pair would be a [section](#sections-a-page-built-from-parts).
- **To change a built-in component, copy it.** `tiles` above is `cards` adapted to documents.
  Copy the function from `setzer-components` into your module, rename it and edit it. Your copy
  is independent of Setzer's updates.

## Embedding app code (WASM)

A widget crate with `crate-type = ["cdylib"]` and `wasm-bindgen` builds to a few kilobytes. To
embed it, copy the `.wasm` file and its glue code via `static_dir!`, emit a mount point and a
`<script type="module">` loader from a component with `setzer::script`, which adds its hash to
the CSP. Setzer doesn't build the WASM for you yet. Use WASM where it is faster than JavaScript, and nowhere else.

## Deploy

Setzer's output works on any static host. Keep only the sources in version control and upload
`target/site`. On Cloudflare Pages, run `wrangler pages deploy target/site --project-name <name>`;
Cloudflare picks up the generated `_headers` file automatically.

## Troubleshooting

- **The editor shows "proc-macro map is missing error entry for crate".** rust-analyzer still
  holds an old build of the Setzer macros. Restart the language server (in Zed:
  `editor: restart language server`; in VS Code: `rust-analyzer: Restart server`). This only
  happens after the macros themselves change.
- **A new Markdown file isn't picked up.** Check that `build.rs` contains
  `println!("cargo::rerun-if-changed=src/www")`.
- **The build says an item in a page's `mod.rs` is private.** `render`, `feed` and anything
  another page uses must be `pub`, because the generated code calls them from outside.

## Status

Setzer is early. It was built for [patrick.mukherjee.de](https://patrick.mukherjee.de) and more
of my sites, and the API might change. Contributions are welcome if they keep to the rules:
No template language, no framework JavaScript, no literal values in a style, and nothing hidden.
that can't be explained in one sentence.

## License

MIT.
